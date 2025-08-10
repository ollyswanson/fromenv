mod field;

use darling::{
    FromDeriveInput,
    ast::{Data, Fields},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, ExprPath, Ident, Visibility, parse_macro_input};

use crate::field::{EnvAttribute, FromEnvFieldReceiver};

#[proc_macro_derive(FromEnv, attributes(env))]
pub fn derive_from_env(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_derive(input) {
        Ok(output) => output.into(),
        Err(err) => err.write_errors().into(),
    }
}

fn impl_derive(input: DeriveInput) -> darling::Result<TokenStream> {
    let config_struct = match FromEnvReceiver::from_derive_input(&input) {
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
struct FromEnvReceiver {
    pub ident: Ident,
    pub vis: Visibility,
    pub data: Data<(), FromEnvFieldReceiver>,
}

struct ConstTokens {
    private_path: TokenStream,
    errors_ident: TokenStream,
    builder_name: Ident,
}

impl ToTokens for FromEnvReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let consts = ConstTokens {
            builder_name: format_ident!("{}Builder", self.ident),
            private_path: quote!(__fromenv::__private),
            errors_ident: quote!(__fromenv_derive_builder_errors),
        };
        let private_path = &consts.private_path;

        let impl_struct = self.impl_struct(&consts);
        let builder_struct = self.builder_struct(&consts);
        let impl_from_env = self.impl_from_env(&consts);
        let impl_from_env_builder = self.impl_from_env_builder(&consts);
        let impl_builder = self.impl_builder(&consts);

        let derive = quote! {
            const _: () = {
                extern crate fromenv as __fromenv;
                use #private_path::Parser as _;

                #impl_struct

                #builder_struct

                #impl_from_env

                #impl_from_env_builder

                #impl_builder
            };
        };

        tokens.extend(derive);
    }
}

