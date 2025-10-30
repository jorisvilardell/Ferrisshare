#[derive(Debug)]
pub struct Config {
    pub ferris_base_path: String,
    pub ferris_port: u16,
    pub ferris_host: String,
}

impl Config {
    pub fn from_env() -> Self {
        let ferris_base_path =
            std::env::var("FERRIS_BASE_PATH").unwrap_or_else(|_| "./public".to_string());
        let ferris_port = std::env::var("FERRIS_PORT")
            .unwrap_or_else(|_| "9000".to_string())
            .parse()
            .expect("FERRIS_PORT must be a valid u16");
        let ferris_host = std::env::var("FERRIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        Config {
            ferris_base_path,
            ferris_port,
            ferris_host,
        }
    }
}
