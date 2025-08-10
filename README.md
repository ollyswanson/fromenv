# FromEnv

<!-- cargo-rdme start -->

A declarative, type-safe library for loading configuration from environment
variables.

## Features

* A simple derive macro, `FromEnv`, that handles environment variable to
struct mapping.
* Composition of configuration structs using the `#[env(nested)]` attribute.
* Default values using the `#[env(from, default = "...")]` attribute.
* Custom parsers using the `#[env(from, with = my_parser)]` attribute.
* Comprehensive error reporting that collects all configuration issues
before failing.
* A type-safe builder pattern for overriding configuration values.
* Opinionated error handling, parsing errors aren't silently ignored when
defaults are provided.
* Documentation of configuration options using the `requirements` method.

## Usage

### Basic example

```rust
use fromenv::FromEnv;

#[derive(FromEnv, Debug)]
pub struct Config {
    #[env(from = "DATABASE_URL")]
    database_url: String,

    #[env(from = "PORT", default = "8080")]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env().finalize()?;

    println!("Config: {:?}", config);

    println!("Documentation:\n{}", Config::requirements());

    Ok(())
}
```

### Nested Configuration

```rust
use fromenv::FromEnv;

#[derive(FromEnv, Debug)]
pub struct DatabaseConfig {
    #[env(from = "DB_HOST", default = "localhost")]
    host: String,

    #[env(from = "DB_PORT", default = "5432")]
    port: u16,
}

#[derive(FromEnv, Debug)]
pub struct Config {
    #[env(nested)]
    database: DatabaseConfig,

    #[env(from = "APP_NAME")]
    name: String,
}
```

### Custom parsers

By default, `FromEnv` will use the types `FromStr` implementation. This can
be specified explicitly using `#[env(from, with = fromstr)]`.

You can also specify `#[env(from, with = into)]` which can be useful for
types, such as [secrecy's](https://crates.io/crates/secrecy)
[`SecretString`](https://docs.rs/secrecy/0.10.3/secrecy/type.SecretString.html)
type which can't be constructed using `FromStr` but can using `Into`.

In addition to these built in parsers, you can supply a path to your own
parser function.


```rust
use fromenv::{FromEnv, ParseResult};
use secrecy::SecretString;

// Any function with the signature `fn<T>(&str) -> Result<T, Box<dyn StdError>>`
// can be used as a custom parser.
fn comma_separated(s: &str) -> ParseResult<Vec<String>> {
    Ok(s.split(',').map(ToOwned::to_owned).collect())
}

#[derive(FromEnv, Debug)]
pub struct KafkaConfig {
    #[env(from = "KAFKA_BOOTSTRAP_SERVERS", with = comma_separated)]
    bootstrap_servers: Vec<String>,
}

#[derive(FromEnv, Debug)]
pub struct Config {
    #[env(from = "API_KEY", with = into)]
    api_key: SecretString,

    #[env(nested)]
    kafka: KafkaConfig,
}
```

### Optional Fields

Both "flat" and "nested" fields can be made optional. When making a field optional:

* A default _cannot_ be specified.
* Parse errors will _not_ be ignored.
* When used on a `nested` configuration, the field will be set to `None` if
and only if all of the errors returned from attempting to parse it are due
to missing env vars or values.

```rust
use fromenv::FromEnv;

#[derive(FromEnv, Debug)]
pub struct Config {
    #[env(from = "OTEL_RESOURCE_ATTRIBUTES")]
    resource_attributes: Option<String>,
}
```

### Builder Overrides

In tests especially, it can be frustrating to want to override a portion of the
configuration but read the rest from environment variables, only to find
yourself having to explictly set every field in the config.

It is possible to override portions of the configuration before calling
`finalize`.

Overriding has a higher precedence than reading environment variables, and any
fields which have been overridden will skip reading from the environment,
avoiding any errors that might arise from missing environment variables.

```rust
use fromenv::FromEnv;

#[derive(FromEnv, Debug)]
pub struct TelemetryConfig {
    #[env(from = "RUST_LOG")]
    log_level: String,
}

#[derive(FromEnv, Debug)]
pub struct Config {
    #[env(from)]
    database_url: String,

    #[env(from, default = "8080")]
    port: u16,

    #[env(nested)]
    telemetry: TelemetryConfig,
}

#[test]
fn test() {
    // Override `port` and `telemetry.log_level` but read `database_url` from
    // the environment.
    let config = Config::from_env()
        .port(0)
        .telemetry(|telemetry| telemetry.log_level("debug".into()))
        .finalize()
        .unwrap();

    // Rest of the test...
}
```

## Attribute Options

* `#[env(from = "ENV_NAME")]` - Load from specified environment variable.
* `#[env(from)]` - Load from environment variable matching field's uppercase
name.
* `#[env(from, default = "value")]` - Default value if environment variable
is not set.
* `#[env(from, with = parser_fn)]` - Custom parser function.
* `#[env(nested)]` - For nested configuration structures.
* It is possible skip the `env` attribute for a field, but to avoid any
errors the value must be set using the override methods before calling
`finalize`.

## Error Handling

The `finalize()` method returns a `Result<T, FromEnvErrors>` where
`FromEnvErrors` contains the accumulated configuration errors.

This produces error messages of the form:

```text
2 configuration errors:
    1. `Config.database_url`: Missing required environment variable 'DATABASE_URL'
    2. `Config.port`: Failed to parse 'PORT'="invalid": invalid digit found in string
```

<!-- cargo-rdme end -->

## Alternatives / Motivation

This crate is designed to scratch a very specific itch.

1. All of the configuration is handled by environment variables.
2. The struct hierarchy shouldn't matter, no magic inference of environment
   variable names through layers of nested structs - I want to be able to
   compose configuration defined by a number of separate crates without having
   to worry about the effect that the structure will have on what the
   environment variables need to be named, and vice versa.
3. The ability to override _parts_ of the configuration whilst still reading
   most of the configuration from the environment - not throwing the baby out
   with the bath water when parts of the configuration need to be set
   programmatically, such as when writing tests.
4. The ability to document all of the environment variables that an application
   might make use of as a single string and not as scattered Rust docs. I'm
   using this with `cargo insta` to both document the env vars required for my
   applications, and to prevent unintended breakage in my configuration.

If you don't have the same itch, or would rather find other workarounds, or
(sensibly) want to use what everyone else is using, use:

* [config](https://crates.io/crates/config) - the de facto crate for configuring
  applications.
* [envconfig](https://crates.io/crates/envconfig) - the primary inspiration for
  _this_ crate. I just wanted a few more frills, otherwise I'd be using
  envconfig as well.
* [envstruct](https://crates.io/crates/envstruct) - I came across this crate
  while hunting for a crate name that hadn't already been taken. It has a _very_
  similar API, and takes a different but nice approach.

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
