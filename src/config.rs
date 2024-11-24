use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 50051,  // Changed from 8080 to 50051
            },
            client: ClientConfig {
                host: "127.0.0.1".to_string(),
                port: 50051,  // Changed from 8080 to 50051
            },
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new("config/config.toml");
    
    if !config_path.exists() {
        println!("Config file not found, using default configuration");
        return Ok(Config::default());
    }

    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;
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
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 50051);  // Updated assertion
        assert_eq!(config.client.host, "127.0.0.1");
        assert_eq!(config.client.port, 50051);  // Updated assertion
    }

    #[test]
    fn test_load_config_default() {
        let config = load_config().unwrap();
        assert_eq!(config.server.port, 50051);  // Updated assertion
    }

    #[test]
    fn test_load_custom_config() {
        // Create a temporary config file
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
        assert_eq!(config.server.port, 50051);  // Updated assertion
        assert_eq!(config.client.host, "localhost");
        assert_eq!(config.client.port, 50051);  // Updated assertion

        // Clean up
        std::fs::remove_file("config/config.toml").unwrap();
    }
}
