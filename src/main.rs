use std::{net::Ipv4Addr, str::FromStr};

use color_eyre::Report;
use futures::{stream::FuturesUnordered, StreamExt};
use structopt::StructOpt;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod cloudflare_meta;
mod cloudflare_record;
mod cloudflare_zone;

const SERVICE_URLS: [&'static str; 3] = [
    "https://checkip.amazonaws.com",
    "https://api.ipify.org",
    "https://api.my-ip.io/ip",
];

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    domain: String,
    #[structopt(short, long)]
    fqdn: String,
    #[structopt(short, long)]
    token: String,
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "0")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup()?;
    let opt = Opt::from_args();

    let client = reqwest::Client::new();
    let ip_response = fetch_ips(client.clone()).await?;
    let current_record = cloudflare_record::record_data(client.clone(), &opt).await?;

    if ip_response == current_record.ip() {
        info!(ip=?ip_response, "IP address is unchanged.");
        return Ok(());
    }

    cloudflare_record::update_record(
        client,
        &opt,
        current_record.zone_id(),
        current_record.id(),
        ip_response,
    )
    .await?;

    info!(old_ip=?current_record.ip(), new_ip=?ip_response, "Updated IP");

    Ok(())
}

async fn fetch_ips(client: reqwest::Client) -> Result<Ipv4Addr, Report> {
    let default_ip = Ipv4Addr::from_str("1.1.1.1")?;
    let mut ip_response = default_ip;

    let mut futs = SERVICE_URLS
        .iter()
        .map(|url| fetch_ip(client.clone(), url))
        .collect::<FuturesUnordered<_>>();

    while let Some(res) = futs.next().await {
        if res.is_err() {
            continue;
        }
        ip_response = Ipv4Addr::from_str(res?.trim())?;
        break;
    }

    if ip_response != default_ip {
        return Ok(ip_response);
    }

    Err(Report::msg("Failed to receive any IPs"))
}

async fn fetch_ip(client: reqwest::Client, url: &str) -> Result<String, Report> {
    let res = client.get(url).send().await?.error_for_status()?;
    let text = res.text().await?;
    Ok(text)
}
