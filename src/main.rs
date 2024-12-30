mod sitemap;
use crawler::crawl_site;

mod config;
use config::Config;

mod crawler;

use std::{path::PathBuf, process::exit};
use clap::Parser;
use reqwest::Client;
use tokio::fs;

#[derive(clap::Parser)]
pub struct Args {
    /// Path to the TOML config
    #[arg(short, long, default_value = "./scrape.toml")]
    config: PathBuf,

    /// Create a default config file
    #[arg(short, long, default_value = "false", action)]
    init: bool,
    
    /// Don't write any pages
    #[arg(long, default_value = "false", action)]
    dry_run: bool,
    
    /// Target directory for pages
    #[arg(short, long, default_value = "./")]
    target: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.init {
        if let Err(err) = fs::write(
            &args.config,
            // TODO: make a test for it
            toml::to_string_pretty(
                &Config::default()
            ).expect("the default config should be valid"),
        ).await {
            eprintln!("Couldn't create config at {}: {err}", args.config.to_string_lossy());
            exit(20);
        }
    }

    let config_file = match fs::read_to_string(&args.config).await {
        Ok(it) => it,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                eprintln!(
                    "You don't have a config file at {}: {err}\nYou can create one with --init",
                    args.config.to_string_lossy(),
                );
                exit(11);
            },
            _ => {
                eprintln!("Couldn't open config at {}: {err}", args.config.to_string_lossy());
                exit(10);
            },
        },
    };

    let client = Client::new();
    // TODO: a proper error message
    let config = toml::from_str(&config_file).unwrap();

    crawl_site(&config, &client, &args).await;
}