impl FromEnvReceiver {
    fn validate(&self) -> darling::Result<()> {
        if !matches!(&self.vis, Visibility::Public(_)) {
            let err = darling::Error::custom("FromEnv derive requires a public struct")
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

            match &field.env_attr {
                EnvAttribute::Nested => {
                    quote! { #ident: Option<<#ty as #private_path::FromEnv>::FromEnvBuilder> }
                }
                EnvAttribute::Flat { .. } | EnvAttribute::None => {
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
                pub fn from_env() -> #builder_name {
                    <Self as #private_path::FromEnv>::from_env()
                }

                pub fn requirements() -> String {
                    let mut requirements = ::std::string::String::new();
                    <Self as #private_path::FromEnv>::requirements(&mut requirements);
                    requirements
                }
            }
        }
    }

    fn impl_from_env(&self, consts: &ConstTokens) -> TokenStream {
        let struct_name = &self.ident;
        let builder_name = &consts.builder_name;
        let private_path = &consts.private_path;

        let fields = self.get_fields().iter().map(|field| {
            let ident = &field.ident;
            let ty = field.option.as_ref().unwrap_or(&field.ty);

            match &field.env_attr {
                EnvAttribute::Nested => {
                    quote! { #ident: Some(<#ty as #private_path::FromEnv>::from_env()) }
                }
                EnvAttribute::Flat { .. } | EnvAttribute::None => {
                    quote! { #ident: None }
                }
            }
        });

        let requirements = self.get_fields().iter().map(|field| {
            let ty = field.option.as_ref().unwrap_or(&field.ty);

            match &field.env_attr {
                EnvAttribute::Nested => {
                    quote! {
                        <#ty as #private_path::FromEnv>::requirements(requirements);
                    }
                }
                EnvAttribute::Flat {
                    from,
                    default,
                    with: _,
                } => {
                    let from = from.value();
                    let default = default
                        .as_ref()
                        .map(|default| default.value())
                        .unwrap_or(String::new());
                    let out = format!("{from}={default}\n");

                    quote! {
                        requirements.push_str(#out);
                    }
                }
                _ => TokenStream::new(),
            }
        });

        quote! {
            impl #private_path::FromEnv for #struct_name {
                type FromEnvBuilder = #builder_name;

                fn from_env() -> Self::FromEnvBuilder {
                    #builder_name {
                        #(#fields,)*
                    }
                }

                fn requirements(requirements: &mut ::std::string::String) {
                    #(#requirements)*
                }
            }
        }
    }

    fn impl_from_env_builder(&self, consts: &ConstTokens) -> TokenStream {
        let struct_name = &self.ident;
        let private_path = &consts.private_path;
        let errors_ident = &consts.errors_ident;
        let builder_name = &consts.builder_name;

        // This is the secret sauce that lets us gather all errors before
        // failing.
        let assignments = self.get_fields().iter().map(|field| {
            let ident = &field.ident;
            let path = format!("{struct_name}.{ident}");

            match (&field.env_attr, field.option.is_some()) {
                // #[config(nested)] field: T,
                (EnvAttribute::Nested, false) => {
                    quote! {
                        let #ident = match #private_path::FromEnvBuilder::finalize(self.#ident.take().unwrap()) {
                            Ok(inner) => Ok(inner),
                            Err(errors) => {
                                #errors_ident.extend(errors);
                                Err(())
                            }
                        };
                    }
                }
                // #[config(nested)] field: Option<T>,
                (EnvAttribute::Nested, true) => {
                    quote! {
                        let #ident = match #private_path::FromEnvBuilder::finalize(self.#ident.take().unwrap()) {
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
                    EnvAttribute::Flat {
                        from,
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
                            match #with.parse_from_env(#from) {
                                Some((_, Ok(val))) => Ok(val),
                                Some((value, Err(error))) => {
                                    let err = #private_path::FromEnvError::ParseError {
                                        path: #path.to_string(),
                                        env_var: #from.to_string(),
                                        value,
                                        error,
                                    };
                                    #errors_ident.add(err);
                                    Err(())
                                }
                                None => {
                                    #with.parse(#default).map_err(|error| {
                                        let err = #private_path::FromEnvError::ParseError {
                                            path: #path.to_string(),
                                            env_var: #from.to_string(),
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
                // #[env(from = "...")] field: T
                (
                    EnvAttribute::Flat {
                        from,
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
                        match #with.parse_from_env(#from) {
                            Some((_, Ok(val))) => Ok(val),
                            Some((value, Err(error))) => {
                                let err = #private_path::FromEnvError::ParseError {
                                    path: #path.to_string(),
                                    env_var: #from.to_string(),
                                    value,
                                    error,
                                };
                                #errors_ident.add(err);
                                Err(())
                            }
                            None => {
                                let err = #private_path::FromEnvError::MissingEnv {
                                    path: #path.to_string(),
                                    env_var: #from.to_string(),
                                };
                                #errors_ident.add(err);
                                Err(())
                            }
                        }
                    };
                    }
                }
                // #[env(from = "...")] field: Option<T>
                (
                    EnvAttribute::Flat {
                        from,
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
                            match #with.parse_from_env(#from) {
                                Some((_, Ok(val))) => Ok(Some(val)),
                                Some((value, Err(error))) => {
                                    let err = #private_path::FromEnvError::ParseError {
                                        path: #path.to_string(),
                                        env_var: #from.to_string(),
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
                // #[env(default = "...")] field: Option<T>
                (
                    EnvAttribute::Flat {
                        from: _,
                        with: _,
                        default: Some(_),
                    },
                    true,
                ) => unreachable!("we've already checked that Optional fields can't have a default"),
                (EnvAttribute::None, false) => {
                    quote! {
                        let #ident = match self.#ident {
                            Some(inner) => Ok(inner),
                            None => {
                                let err = #private_path::FromEnvError::MissingValue {
                                    path: #path.to_string(),
                                };
                                #errors_ident.add(err);
                                Err(())
                            }
                        };
                    }
                }
                (EnvAttribute::None, true) => {
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
            impl #private_path::FromEnvBuilder for #builder_name {
                type Target = #struct_name;

                fn finalize(mut self) -> Result<Self::Target, #private_path::FromEnvErrors> {
                    let mut #errors_ident = #private_path::FromEnvErrors::new();

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
            let doc_attrs = &field.doc_attrs;

            match &field.env_attr {
                EnvAttribute::Nested => {
                    quote! {
                        #(#doc_attrs)*
                        pub fn #ident<F>(mut self, f: F) -> Self
                        where
                            F: FnOnce(<#ty as #private_path::FromEnv>::FromEnvBuilder) -> <#ty as #private_path::FromEnv>::FromEnvBuilder,
                        {
                            let nested = self.#ident.take().unwrap();
                            let nested = f(nested);
                            self.#ident = Some(nested);
                            self
                        }
                    }
                }
                EnvAttribute::Flat { .. } | EnvAttribute::None => {
                    quote! {
                        #(#doc_attrs)*
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

                pub fn finalize(self) -> Result<<Self as #private_path::FromEnvBuilder>::Target, #private_path::FromEnvErrors> {
                    #private_path::FromEnvBuilder::finalize(self)
                }
            }
        }
    }

    fn get_fields(&self) -> &Fields<FromEnvFieldReceiver> {
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
