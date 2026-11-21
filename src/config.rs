pub struct Config {
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Config, String> {
        let port = match std::env::var("TICKETS_PORT") {
            Ok(raw) => raw
                .parse::<u16>()
                .map_err(|_| format!("TICKETS_PORT is not a valid port: {raw:?}"))?,
            Err(_) => 3000,
        };

        let db_host = std::env::var("TICKETS_DB_HOST")
            .map_err(|_| "TICKETS_DB_HOST is not set.".to_string())?;
        let db_port = match std::env::var("TICKETS_DB_PORT") {
            Ok(raw) => raw
                .parse::<u16>()
                .map_err(|_| format!("TICKETS_DB_PORT is not a valid port: {raw:?}"))?,
            Err(_) => 5432,
        };
        let db_name = std::env::var("TICKETS_DB_NAME")
            .map_err(|_| "TICKETS_DB_NAME is not set.".to_string())?;
        let db_user = std::env::var("TICKETS_DB_USER")
            .map_err(|_| "TICKETS_DB_USER is not set.".to_string())?;
        let db_password = std::env::var("TICKETS_DB_PASSWORD")
            .map_err(|_| "TICKETS_DB_PASSWORD is not set.".to_string())?;

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        Ok(Config { port, database_url })
    }
}