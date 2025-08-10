use fromenv::FromEnv;

#[derive(FromEnv)]
pub struct Config {
    #[env]
    foo: String,
}

fn main() {}
