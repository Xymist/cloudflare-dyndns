use crate::Opt;
use color_eyre::Report;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloudflareResult {
    #[serde(rename = "result")]
    zones: Vec<CloudflareZone>,
    result_info: crate::cloudflare_meta::CloudflareResultInfo,
    success: bool,
    errors: Vec<Option<serde_json::Value>>,
    messages: Vec<Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloudflareZone {
    id: String,
    name: String,
    status: String,
    paused: bool,
    #[serde(rename = "type")]
    result_type: String,
    development_mode: i64,
    name_servers: Vec<String>,
    original_name_servers: Vec<String>,
    original_registrar: Option<serde_json::Value>,
    original_dnshost: Option<serde_json::Value>,
    modified_on: String,
    created_on: String,
    activated_on: String,
    meta: Meta,
    owner: Owner,
    account: Account,
    permissions: Vec<String>,
    plan: Plan,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    step: i64,
    wildcard_proxiable: bool,
    custom_certificate_quota: i64,
    page_rule_quota: i64,
    phishing_detected: bool,
    multiple_railguns_allowed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Owner {
    id: String,
    #[serde(rename = "type")]
    owner_type: String,
    email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plan {
    id: String,
    name: String,
    price: i64,
    currency: String,
    frequency: String,
    is_subscribed: bool,
    can_subscribe: bool,
    legacy_id: String,
    legacy_discount: bool,
    externally_managed: bool,
}

pub(crate) async fn zone_id(client: reqwest::Client, opt: &Opt) -> Result<String, Report> {
    let res: CloudflareResult = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones?name={}",
            opt.domain
        ))
        .bearer_auth(&opt.token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let zone = res
        .zones
        .get(0)
        .ok_or(Report::msg("Failed to receive any Zones"))?;

    Ok(zone.id.clone())
}
