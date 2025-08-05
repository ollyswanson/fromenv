pub use crate::error::{ConfigError, ConfigErrors};

use super::BoxError;

pub trait Configurable {
    type ConfigBuilder: ConfigBuilder;

    fn configure() -> Self::ConfigBuilder;
}

pub trait ConfigBuilder {
    type Target;

    fn finalize(self) -> Result<Self::Target, ConfigErrors>;
}

pub fn try_parse_env<T>(env_var: &str) -> Option<(String, Result<T, BoxError>)>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    std::env::var(env_var).ok().map(|value| {
        let parse_result = value.parse::<T>().map_err(|e| e.into());
        (value, parse_result)
    })
}
