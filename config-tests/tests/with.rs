use std::str;

use config::{Config, ParserResult};

#[test]
fn with_into_parser() {
    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env, default = "postgres://postgres@postgres/postgres", with = into)]
        database_url: String,
    }

    let expected = Config {
        database_url: "postgres://postgres@postgres/postgres".into(),
    };
    let actual = Config::configure().finalize().unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn with_into_secret() {
    use secrecy::ExposeSecret;

    #[derive(Config, Debug)]
    pub struct Config {
        #[config(env = "API_KEY", with = into)]
        api_key: secrecy::SecretString,
    }

    let expected = Config {
        api_key: "definitely-not-an-api-key".into(),
    };

    let actual = temp_env::with_var("API_KEY", Some("definitely-not-an-api-key"), || {
        Config::configure().finalize()
    })
    .unwrap();

    assert_eq!(
        expected.api_key.expose_secret(),
        actual.api_key.expose_secret()
    );
}

#[test]
fn with_custom_parser_function() {
    fn frobnicate(s: &str) -> ParserResult<u16> {
        let mut v = s.as_bytes().to_vec();
        v.rotate_left(2);
        let s = str::from_utf8(&v)?;
        s.parse::<u16>().map_err(|e| e.into())
    }

    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env = "SERVER_PORT", with = frobnicate)]
        port: u16,
    }

    let expected = Config { port: 30 };

    let actual = temp_env::with_var("SERVER_PORT", Some("3000"), || {
        Config::configure().finalize()
    })
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn collection_types() {
    use foo::baz::Config;
    // simulate module structure
    mod foo {
        pub mod bar {
            use config::ParserResult;

            pub fn comma_separated(s: &str) -> ParserResult<Vec<String>> {
                Ok(s.split(',').map(ToOwned::to_owned).collect())
            }
        }

        pub mod baz {
            use config::Config;

            #[derive(Config, Debug, PartialEq)]
            pub struct Config {
                #[config(env = "KAFKA_TOPICS", with = super::bar::comma_separated)]
                pub topics: Vec<String>,
            }
        }
    }

    let expected = Config {
        topics: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
    };

    let actual = temp_env::with_var("KAFKA_TOPICS", Some("a,b,c"), || {
        Config::configure().finalize()
    })
    .unwrap();

    assert_eq!(expected, actual);
}
