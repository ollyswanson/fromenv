use config::Config;

#[test]
fn env_variable_parsing() {
    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env = "DATABASE_URL")]
        db_url: String,
    }

    let expected = Config {
        db_url: "postgres://postgres@postgres/postgres".to_owned(),
    };

    let actual = temp_env::with_var(
        "DATABASE_URL",
        Some("postgres://postgres@postgres/postgres"),
        || Config::configure().finalize(),
    )
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn default_value_fallback() {
    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env = "APP_PORT", default = "8080")]
        port: u16,
    }

    let expected = Config { port: 8080 };
    let actual = Config::configure().finalize().unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn invalid_value_returns_error_despite_default() {
    #[derive(Config)]
    #[allow(unused)]
    pub struct Config {
        #[config(env = "APP_PORT", default = "8080")]
        port: u16,
    }

    let result = temp_env::with_var("APP_PORT", Some("not a u16"), || {
        Config::configure().finalize()
    });

    assert!(result.is_err());
}

#[test]
fn implicit_env_name_uses_uppercase_field() {
    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env)]
        port: u16,
    }

    let expected = Config { port: 8080 };

    let actual =
        temp_env::with_var("PORT", Some("8080"), || Config::configure().finalize()).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn builder_overrides_env_values() {
    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env)]
        port: u16,
    }

    let expected = Config { port: 56781 };

    let actual = temp_env::with_var("PORT", Some("8080"), || {
        Config::configure().port(56781).finalize()
    })
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn builder_bypasses_invalid_env_values() {
    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env)]
        port: u16,
    }

    let expected = Config { port: 56781 };

    let actual = temp_env::with_var("PORT", Some("not-a-u16"), || {
        Config::configure().port(56781).finalize()
    })
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn optional_fields() {
    use config::Config;

    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env = "OTEL_RESOURCE_ATTRIBUTES")]
        resource_attributes: Option<String>,
        #[config(env = "OTEL_LOG_LEVEL", default = "info", with = into)]
        log_level: String,
    }

    let expected = Config {
        resource_attributes: None,
        log_level: "info".into(),
    };

    let actual = Config::configure().finalize().unwrap();

    assert_eq!(expected, actual);
}
