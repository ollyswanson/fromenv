use fromenv::FromEnv;

#[derive(FromEnv)]
pub struct Config {
    #[env(default = "a")]
    a: String,
    #[env(with = into)]
    b: String,
    #[env(default = "a", with = into)]
    c: String,
}

fn main() {}
