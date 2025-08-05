use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ExprPath, Ident};

use super::field::{FieldRepr, FieldType};

pub struct CodeGenerator {
    struct_name: Ident,
    private_path: TokenStream,
    errors_ident: TokenStream,
}

impl CodeGenerator {
    pub fn new(struct_name: &Ident) -> Self {
        Self {
            struct_name: struct_name.to_owned(),
            private_path: quote!(__config::__private),
            errors_ident: quote!(__config_derive_builder_errors),
        }
    }

    pub fn generate(&self, fields: &[FieldRepr]) -> TokenStream {
        let struct_name = &self.struct_name;
        let builder_name = format_ident!("{}Builder", struct_name);
        let private_path = &self.private_path;
        let errors_ident = &self.errors_ident;

        let builder_field_definitions = fields
            .iter()
            .map(|field| self.builder_field_definition(field));
        let builder_field_inits = fields.iter().map(|field| self.builder_field_init(field));
        let builder_field_gather_errors = fields
            .iter()
            .map(|field| self.builder_field_gather_error(field));
        let builder_field_returns = fields.iter().map(|field| self.builder_field_return(field));
        let builder_field_setters = fields.iter().map(|field| self.builder_field_setter(field));

        quote! {
            const _: () = {
                extern crate config as __config;
                use #private_path::Converter as _;

                pub struct #builder_name {
                    #(#builder_field_definitions,)*
                }

                impl #struct_name {
                    pub fn configure() -> #builder_name {
                       <Self as #private_path::Configurable>::configure()
                    }
                }

                impl #private_path::Configurable for #struct_name {
                    type ConfigBuilder = #builder_name;

                    fn configure() -> Self::ConfigBuilder {
                        #builder_name {
                            #(#builder_field_inits,)*
                        }
                    }
                }

                impl #private_path::ConfigBuilder for #builder_name {
                    type Target = #struct_name;

                    fn finalize(mut self) -> Result<Self::Target, #private_path::ConfigErrors> {
                        let mut #errors_ident = #private_path::ConfigErrors::new();

                        #(#builder_field_gather_errors)*

                        Ok(#struct_name {
                            #(#builder_field_returns,)*
                        })
                    }
                }

                impl #builder_name {
                    #(#builder_field_setters)*

                    pub fn finalize(self) -> Result<<Self as #private_path::ConfigBuilder>::Target, #private_path::ConfigErrors> {
                        #private_path::ConfigBuilder::finalize(self)
                    }
                }
            };
        }
    }

    fn builder_field_definition(&self, field: &FieldRepr) -> TokenStream {
        let ident = &field.ident;
        let ty = field.option.as_ref().unwrap_or(&field.ty);
        let private_path = &self.private_path;

        match &field.field_type {
            FieldType::Nested => {
                quote! { #ident: Option<<#ty as #private_path::Configurable>::ConfigBuilder> }
            }
            FieldType::ConfigValue { .. } | FieldType::Standard => {
                quote! { #ident: Option<#ty> }
            }
        }
    }

    fn builder_field_init(&self, field: &FieldRepr) -> TokenStream {
        let ident = &field.ident;
        let ty = field.option.as_ref().unwrap_or(&field.ty);
        let private_path = &self.private_path;

        match &field.field_type {
            FieldType::Nested => {
                quote! { #ident: Some(<#ty as #private_path::Configurable>::configure()) }
            }
            FieldType::ConfigValue { .. } | FieldType::Standard => {
                quote! { #ident: None }
            }
        }
    }

    fn builder_field_gather_error(&self, field: &FieldRepr) -> TokenStream {
        let ident = &field.ident;
        let private_path = &self.private_path;
        let errors_ident = &self.errors_ident;

        match (&field.field_type, field.option.is_some()) {
            // #[config(nested)] field: T,
            (FieldType::Nested, false) => {
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
            (FieldType::Nested, true) => {
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
                FieldType::ConfigValue {
                    env,
                    with,
                    default: Some(default),
                },
                false,
            ) => {
                let with = self.converter_path(with.as_ref());

                quote! {
                    let #ident = if let Some(inner) = self.#ident {
                        Ok(inner)
                    } else {
                        match #with.convert_from_env(#env) {
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
                                #with.convert(#default).map_err(|error| {
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
                FieldType::ConfigValue {
                    env,
                    with,
                    default: None,
                },
                false,
            ) => {
                let with = self.converter_path(with.as_ref());

                quote! {
                   let #ident = if let Some(inner) = self.#ident {
                       Ok(inner)
                   } else {
                       match #with.convert_from_env(#env) {
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
                FieldType::ConfigValue {
                    env,
                    with,
                    default: None,
                },
                true,
            ) => {
                let with = self.converter_path(with.as_ref());

                quote! {
                    let #ident = if let Some(inner) = self.#ident {
                        Ok(Some(inner))
                    } else {
                        match #with.convert_from_env(#env) {
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
                FieldType::ConfigValue {
                    env: _,
                    with: _,
                    default: Some(_),
                },
                true,
            ) => unreachable!("we've already checked that Optional fields can't have a default"),
            (FieldType::Standard, false) => {
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
            (FieldType::Standard, true) => {
                quote! {
                    let #ident = self.#ident;
                }
            }
        }
    }

    fn builder_field_return(&self, field: &FieldRepr) -> TokenStream {
        let ident = &field.ident;
        let errors_ident = &self.errors_ident;

        quote! {
            #ident: match #ident {
                Ok(val) => val,
                Err(_) => {
                    return Err(#errors_ident);
                }
            }
        }
    }

    fn builder_field_setter(&self, field: &FieldRepr) -> TokenStream {
        let ident = &field.ident;
        let ty = field.option.as_ref().unwrap_or(&field.ty);
        let private_path = &self.private_path;

        match &field.field_type {
            FieldType::Nested => {
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
            FieldType::ConfigValue { .. } | FieldType::Standard => {
                quote! {
                    pub fn #ident(mut self, #ident: #ty) -> Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }
            }
        }
    }

    fn converter_path(&self, path: Option<&ExprPath>) -> TokenStream {
        let ident = path
            .and_then(|path| path.path.get_ident())
            .map(|ident| ident.to_string())
            .unwrap_or_else(|| "from_str".into());

        let private_path = &self.private_path;

        match ident.as_str() {
            "from_str" => quote!(#private_path::from_str),
            "into" => quote!(#private_path::into),
            _ => quote!(#path),
        }
    }
}
