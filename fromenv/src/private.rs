//! Internal traits and types used by the derive macro.
//!
//! There are not meant to be used directly by users of the library.
//!
//! The traits exist to make the `nested` attribute possible, by making use of
//! `Ty as Trait` syntax we can both assert that the nested struct has derived
//! `FromEnv` and gain access to its associated builder type without having to
//! name it explicitly.
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
