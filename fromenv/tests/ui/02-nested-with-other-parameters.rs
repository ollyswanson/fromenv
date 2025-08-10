use fromenv::FromEnv;

#[derive(FromEnv)]
pub struct Config {
    #[env(from, nested)]
    database_url: String,
}

fn main() {}
