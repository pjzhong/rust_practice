use std::collections::HashMap;

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
    pub local_addr: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct ClientConfig {
    pub remote_addr: String,
    pub default_token: Option<String>,
    pub services: HashMap<String, ClientServiceConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub default_token: Option<String>,
    pub services: HashMap<String, ServerServiceConfig>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub server: Option<ServerConfig>,
    pub client: Option<ClientConfig>,
}

impl Config {

}




