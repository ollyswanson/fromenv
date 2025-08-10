#[test]
fn captures_basic_errors() {
    use fromenv::FromEnv;

    #[derive(FromEnv, Debug)]
    #[allow(unused)]
    pub struct Config {
        #[env(from)]
        first: String,
        #[env(from)]
        second: String,
        #[env(from)]
        third: String,
    }

    let expected = r#"3 configuration errors:
  1. `Config.first`: Missing required environment variable 'FIRST'
  2. `Config.second`: Missing required environment variable 'SECOND'
  3. `Config.third`: Missing required environment variable 'THIRD'
"#;
    let actual = Config::from_env().finalize().unwrap_err().to_string();

    assert_eq!(expected, actual);
}

#[test]
fn captures_nested_errors() {
    use foo::bar::AppConfig;

    // simulate module structure
    mod foo {
        pub mod bar {
            use super::baz::KafkaConfig;
            use fromenv::FromEnv;

            #[derive(FromEnv, Debug)]
            #[allow(unused)]
            pub struct AppConfig {
                #[env(from)]
                pub database_url: String,
                #[env(nested)]
                pub kafka: KafkaConfig,
            }
        }

        pub mod baz {
            use fromenv::FromEnv;

            #[derive(FromEnv, Debug)]
            #[allow(unused)]
            pub struct KafkaConfig {
                #[env(from = "KAFKA_BROKER")]
                pub broker: String,
            }
        }
    }

    let expected = r#"2 configuration errors:
  1. `AppConfig.database_url`: Missing required environment variable 'DATABASE_URL'
  2. `KafkaConfig.broker`: Missing required environment variable 'KAFKA_BROKER'
"#;
    let actual = AppConfig::from_env().finalize().unwrap_err().to_string();

    assert_eq!(expected, actual);
}
