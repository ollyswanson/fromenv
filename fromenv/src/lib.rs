#![doc = include_str!("../../README.md")]

mod error;
mod parser;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub use error::{FromEnvError, FromEnvErrors};

/// Derive macro for loading configuration from environment variables.
///
/// ## Attribute Options
///
/// * `#[env(from = "ENV_NAME")]` - Load from specified environment variable.
/// * `#[env(from)]` - Load from environment variable matching field's uppercase name.
/// * `#[env(from, default = "value")]` - Default value if environment variable is not set.
/// * `#[env(from, with = parser_fn)]` - Custom parser function.
/// * `#[env(nested)]` - For nested configuration structures.
/// * It is possible skip the `env` attribute for a field, but to avoid any
///   errors the value must be set using the override methods before calling
///   `finalize`.
///
/// ## Examples
///
/// ```rust,no_run
/// # fn main() {
///
/// use fromenv::FromEnv;
///
/// #[derive(FromEnv, Debug)]
/// pub struct Config {
///     #[env(from = "DATABASE_URL", default = "postgres://localhost/mydb")]
///     database_url: String,
///
///     #[env(from, default = "8080")]
///     port: u16,
/// }
///
/// // Load configuration
/// let config = Config::from_env().finalize().expect("Invalid configuration");
///
/// // Or provide some values directly
/// let config = Config::from_env()
///     .database_url("postgres://remote/db".into())
///     .finalize()
///     .expect("Invalid configuration");
/// # }
/// ```
pub use fromenv_derive::FromEnv;
pub use parser::ParseResult;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
