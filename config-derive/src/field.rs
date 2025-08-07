use darling::{
    ast::NestedMeta,
    util::{Flag, Override},
    FromField, FromMeta,
};
use proc_macro2::Span;
use syn::{spanned::Spanned, ExprPath, GenericArgument, Ident, LitStr, PathArguments, Type};

#[derive(Debug)]
pub struct ConfigFieldReceiver {
    pub ident: Ident,
    pub ty: Type,
    pub option: Option<Type>,
    pub config_attr: ConfigAttribute,
}

#[derive(Debug)]
pub enum ConfigAttribute {
    /// #[config(env = "...")] OR #[config]
    Flat {
        env: LitStr,
        default: Option<LitStr>,
        with: Option<ExprPath>,
    },
    /// #[config(nested)]
    Nested,
    /// No config attr.
    None,
}

impl FromField for ConfigFieldReceiver {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let mut accumulator = darling::Error::accumulator();
        let ident = field
            .ident
            .as_ref()
            .map(|f| f.to_owned())
            .expect("asserted the shape of the struct already");

        let ty = field.ty.to_owned();
        let option = parse_option(&ty).map(ToOwned::to_owned);

        let mut env: Option<Override<LitStr>> = None;
        let mut default: Option<LitStr> = None;
        let mut with: Option<ExprPath> = None;
        let mut nested = Flag::default();

        let mut default_path_span = Span::call_site();
        let mut with_path_span = Span::call_site();

        for attr in &field.attrs {
            if attr.path().is_ident("config") {
                let meta_list = match attr.meta.require_list() {
                    Ok(list) => list,
                    Err(e) => {
                        accumulator.push(e.into());
                        continue;
                    }
                };

                let nested_meta_list = match NestedMeta::parse_meta_list(meta_list.tokens.clone()) {
                    Ok(nested) => nested,
                    Err(e) => {
                        accumulator.push(e.into());
                        continue;
                    }
                };

                for meta in nested_meta_list {
                    let meta = match meta {
                        NestedMeta::Meta(meta) => meta,
                        NestedMeta::Lit(lit) => {
                            accumulator.push(darling::Error::unexpected_lit_type(&lit));
                            continue;
                        }
                    };

                    if meta.path().is_ident("env") {
                        match FromMeta::from_meta(&meta) {
                            Ok(v) => env = Some(v),
                            Err(e) => {
                                accumulator.push(e);
                            }
                        }
                    } else if meta.path().is_ident("default") {
                        default_path_span = meta.path().span();
                        match FromMeta::from_meta(&meta) {
                            Ok(v) => default = Some(v),
                            Err(e) => {
                                accumulator.push(e);
                            }
                        }
                    } else if meta.path().is_ident("with") {
                        with_path_span = meta.path().span();
                        match FromMeta::from_meta(&meta) {
                            Ok(v) => with = Some(v),
                            Err(e) => {
                                accumulator.push(e);
                            }
                        }
                    } else if meta.path().is_ident("nested") {
                        match FromMeta::from_meta(&meta) {
                            Ok(v) => nested = v,
                            Err(e) => {
                                accumulator.push(e);
                            }
                        }
                    } else {
                        accumulator.push(darling::Error::unknown_field_path(meta.path()));
                    }
                }
            }
        }

        const NESTED_CLASH: &str = "`nested` cannot be used with other attributes";
        const DEFAULT_WITHOUT_ENV: &str = "`default` cannot be used without `env`";
        const WITH_WITHOUT_ENV: &str = "`with` cannot be used without `env`";
        const OPTION_WIH_DEFAULT: &str = "Optional fields cannot have a default";

        match (env, default, with, nested.is_present()) {
            (Some(_), _, _, true) | (_, Some(_), _, true) | (_, _, Some(_), true) => {
                accumulator.push(darling::Error::custom(NESTED_CLASH).with_span(&nested.span()));

                Err(accumulator.finish().unwrap_err())
            }
            (None, Some(_), None, false) => {
                accumulator.push(
                    darling::Error::custom(DEFAULT_WITHOUT_ENV).with_span(&default_path_span),
                );

                Err(accumulator.finish().unwrap_err())
            }
            (None, None, Some(_), false) => {
                accumulator
                    .push(darling::Error::custom(WITH_WITHOUT_ENV).with_span(&with_path_span));

                Err(accumulator.finish().unwrap_err())
            }
            (None, Some(_), Some(_), false) => {
                accumulator.push(
                    darling::Error::custom(DEFAULT_WITHOUT_ENV).with_span(&default_path_span),
                );
                accumulator
                    .push(darling::Error::custom(WITH_WITHOUT_ENV).with_span(&with_path_span));

                Err(accumulator.finish().unwrap_err())
            }
            (None, None, None, false) => accumulator.finish_with(Self {
                ident,
                ty,
                option,
                config_attr: ConfigAttribute::None,
            }),
            (None, None, None, true) => accumulator.finish_with(Self {
                ident,
                ty,
                option,
                config_attr: ConfigAttribute::Nested,
            }),
            (Some(env), default, with, false) => {
                if option.is_some() && default.is_some() {
                    let err =
                        darling::Error::custom(OPTION_WIH_DEFAULT).with_span(&default_path_span);

                    accumulator.push(err);
                }

                let env = env.unwrap_or_else(|| {
                    LitStr::new(&ident.to_string().to_uppercase(), ident.span())
                });

                accumulator.finish_with(Self {
                    ident,
                    ty,
                    option,
                    config_attr: ConfigAttribute::Flat { env, default, with },
                })
            }
        }
    }
}

fn parse_option(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;

    let generic_args = if segment.ident == "Option" {
        let PathArguments::AngleBracketed(generic_args) = &segment.arguments else {
            return None;
        };
        generic_args
    } else {
        return None;
    };

    if generic_args.args.len() == 1 {
        if let GenericArgument::Type(inner_type) = &generic_args.args[0] {
            return Some(inner_type);
        }
    }

    None
}
