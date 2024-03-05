pub struct Config {
    pub mongodb_url: String,
    pub port: u16,
    pub socket_path: String,
}

pub fn config() -> Config {
    Config {
        mongodb_url: std::env::var("MONGODB_URL").unwrap(),
        port: std::env::var("PORT").unwrap().parse::<u16>().unwrap(),
        socket_path: std::env::var("SOCKET_PATH").unwrap(),
    }
}
