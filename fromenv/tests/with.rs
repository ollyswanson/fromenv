use std::str;

use fromenv::{FromEnv, ParserResult};

#[test]
fn with_into_parser() {
    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from, default = "postgres://postgres@postgres/postgres", with = into)]
        database_url: String,
    }

    let expected = Config {
        database_url: "postgres://postgres@postgres/postgres".into(),
    };
    let actual = Config::from_env().finalize().unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn with_into_secret() {
    use secrecy::ExposeSecret;

    #[derive(FromEnv, Debug)]
    pub struct Config {
        #[env(from = "API_KEY", with = into)]
        api_key: secrecy::SecretString,
    }

    let expected = Config {
        api_key: "definitely-not-an-api-key".into(),
    };

    let actual = temp_env::with_var("API_KEY", Some("definitely-not-an-api-key"), || {
        Config::from_env().finalize()
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
        s.strip_prefix("0o")
            .ok_or("not an octal".into())
            .and_then(|s| u16::from_str_radix(s, 8).map_err(|e| e.into()))
    }

    #[derive(FromEnv, Debug, PartialEq)]
    pub struct Config {
        #[env(from = "SERVER_PORT", with = frobnicate)]
        port: u16,
    }

    let expected = Config { port: 24 };

    let actual = temp_env::with_var("SERVER_PORT", Some("300o"), || {
        Config::from_env().finalize()
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
            use fromenv::ParserResult;

            pub fn comma_separated(s: &str) -> ParserResult<Vec<String>> {
                Ok(s.split(',').map(ToOwned::to_owned).collect())
            }
        }

        pub mod baz {
            use fromenv::FromEnv;

            #[derive(FromEnv, Debug, PartialEq)]
            pub struct Config {
                #[env(from = "KAFKA_TOPICS", with = super::bar::comma_separated)]
                pub topics: Vec<String>,
            }
        }
    }

    let expected = Config {
        topics: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
    };

    let actual = temp_env::with_var("KAFKA_TOPICS", Some("a,b,c"), || {
        Config::from_env().finalize()
    })
    .unwrap();

    assert_eq!(expected, actual);
}
