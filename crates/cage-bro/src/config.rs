#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub sandbox: SandboxConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct SandboxConfig {
    pub runtime: String,
    pub memory_limit_mb: u64,
    pub cpu_limit_percent: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            sandbox: SandboxConfig {
                runtime: "process".to_string(),
                memory_limit_mb: 512,
                cpu_limit_percent: 50,
            },
        }
    }
}
