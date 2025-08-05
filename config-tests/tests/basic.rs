use config::Config;

#[test]
fn basic_init_config() {
    #[derive(Config, Debug, PartialEq)]
    pub struct Config {
        #[config(env = "HOST")]
        server_host: String,
        #[config(default = "8080")]
        server_port: u16,
        #[config]
        database_url: String,
    }

    let expected = Config {
        server_host: "localhost".into(),
        server_port: 3000,
        database_url: "postgres://user@localhost/postgres".into(),
    };

    let actual = temp_env::with_vars(
        [
            ("HOST", Some("localhost")),
            ("SERVER_PORT", Some("3000")),
            ("DATABASE_URL", Some("postgres://user@localhost/postgres")),
        ],
        || Config::configure().finalize().unwrap(),
    );

    assert_eq!(expected, actual);
}
