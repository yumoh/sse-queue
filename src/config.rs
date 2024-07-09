use serde::{Deserialize, Serialize};
/*
[server]
address="127.0.0.1:8545;::1:8545"
token="secret"
timeout=300
workers=2

[ssl]
cert=""
key=""
*/

fn default_workers() -> usize {
    2
}

fn default_timeout() -> u64 {
    15 * 60
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigServer {
    pub bind: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default,rename="log-level",skip_serializing_if="Option::is_none")]
    pub log_level: Option<String>,
    #[serde(default,skip_serializing_if="String::is_empty")]
    pub prefix: String,
}

impl Default for ConfigServer {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8545;::1:8545".to_string(),
            timeout: 300,
            workers: 2,
            log_level: None,
            prefix: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigSSL {
    pub cert: String,
    pub key: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ConfigAuth {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub server: ConfigServer,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ssl: Option<ConfigSSL>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub auth: Option<ConfigAuth>,
}

impl Config {
    /// 从 toml 配置中加载配置
    pub fn load(config_path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let config = std::fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&config)?)
    }
    /// 将配置保存到 toml 文件中
    #[allow(unused)]
    pub fn dump(&self, config_path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let config = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, config)?;
        Ok(())
    }
    pub fn update_with_cli(&mut self, cli: &clap::ArgMatches) -> anyhow::Result<()> {
        if let Some(address) = cli.get_one::<String>("bind") {
            self.server.bind.clone_from(address);
        }
        // if let Some(token) = cli.get_one::<String>("token") {
        //     self.server.token = token.clone();
        // }
        // if let Some(timeout) = cli.get_one::<u64>("timeout") {
        //     self.server.timeout = *timeout;
        // }
        if let Some(workers) = cli.get_one::<usize>("workers") {
            self.server.workers = *workers;
        }
        if let Some(log_level) = cli.get_one::<String>("log-level") {
            self.server.log_level = Some(log_level.clone());
        }
        let ssl_cert = cli.get_one::<String>("ssl_cert");
        let ssl_key = cli.get_one::<String>("ssl_key");
        if ssl_cert.is_some() || ssl_key.is_some() {
            self.ssl = Some(ConfigSSL {
                cert: ssl_cert.cloned().unwrap_or_default(),
                key: ssl_key.cloned().unwrap_or_default(),
            });
        }
        Ok(())
    }
}
