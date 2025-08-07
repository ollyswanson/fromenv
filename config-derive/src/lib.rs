mod field;

use darling::{
    ast::{Data, Fields},
    FromDeriveInput,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, ExprPath, Ident, Visibility};

use crate::field::{ConfigAttribute, ConfigFieldReceiver};

#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_derive(input) {
        Ok(output) => output.into(),
        Err(err) => err.write_errors().into(),
    }
}

fn impl_derive(input: DeriveInput) -> darling::Result<TokenStream> {
    let config_struct = match ConfigReceiver::from_derive_input(&input) {
        Ok(config_struct) => config_struct,
        Err(e) => {
            return Err(e);
        }
    };

    config_struct.validate()?;

    Ok(config_struct.to_token_stream())
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
struct ConfigReceiver {
    pub ident: Ident,
    pub vis: Visibility,
    pub data: Data<(), ConfigFieldReceiver>,
}

struct ConstTokens {
    private_path: TokenStream,
    errors_ident: TokenStream,
    builder_name: Ident,
}

impl ToTokens for ConfigReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let consts = ConstTokens {
            builder_name: format_ident!("{}Builder", self.ident),
            private_path: quote!(__config::__private),
            errors_ident: quote!(__config_derive_builder_errors),
        };
        let private_path = &consts.private_path;

        let impl_struct = self.impl_struct(&consts);
        let builder_struct = self.builder_struct(&consts);
        let impl_configurable = self.impl_configurable(&consts);
        let impl_config_builder = self.impl_config_builder(&consts);
        let impl_builder = self.impl_builder(&consts);

        let derive = quote! {
            const _: () = {
                extern crate config as __config;
                use #private_path::Parser as _;

                #impl_struct

                #builder_struct

                #impl_configurable

                #impl_config_builder

                #impl_builder
            };
        };

        tokens.extend(derive);
    }
}

impl ConfigReceiver {
    fn validate(&self) -> darling::Result<()> {
        if !matches!(&self.vis, Visibility::Public(_)) {
            let err = darling::Error::custom("Config derive requires a public struct")
                .with_span(&self.ident.span());
            Err(err)
        } else {
            Ok(())
        }
    }

