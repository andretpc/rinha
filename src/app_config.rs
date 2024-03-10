pub struct Config {
    pub mongodb_url: String,
    pub socket_path: String,
}

pub fn config() -> Config {
    Config {
        mongodb_url: std::env::var("MONGODB_URL").unwrap(),
        socket_path: std::env::var("SOCKET_PATH").unwrap(),
    }
}
