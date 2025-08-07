mod error;
mod parser;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub use error::{FromEnvError, FromEnvErrors};
pub use fromenv_derive::FromEnv;
pub use parser::ParserResult;

type BoxError = Box<dyn std::error::Error + 'static>;
