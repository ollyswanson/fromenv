mod field;
mod generate;
mod helpers;

use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Fields, FieldsNamed, parse_macro_input};

use crate::{field::FieldRepr, generate::CodeGenerator};

#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_derive(input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = input.ident.to_owned();

    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            struct_name,
            "Config derive only supports structs",
        ));
    };

    let Fields::Named(FieldsNamed { named: fields, .. }) = data.fields else {
        return Err(syn::Error::new_spanned(
            struct_name,
            "Config derive only supports structs with named fields",
        ));
    };

    let fields = fields
        .into_iter()
        .map(FieldRepr::parse)
        .collect::<syn::Result<Vec<_>>>()?;

    let out = CodeGenerator::new(&struct_name).generate(&fields);
    Ok(out)
}
