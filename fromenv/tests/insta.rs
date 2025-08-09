//! These set of tests are supposed to provide an example for combining the
//! `requirements` method with insta to document the environment variables
//! needed by an application.

#[test]
fn snapshot_as_documentation() {
    use foo::bar::AppConfig;
    // simulate module structure.
    #[allow(unused)]
    mod foo {
        pub mod bar {
            use std::net::SocketAddr;

            use fromenv::FromEnv;

            use super::baz::KafkaConfig;

            #[derive(FromEnv)]
            pub struct AppConfig {
                #[env(from, default = "postgres://postgres@postgres/postgres", with = into)]
                pub database_url: String,
                #[env(from, default = "127.0.0.1:3000")]
                pub socket_addr: SocketAddr,
                #[env(nested)]
                pub kafka: KafkaConfig,
            }
        }

        pub mod baz {
            use fromenv::FromEnv;

            #[derive(FromEnv)]
            pub struct KafkaConfig {
                #[env(from = "KAFKA_BROKER")]
                pub broker: String,
            }
        }
    }

    insta::assert_snapshot!(AppConfig::requirements())
}
