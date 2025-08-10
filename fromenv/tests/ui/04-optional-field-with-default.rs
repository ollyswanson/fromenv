use fromenv::FromEnv;

#[derive(FromEnv)]
pub struct Config {
    #[env(from, default = "FOO")]
    foo: Option<String>,
}

fn main() {}
