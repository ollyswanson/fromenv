use fromenv::FromEnv;

#[derive(FromEnv)]
pub struct Config {
    #[env(from, invalid)]
    foo: String,
}

fn main() {}
