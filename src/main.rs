mod config;
mod loginit;
mod apps;



fn parse_cli() -> clap::Command {
    clap::Command::new("sse")
        .about("msg queue use sse")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .required(false)
                .value_name("FILE")
                .help("init from toml config."),
        )
        .arg(
            clap::Arg::new("log-level")
                .short('l')
                .long("log-level")
                .required(false)
                .help("set log level"),
        )
        .arg(
            clap::Arg::new("bind")
                .short('b')
                .long("bind")
                .required(false)
                .help("server address. default: 127.0.0.0:8585"),
        )
        .arg(
            clap::Arg::new("workers")
                .short('w')
                .long("workers")
                .required(false)
                .value_parser(clap::value_parser!(usize))
                .help("worker nums"),
        )
        .arg(
            clap::Arg::new("ssl_cert")
                .long("ssl-cert")
                .required(false)
                .help("ssl cert file path"),
        )
        .arg(
            clap::Arg::new("ssl_key")
                .long("ssl-key")
                .required(false)
                .help("ssl key file path"),
        )
}
fn main() -> anyhow::Result<()> {
    let cli = parse_cli();
    let matches = cli.get_matches();
    let cpath = matches.get_one::<String>("config");
    let mut cfg = if let Some(p) = cpath {
        config::Config::load(p)?
    } else {
        config::Config::default()
    };
    cfg.update_with_cli(&matches)?;
    if let Some(level) = &cfg.server.log_level {
        loginit::init_level(level);
    } else {
        loginit::init_from_env(None);
    }
    log::debug!("\n== config ==\n{cfg:?}\n== ==");
    apps::server_up(&cfg)?;
    println!("ok");
    Ok(())
}
