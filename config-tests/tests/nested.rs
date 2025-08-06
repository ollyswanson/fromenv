#[test]
fn nested_config_structs() {
    // Simulate composing config from different modules
    use foo::{bar::KafkaConfig, baz::AppConfig};

    mod foo {
        pub mod bar {
            use config::Config;

            #[derive(Config, Debug, PartialEq)]
            pub struct KafkaConfig {
                #[config(env = "KAFKA_BROKERS")]
                pub brokers: String,
            }
        }

        pub mod baz {
            use config::Config;

            #[derive(Config, Debug, PartialEq)]
            pub struct AppConfig {
                #[config(env)]
                pub database_url: String,
                #[config(nested)]
                pub kafka: super::bar::KafkaConfig,
            }
        }
    }

    let expected = AppConfig {
        database_url: "postgres://postgres@postgres/postgres".into(),
        kafka: KafkaConfig {
            brokers: "kafka:29092".into(),
        },
    };

    let actual = temp_env::with_vars(
        [
            (
                "DATABASE_URL",
                Some("postgres://postgres@postgres/postgres"),
            ),
            ("KAFKA_BROKERS", Some("kafka:29092")),
        ],
        || AppConfig::configure().finalize(),
    )
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn nested_builder_methods() {
    // Simulate composing config from different modules
    use foo::{bar::KafkaConfig, baz::AppConfig};

    mod foo {
        pub mod bar {
            use config::Config;

            #[derive(Config, Debug, PartialEq)]
            pub struct KafkaConfig {
                #[config(env = "KAFKA_BROKERS")]
                pub brokers: String,
            }
        }

        pub mod baz {
            use config::Config;

            #[derive(Config, Debug, PartialEq)]
            pub struct AppConfig {
                #[config(env)]
                pub database_url: String,
                #[config(nested)]
                pub kafka: super::bar::KafkaConfig,
            }
        }
    }

    let expected = AppConfig {
        database_url: "postgres://postgres@postgres/postgres".into(),
        kafka: KafkaConfig {
            brokers: "kafka:29092".into(),
        },
    };

    let actual = AppConfig::configure()
        .database_url("postgres://postgres@postgres/postgres".into())
        .kafka(|kafka| kafka.brokers("kafka:29092".into()))
        .finalize()
        .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn optional_nested_structs() {
    // Simulate composing config from different modules
    use foo::baz::AppConfig;

    mod foo {
        pub mod bar {
            use config::Config;

            #[derive(Config, Debug, PartialEq)]
            pub struct KafkaConfig {
                #[config(env = "KAFKA_BROKERS")]
                pub brokers: String,
                #[config(env = "KAFKA_TOPICS")]
                pub topics: String,
            }
        }

        pub mod baz {
            use config::Config;

            #[derive(Config, Debug, PartialEq)]
            pub struct AppConfig {
                #[config(env)]
                pub database_url: String,
                #[config(nested)]
                pub kafka: Option<super::bar::KafkaConfig>,
            }
        }
    }

    let expected = AppConfig {
        database_url: "postgres://postgres@postgres/postgres".into(),
        kafka: None,
    };

    // Leave KAFKA_TOPICS unset
    let actual = temp_env::with_vars(
        [
            (
                "DATABASE_URL",
                Some("postgres://postgres@postgres/postgres"),
            ),
            ("KAFKA_BROKERS", Some("kafka:29092")),
        ],
        || AppConfig::configure().finalize(),
    )
    .unwrap();

    assert_eq!(expected, actual);
}
