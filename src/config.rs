

pub struct Config {
    pub port: u16,
    pub database_url: String,
}

impl Config {
    /// Reads `TICKETS_PORT` and `DATABASE_URL` from the environment
    pub fn from_env() -> Result<Config, String> {
        let port = match std::env::var("TICKETS_PORT") {
            Ok(raw) => raw
                .parse::<u16>()
                .map_err(|_| format!("TICKETS_PORT is not a valid port: {raw:?}"))?,
            Err(_) => 3000,
        };
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is not set.".to_string())?;
        Ok(Config { port, database_url })
    }
}
