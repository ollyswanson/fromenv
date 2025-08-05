use syn::{GenericArgument, PathArguments, Type};

pub fn parse_option(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let Some(segment) = type_path.path.segments.last() else {
        return None;
    };

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
