use std::{error::Error as StdError, fmt};

use super::BoxError;

#[derive(Debug)]
pub enum FromEnvError {
    MissingEnv {
        path: String,
        env_var: String,
    },
    ParseError {
        path: String,
        env_var: String,
        value: String,
        error: BoxError,
    },
    MissingValue {
        path: String,
    },
}

impl fmt::Display for FromEnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv { path, env_var } => {
                write!(
                    f,
                    "`{path}`: Missing required environment variable '{env_var}'"
                )
            }
            Self::ParseError {
                path,
                env_var,
                value,
                error,
            } => {
                write!(
                    f,
                    "`{path}`: Failed to parse '{env_var}'=\"{value}\": {error}"
                )
            }
            Self::MissingValue { path } => {
                write!(f, "`{path}`: No value provided")
            }
        }
    }
}

impl StdError for FromEnvError {}

/// A collection of configuration errors encountered during environment variable
/// loading.
///
/// This error type is returned from the `finalize()` method when configuration
/// loading fails. It collects all errors rather than failing on the first one,
/// making it easier to fix multiple issues at once.
///
/// # Display Format
///
/// When displayed, this error shows a count and list of all issues:
///
/// ```text
/// 2 configuration errors:
///   1. `Config.database_url`: Missing required environment variable 'DATABASE_URL'
///   2. `Config.port`: Failed to parse 'PORT'="invalid": invalid digit found in string
/// ```
#[derive(Debug, Default)]
pub struct FromEnvErrors(Vec<FromEnvError>);

impl FromEnvErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: FromEnvError) {
        self.0.push(error);
    }

    pub fn extend(&mut self, other: FromEnvErrors) {
        self.0.extend(other.0);
    }

    pub fn has_errors(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn only_missing_errors(&self) -> bool {
        self.0.iter().all(|e| {
            matches!(
                e,
                FromEnvError::MissingEnv { .. } | FromEnvError::MissingValue { .. }
            )
        })
    }
}

impl fmt::Display for FromEnvErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.len() {
            1 => writeln!(f, "1 configuration error:")?,
            n => writeln!(f, "{n} configuration errors:")?,
        }

        for (i, error) in self.0.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, error)?;
        }
        Ok(())
    }
}

impl StdError for FromEnvErrors {}
