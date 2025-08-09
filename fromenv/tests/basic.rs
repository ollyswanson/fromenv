use fromenv::FromEnv;

#[test]
fn env_variable_parsing() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from = "DATABASE_URL")]
        db_url: String,
    }

    let expected = Config {
        db_url: "postgres://postgres@postgres/postgres".to_owned(),
    };

    let actual = temp_env::with_var(
        "DATABASE_URL",
        Some("postgres://postgres@postgres/postgres"),
        || Config::from_env().finalize(),
    )
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn default_value_fallback() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from = "APP_PORT", default = "8080")]
        port: u16,
    }

    let expected = Config { port: 8080 };
    let actual = Config::from_env().finalize().unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn invalid_value_returns_error_despite_default() {
    #[derive(FromEnv)]
    #[allow(unused)]
    pub struct Config {
        #[env(from = "APP_PORT", default = "8080")]
        port: u16,
    }

    let result = temp_env::with_var("APP_PORT", Some("not a u16"), || {
        Config::from_env().finalize()
    });

    assert!(result.is_err());
}

#[test]
fn implicit_env_name_uses_uppercase_field() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from)]
        port: u16,
    }

    let expected = Config { port: 8080 };

    let actual =
        temp_env::with_var("PORT", Some("8080"), || Config::from_env().finalize()).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn builder_overrides_env_values() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from)]
        port: u16,
    }

    let expected = Config { port: 56781 };

    let actual = temp_env::with_var("PORT", Some("8080"), || {
        Config::from_env().port(56781).finalize()
    })
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn builder_bypasses_invalid_env_values() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from)]
        port: u16,
    }

    let expected = Config { port: 56781 };

    let actual = temp_env::with_var("PORT", Some("not-a-u16"), || {
        Config::from_env().port(56781).finalize()
    })
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn optional_fields() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from = "OTEL_RESOURCE_ATTRIBUTES")]
        resource_attributes: Option<String>,
        #[env(from = "OTEL_LOG_LEVEL", default = "info", with = into)]
        log_level: String,
    }

    let expected = Config {
        resource_attributes: None,
        log_level: "info".into(),
    };

    let actual = Config::from_env().finalize().unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn requirements() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from = "OTEL_RESOURCE_ATTRIBUTES")]
        resource_attributes: Option<String>,
        #[env(from = "OTEL_LOG_LEVEL", default = "info", with = into)]
        log_level: String,
    }

    let actual = Config::requirements();
    let expected = "\
        OTEL_RESOURCE_ATTRIBUTES=\n\
        OTEL_LOG_LEVEL=info\n\
    ";
    assert_eq!(expected, actual);
}
