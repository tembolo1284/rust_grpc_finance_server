use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 50051,
            },
            client: ClientConfig {
                host: "grpc-finance-server".to_string(),
                port: 50051,
            },
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config/config.toml".to_string());
    let config_path = Path::new(&config_path);
    
    if !config_path.exists() {
        println!("Config file not found at {:?}, using default configuration", config_path);
        return Ok(Config::default());
    }

    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;
    println!("Loaded configuration: {:?}", config);
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.client.host, "grpc-finance-server");
        assert_eq!(config.client.port, 50051);
    }

    #[test]
    fn test_load_config_default() {
        let config = load_config().unwrap();
        assert_eq!(config.server.port, 50051);
    }

    #[test]
    fn test_load_custom_config() {
        let config_content = r#"
            [server]
            host = "0.0.0.0"
            port = 50051

            [client]
            host = "localhost"
            port = 50051
        "#;

        std::fs::create_dir_all("config").unwrap();
        let mut file = File::create("config/config.toml").unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        let config = load_config().unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.client.host, "localhost");
        assert_eq!(config.client.port, 50051);

        std::fs::remove_file("config/config.toml").unwrap();
    }
}
