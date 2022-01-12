use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum TransportType {
    #[serde(rename = "tcp")]
    Tcp,
}

impl Default for TransportType {
    fn default() -> Self {
        TransportType::Tcp
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum ServiceType {
    #[serde(rename = "tcp")]
    Tcp,
}

impl Default for ServiceType {
    fn default() -> Self {
        ServiceType::Tcp
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClientServiceConfig {
    #[serde(rename = "type", default = "ServiceType::default")]
    pub service_type: ServiceType,
    #[serde(skip)]
    pub name: String,
    pub local_addr: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ServerServiceConfig {
    #[serde(rename = "type", default = "ServiceType::default")]
    pub service_type: ServiceType,
    #[serde(skip)]
    pub name: String,
    pub bind_addr: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct ClientConfig {
    pub remote_addr: String,
    pub default_token: Option<String>,
    pub services: HashMap<String, ClientServiceConfig>,
    #[serde(default = "TransportType::default")]
    pub transport: TransportType
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub default_token: Option<String>,
    pub services: HashMap<String, ServerServiceConfig>,
    #[serde(default = "TransportType::default")]
    pub transport: TransportType

}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub server: Option<ServerConfig>,
    pub client: Option<ClientConfig>,
}

impl Config {
    fn from_str(s: &str) -> Result<Config> {
        let mut config: Config = toml::from_str(s).with_context(|| "Failed to parse config")?;

        if let Some(server) = config.server.as_mut() {
            Config::validate_server_config(server);
        }

        if let Some(client) = config.client.as_mut() {
            Config::validate_client_config(client);
        }

        if config.server.is_none() && config.client.is_none() {
            Err(anyhow!("Neither of `[server]` or `[client]` is defined"))
        } else {
            Ok(config)
        }
    }

    fn validate_server_config(server: &mut ServerConfig) -> Result<()> {
        for (name, s) in &mut server.services {
            s.name = name.clone();
            if s.token.is_none() {
                s.token = server.default_token.clone();
                if s.token.is_none() {
                    bail!("The token of service {} is not set", name)
                }
            }
        }

        Ok(())
    }

    fn validate_client_config(client: &mut ClientConfig) -> Result<()> {
        for (name, s) in &mut client.services {
            s.name = name.clone();
            if s.token.is_none() {
                s.token = client.default_token.clone();
                if s.token.is_none() {
                    bail!("The token of service {} is not set", name)
                }
            }
        }
        Ok(())
    }

    pub fn from_file(path: &Path) -> Result<Config> {
        let s = fs::read_to_string(path)
            .with_context(|| format!("Failed to read the config: {:?}", path))?;

        Config::from_str(&s).with_context(|| {
            "Configuration is invalid. Please refer to the configuration specification."
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use anyhow::Result;
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn test_simple_client_config() -> Result<()> {
        let path = PathBuf::from_str("tests/config_test/client.toml")?;
        let s = fs::read_to_string(path)?;
        println!("{:?}", Config::from_str(&s)?);

        Ok(())
    }
}
