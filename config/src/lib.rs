mod error;
mod parser;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub use config_derive::Config;
pub use error::{ConfigError, ConfigErrors};
pub use parser::ParserResult;

type BoxError = Box<dyn std::error::Error + 'static>;
