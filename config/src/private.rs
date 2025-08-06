pub use crate::error::{ConfigError, ConfigErrors};
pub use crate::parser::{Parser, from_str, into};

pub trait Configurable {
    type ConfigBuilder: ConfigBuilder;

    fn configure() -> Self::ConfigBuilder;
}

pub trait ConfigBuilder {
    type Target;

    fn finalize(self) -> Result<Self::Target, ConfigErrors>;
}
