mod converter;
mod error;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub use config_derive::Config;
pub use error::{ConfigError, ConfigErrors};

type BoxError = Box<dyn std::error::Error + 'static>;
