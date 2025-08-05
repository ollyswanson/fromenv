pub use crate::converter::{Converter, from_str, into};
pub use crate::error::{ConfigError, ConfigErrors};

pub trait Configurable {
    type ConfigBuilder: ConfigBuilder;

    fn configure() -> Self::ConfigBuilder;
}

pub trait ConfigBuilder {
    type Target;

    fn finalize(self) -> Result<Self::Target, ConfigErrors>;
}
