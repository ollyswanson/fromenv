pub use crate::error::{FromEnvError, FromEnvErrors};
pub use crate::parser::{Parser, from_str, into};

pub trait FromEnv {
    type FromEnvBuilder: FromEnvBuilder;

    fn from_env() -> Self::FromEnvBuilder;

    fn requirements(requirements: &mut String);
}

pub trait FromEnvBuilder {
    type Target;

    fn finalize(self) -> Result<Self::Target, FromEnvErrors>;
}
