use config::ConfigError;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub jwt: JwtSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

/// JWT authentication settings
#[derive(serde::Deserialize, Clone)]
pub struct JwtSettings {
    pub secret: String,
    pub access_token_expiry: i64,   // seconds (e.g., 900 for 15 minutes)
    pub refresh_token_expiry: i64,  // seconds (e.g., 604800 for 7 days)
    pub issuer: String,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration").required(false))
        .build()?;
    settings.try_deserialize::<Settings>()
}