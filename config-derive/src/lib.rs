mod generate;
mod helpers;
mod parser;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use syn::{DeriveInput, Visibility, parse_macro_input};

use crate::{generate::CodeGenerator, parser::ConfigReceiver};

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

    let mut accumulator = darling::Error::accumulator();

    let struct_name = config_struct.ident;

    if !matches!(config_struct.vis, Visibility::Public(_)) {
        accumulator.push(
            darling::Error::custom("Config derive requires a public struct")
                .with_span(&struct_name.span()),
        );
    }
    accumulator.finish()?;

    let fields = config_struct
        .data
        .take_struct()
        .expect("to have validated that it is a struct")
        .fields;

    let out = CodeGenerator::new(&struct_name).generate(&fields);
    Ok(out)
}
