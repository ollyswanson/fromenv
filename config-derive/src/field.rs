use syn::{ExprPath, Field, Ident, LitStr, Meta, Type};

use crate::helpers::parse_option;

pub struct FieldRepr {
    pub ident: Ident,
    pub ty: Type,
    pub option: Option<Type>,
    pub field_type: FieldType,
}

pub enum FieldType {
    /// #[config(nested)]
    Nested,
    /// #[config(env = "...")] OR #[config]
    ConfigValue {
        env: LitStr,
        default: Option<LitStr>,
        with: Option<ExprPath>,
    },
    /// No config attr.
    Standard,
}

impl FieldRepr {
    pub fn parse(field: Field) -> syn::Result<FieldRepr> {
        let ident = field.ident.as_ref().map(|f| f.to_owned()).ok_or_else(|| {
            syn::Error::new_spanned(
                field.to_owned(),
                "Config derive only supports structs with named fields",
            )
        })?;
        let ty = field.ty.to_owned();
        let option = parse_option(&ty).map(ToOwned::to_owned);

        struct Attrs {
            env: Option<LitStr>,
            default: Option<LitStr>,
            with: Option<ExprPath>,
            nested: bool,
        }

        let mut has_config_attr = false;
        let mut attrs = Attrs {
            env: None,
            default: None,
            with: None,
            nested: false,
        };

        for attr in &field.attrs {
            if attr.path().is_ident("config") {
                has_config_attr = true;

                if matches!(attr.meta, Meta::Path(_)) {
                    continue;
                }

                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("env") {
                        let value = meta.value()?;
                        let env: LitStr = value.parse()?;
                        attrs.env = Some(env);
                    } else if meta.path.is_ident("default") {
                        let value = meta.value()?;
                        let default: LitStr = value.parse()?;
                        attrs.default = Some(default);
                    } else if meta.path.is_ident("with") {
                        let value = meta.value()?;
                        let with = value.parse()?;
                        attrs.with = Some(with);
                    } else if meta.path.is_ident("nested") {
                        if meta.input.is_empty() {
                            attrs.nested = true;
                        } else {
                            return Err(meta.error("expected `nested` without a value"));
                        }
                    } else {
                        return Err(meta.error(format!(
                            "unsupported config attribute: {}",
                            meta.path.get_ident().unwrap()
                        )));
                    }

                    Ok(())
                })?;
            }
        }

        if !has_config_attr {
            let field_repr = FieldRepr {
                ident,
                ty,
                option,
                field_type: FieldType::Standard,
            };

            return Ok(field_repr);
        }

        if option.is_some() && attrs.default.is_some() {
            return Err(syn::Error::new_spanned(
                field.to_owned(),
                "Optional fields cannot have a default",
            ));
        }

        let field_type = match attrs {
            Attrs {
                env: Some(_),
                default: _,
                with: _,
                nested: true,
            }
            | Attrs {
                env: _,
                default: Some(_),
                with: _,
                nested: true,
            }
            | Attrs {
                env: _,
                default: _,
                with: Some(_),
                nested: true,
            } => {
                return Err(syn::Error::new_spanned(
                    field.to_owned(),
                    "nested must not be used with other attributes",
                ));
            }
            Attrs {
                env: _,
                default: _,
                with: _,
                nested: true,
            } => FieldType::Nested,
            Attrs {
                env,
                default,
                with,
                nested: false,
            } => FieldType::ConfigValue {
                env: env.unwrap_or_else(|| {
                    LitStr::new(&ident.to_string().to_uppercase(), ident.span())
                }),
                default,
                with,
            },
        };

        let field_repr = FieldRepr {
            ident,
            ty,
            option,
            field_type,
        };

        Ok(field_repr)
    }
}
