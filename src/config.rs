use serde::Deserialize;
use std::env;
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
                host: get_default_client_host(),
                port: 50051,
            },
        }
    }
}

fn get_default_client_host() -> String {
    env::var("GRPC_CLIENT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.toml".to_string());
    let config_path = Path::new(&config_path);

    let mut config = if !config_path.exists() {
        println!(
            "Config file not found at {:?}, using default configuration",
            config_path
        );
        Config::default()
    } else {
        let config_str = fs::read_to_string(config_path)?;
        toml::from_str(&config_str)?
    };

    if let Ok(host) = env::var("GRPC_CLIENT_HOST") {
        config.client.host = host;
    }

    println!("Loaded configuration: {:?}", config);
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        env::remove_var("GRPC_CLIENT_HOST");
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.client.host, "127.0.0.1"); // Updated default
        assert_eq!(config.client.port, 50051);
    }

    #[test]
    fn test_default_config_with_env() {
        env::set_var("GRPC_CLIENT_HOST", "test-host");
        let config = Config::default();
        assert_eq!(config.client.host, "test-host");
        env::remove_var("GRPC_CLIENT_HOST");
    }

    #[test]
    fn test_load_config_default() {
        env::remove_var("GRPC_CLIENT_HOST");
        let config = load_config().unwrap();
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.client.port, 50051);
        assert_eq!(config.client.host, "127.0.0.1"); // Updated default
    }

    #[test]
    fn test_load_custom_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let config_content = r#"
[server]
host = "0.0.0.0"
port = 50051
[client]
host = "grpc-finance-server"
port = 50051
"#;
        fs::write(&config_path, config_content).unwrap();
        env::set_var("CONFIG_PATH", config_path.to_str().unwrap());

        env::remove_var("GRPC_CLIENT_HOST");
        let config = load_config().unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.client.host, "grpc-finance-server");
        assert_eq!(config.client.port, 50051);

        env::set_var("GRPC_CLIENT_HOST", "test-host");
        let config = load_config().unwrap();
        assert_eq!(config.client.host, "test-host");

        env::remove_var("GRPC_CLIENT_HOST");
    }

    #[test]
    fn test_config_environment_override() {
        env::set_var("GRPC_CLIENT_HOST", "override-host");
        let config = load_config().unwrap();
        assert_eq!(config.client.host, "override-host");
        env::remove_var("GRPC_CLIENT_HOST");
    }
}
