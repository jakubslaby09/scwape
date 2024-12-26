mod sitemap;
use sitemap::scrape_menus;

mod config;
use config::Config;

use std::{path::PathBuf, process::exit};
use clap::Parser;
use reqwest::Client;
use tokio::fs;

#[derive(clap::Parser)]
struct Args {
    /// Path to the TOML config
    #[arg(short, long, default_value = "./scrape.toml")]
    config: PathBuf,

    /// Create a default config file
    #[arg(short, long, default_value = "false", action)]
    init: bool,
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
    let config = toml::from_str(&config_file).unwrap();
    scrape_site(&config, &client).await;
}

async fn scrape_site(config: &Config, client: &Client) {
    let res = client.get("https://gvh.cz")
    .send().await.expect("couldn't connect to site")
    .text().await.expect("couldn't download home page");
    
    scrape_page(&res, &config);
}

fn scrape_page(page: &str, config: &Config) {
    let dom = scraper::Html::parse_document(page);

    scrape_menus(&dom, config, None, 0);
}
