use std::{error::Error as StdError, fmt};

use super::BoxError;

#[derive(Debug)]
pub enum ConfigError {
    MissingEnv {
        env_var: String,
    },
    ParseError {
        env_var: String,
        value: String,
        error: BoxError,
    },
    MissingValue {
        field: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv { env_var } => {
                write!(f, "Missing required environment variable '{env_var}'")
            }
            Self::ParseError {
                env_var,
                value,
                error,
            } => {
                write!(
                    f,
                    "Failed to parse '{value}' from environment variable '{env_var}': {error}"
                )
            }
            Self::MissingValue { field } => {
                write!(f, "No value provided for '{field}'")
            }
        }
    }
}

impl StdError for ConfigError {}

#[derive(Debug, Default)]
pub struct ConfigErrors(Vec<ConfigError>);

impl ConfigErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: ConfigError) {
        self.0.push(error);
    }

    pub fn extend(&mut self, other: ConfigErrors) {
        self.0.extend(other.0);
    }

    pub fn has_errors(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn only_missing_errors(&self) -> bool {
        self.0.iter().all(|e| {
            matches!(
                e,
                ConfigError::MissingEnv { .. } | ConfigError::MissingValue { .. }
            )
        })
    }
}

impl fmt::Display for ConfigErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} configuration errors:", self.0.len())?;

        for (i, error) in self.0.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, error)?;
        }
        Ok(())
    }
}

impl StdError for ConfigErrors {}
