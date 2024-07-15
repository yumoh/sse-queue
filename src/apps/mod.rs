mod auth;

mod error;
mod init;
mod msg;
mod state;
mod tools;
mod seekstream;
mod storage;
mod onlinelog;

use rocket::tokio::runtime::Runtime;
use rocket::{config::TlsConfig, Config};
use rocket::fs::FileServer;
use std::str::FromStr;

use crate::config::Config as CliConfig;
use error::{WebError, WebResult};

pub fn server_up(cfg: &CliConfig) -> anyhow::Result<()> {
    let cache = state::WebCache::new(cfg)?;

    let mut server_config = Config {
        workers: cfg.server.workers,
        max_blocking: 512,
        keep_alive: 15,
        cli_colors: true,
        ..Default::default()
    };
    let https_on = cfg.ssl.is_some();
    if let Some(ssl) = &cfg.ssl {
        server_config.tls = Some(TlsConfig::from_paths(&ssl.cert, &ssl.key));
        log::info!("https on");
    }
    log::info!("http on: {https_on}");

    let (host, port) = if cfg.server.bind.contains(':') {
        let mut parts = cfg.server.bind.split(':');
        let host = parts.next().unwrap().to_string();
        let port = parts.next().unwrap();
        let port: u16 = port.parse().expect("端口格式错误");
        (host, port)
    } else {
        (cfg.server.bind.clone(), if https_on { 443 } else { 80 })
    };
    server_config.address = std::net::IpAddr::from_str(&host).expect("地址格式错误");
    server_config.port = port;
    let base_api = init::routes();
    let msg_api = msg::routes();
    let storage_api = storage::routes();
    let online_log = onlinelog::routes();
    let fileserver = FileServer::from(cfg.public_workspace()?);
    let build = if cfg.server.prefix.is_empty() || &cfg.server.prefix == "/" {
        rocket::build()
            .configure(server_config)
            .manage(cache)
            .mount("/", base_api)
            .mount("/static/public", fileserver)
            .mount("/msg", msg_api)
            .mount("/storage", storage_api)
            .mount("/onlinelog", online_log)
    } else {
        rocket::build()
            .configure(server_config)
            .manage(cache)
            .mount("/", base_api.clone())
            .mount(&cfg.server.prefix, base_api)
            .mount("/static/public", fileserver.clone())
            .mount(&format!("{}/static/public", cfg.server.prefix), fileserver)
            .mount("/msg", msg_api.clone())
            .mount(&format!("{}/msg", cfg.server.prefix), msg_api)
            .mount("/storage", storage_api.clone())
            .mount(&format!("{}/storage", cfg.server.prefix), storage_api)
            .mount("/onlinelog", online_log.clone())
            .mount(&format!("{}/onlinelog", cfg.server.prefix), online_log)
    };
    let rt = Runtime::new()?;
    rt.block_on(async {
        // if let Err(err) = cache_clone.load_stateful().await {
        //     log::error!("load stateful error: {:?}", err);
        // }
        let _ = build.launch().await;
    });
    Ok(())
}