    fn builder_struct(&self, consts: &ConstTokens) -> TokenStream {
        let private_path = &consts.private_path;
        let builder_name = &consts.builder_name;
        let fields = self.get_fields().iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.option.as_ref().unwrap_or(&field.ty);

            match &field.config_attr {
                ConfigAttribute::Nested => {
                    quote! { #ident: Option<<#ty as #private_path::Configurable>::ConfigBuilder> }
                }
                ConfigAttribute::Flat { .. } | ConfigAttribute::None => {
                    quote! { #ident: Option<#ty> }
                }
            }
        });

        quote! {
            pub struct #builder_name {
                #(#fields,)*
            }
        }
    }

    fn impl_struct(&self, consts: &ConstTokens) -> TokenStream {
        let struct_name = &self.ident;
        let private_path = &consts.private_path;
        let builder_name = &consts.builder_name;

        quote! {
            impl #struct_name {
                pub fn configure() -> #builder_name {
                    <Self as #private_path::Configurable>::configure()
                }
            }
        }
    }

    fn impl_configurable(&self, consts: &ConstTokens) -> TokenStream {
        let struct_name = &self.ident;
        let builder_name = &consts.builder_name;
        let private_path = &consts.private_path;

        let fields = self.get_fields().iter().map(|field| {
            let ident = &field.ident;
            let ty = field.option.as_ref().unwrap_or(&field.ty);

            match &field.config_attr {
                ConfigAttribute::Nested => {
                    quote! { #ident: Some(<#ty as #private_path::Configurable>::configure()) }
                }
                ConfigAttribute::Flat { .. } | ConfigAttribute::None => {
                    quote! { #ident: None }
                }
            }
        });

        quote! {
            impl #private_path::Configurable for #struct_name {
                type ConfigBuilder = #builder_name;

                fn configure() -> Self::ConfigBuilder {
                    #builder_name {
                        #(#fields,)*
                    }
                }
            }
        }
    }

    fn impl_config_builder(&self, consts: &ConstTokens) -> TokenStream {
        let struct_name = &self.ident;
        let private_path = &consts.private_path;
        let errors_ident = &consts.errors_ident;
        let builder_name = &consts.builder_name;

        // This is the secret sauce that lets us gather all errors before
        // failing.
        let assignments = self.get_fields().iter().map(|field| {
            let ident = &field.ident;
            match (&field.config_attr, field.option.is_some()) {
                // #[config(nested)] field: T,
                (ConfigAttribute::Nested, false) => {
                    quote! {
                        let #ident = match #private_path::ConfigBuilder::finalize(self.#ident.take().unwrap()) {
                            Ok(inner) => Ok(inner),
                            Err(errors) => {
                                #errors_ident.extend(errors);
                                Err(())
                            }
                        };
                    }
                }
                // #[config(nested)] field: Option<T>,
                (ConfigAttribute::Nested, true) => {
                    quote! {
                        let #ident = match #private_path::ConfigBuilder::finalize(self.#ident.take().unwrap()) {
                            Ok(inner) => Ok(Some(inner)),
                            Err(errors) if errors.only_missing_errors() => Ok(None),
                            Err(errors) => {
                                #errors_ident.extend(errors);
                                Err(())
                            }
                        };
                    }
                }
                // #[config(env = "...", default = ...)] field: T
                (
                    ConfigAttribute::Flat {
                        env,
                        with,
                        default: Some(default),
                    },
                    false,
                ) => {
                    let with = parser_path(consts, with.as_ref());

                    quote! {
                        let #ident = if let Some(inner) = self.#ident {
                            Ok(inner)
                        } else {
                            match #with.parse_from_env(#env) {
                                Some((_, Ok(val))) => Ok(val),
                                Some((value, Err(error))) => {
                                    let err = #private_path::ConfigError::ParseError {
                                        env_var: #env.to_string(),
                                        value,
                                        error,
                                    };
                                    #errors_ident.add(err);
                                    Err(())
                                }
                                None => {
                                    #with.parse(#default).map_err(|error| {
                                        let err = #private_path::ConfigError::ParseError {
                                            env_var: #env.to_string(),
                                            value: #default.to_string(),
                                            error: error.into(),
                                        };
                                        #errors_ident.add(err);
                                    })
                                },
                            }
                        };
                    }
                }
                // #[config(env = "...")] field: T
                (
                    ConfigAttribute::Flat {
                        env,
                        with,
                        default: None,
                    },
                    false,
                ) => {
                    let with = parser_path(consts, with.as_ref());

                    quote! {
                    let #ident = if let Some(inner) = self.#ident {
                        Ok(inner)
                    } else {
                        match #with.parse_from_env(#env) {
                            Some((_, Ok(val))) => Ok(val),
                            Some((value, Err(error))) => {
                                let err = #private_path::ConfigError::ParseError {
                                    env_var: #env.to_string(),
                                    value,
                                    error,
                                };
                                #errors_ident.add(err);
                                Err(())
                            }
                            None => {
                                let err = #private_path::ConfigError::MissingEnv {
                                    env_var: #env.to_string(),
                                };
                                #errors_ident.add(err);
                                Err(())
                            }
                        }
                    };
                    }
                }
                // #[config(env = "...")] field: Option<T>
                (
                    ConfigAttribute::Flat {
                        env,
                        with,
                        default: None,
                    },
                    true,
                ) => {
                    let with = parser_path(consts, with.as_ref());

                    quote! {
                        let #ident = if let Some(inner) = self.#ident {
                            Ok(Some(inner))
                        } else {
                            match #with.parse_from_env(#env) {
                                Some((_, Ok(val))) => Ok(Some(val)),
                                Some((value, Err(error))) => {
                                    let err = #private_path::ConfigError::ParseError {
                                        env_var: #env.to_string(),
                                        value,
                                        error,
                                    };
                                    #errors_ident.add(err);
                                    Err(())
                                }
                                None => {
                                    Ok(None)
                                }
                            }
                        };
                    }
                }
                // #[config(default = "...")] field: Option<T>
                (
                    ConfigAttribute::Flat {
                        env: _,
                        with: _,
                        default: Some(_),
                    },
                    true,
                ) => unreachable!("we've already checked that Optional fields can't have a default"),
                (ConfigAttribute::None, false) => {
                    let ident_string = ident.to_string();
                    quote! {
                        let #ident = match self.#ident {
                            Some(inner) => Ok(inner),
                            None => {
                                let err = #private_path::ConfigError::MissingValue {
                                    field: String::from(#ident_string),
                                };
                                #errors_ident.add(err);
                                Err(())
                            }
                        };
                    }
                }
                (ConfigAttribute::None, true) => {
                    quote! {
                        let #ident = self.#ident;
                    }
                }
            }
        });

        let fields = self.get_fields().iter().map(|field| {
            let ident = &field.ident;

            quote! {
                #ident: match #ident {
                    Ok(val) => val,
                    Err(_) => {
                        return Err(#errors_ident);
                    }
                }
            }
        });

        quote! {
            impl #private_path::ConfigBuilder for #builder_name {
                type Target = #struct_name;

                fn finalize(mut self) -> Result<Self::Target, #private_path::ConfigErrors> {
                    let mut #errors_ident = #private_path::ConfigErrors::new();

                    #(#assignments)*

                    Ok(#struct_name {
                        #(#fields,)*
                    })
                }
            }
        }
    }

    fn impl_builder(&self, consts: &ConstTokens) -> TokenStream {
        let private_path = &consts.private_path;
        let builder_name = &consts.builder_name;

        let setters = self.get_fields().iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.option.as_ref().unwrap_or(&field.ty);

            match &field.config_attr {
                ConfigAttribute::Nested => {
                    quote! {
                        pub fn #ident<F>(mut self, f: F) -> Self
                        where
                            F: FnOnce(<#ty as #private_path::Configurable>::ConfigBuilder) -> <#ty as #private_path::Configurable>::ConfigBuilder,
                        {
                            let nested = self.#ident.take().unwrap();
                            let nested = f(nested);
                            self.#ident = Some(nested);
                            self
                        }
                    }
                }
                ConfigAttribute::Flat { .. } | ConfigAttribute::None => {
                    quote! {
                        pub fn #ident(mut self, #ident: #ty) -> Self {
                            self.#ident = Some(#ident);
                            self
                        }
                    }
                }
            }
        });

        quote! {
            impl #builder_name {
                #(#setters)*

                pub fn finalize(self) -> Result<<Self as #private_path::ConfigBuilder>::Target, #private_path::ConfigErrors> {
                    #private_path::ConfigBuilder::finalize(self)
                }
            }
        }
    }

    fn get_fields(&self) -> &Fields<ConfigFieldReceiver> {
        let Data::Struct(fields) = &self.data else {
            panic!("we've asserted that it's a struct");
        };

        fields
    }
}

fn parser_path(consts: &ConstTokens, path: Option<&ExprPath>) -> TokenStream {
    let private_path = &consts.private_path;

    if let Some(expr_path) = path {
        if let Some(ident) = expr_path.path.get_ident() {
            let ident_str = ident.to_string();

            match ident_str.as_str() {
                "from_str" => return quote!(#private_path::from_str),
                "into" => return quote!(#private_path::into),
                _ => {}
            }
        }

        return quote!(#expr_path);
    }

    quote!(#private_path::from_str)
}
