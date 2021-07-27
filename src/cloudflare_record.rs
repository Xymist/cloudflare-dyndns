use std::{net::Ipv4Addr, str::FromStr};

use crate::Opt;
use color_eyre::Report;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloudflareResult {
    #[serde(rename = "result")]
    records: Vec<DnsRecord>,
    success: bool,
    errors: Vec<Option<serde_json::Value>>,
    messages: Vec<Option<serde_json::Value>>,
    result_info: crate::cloudflare_meta::CloudflareResultInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsRecord {
    id: String,
    zone_id: String,
    zone_name: String,
    name: String,
    #[serde(rename = "type")]
    dns_record_type: String,
    content: String,
    proxiable: bool,
    proxied: bool,
    ttl: i64,
    locked: bool,
    meta: crate::cloudflare_meta::Meta,
    created_on: String,
    modified_on: String,
}

#[derive(Debug, Clone)]
pub struct CurrentIp {
    zone_id: String,
    id: String,
    ip: Ipv4Addr,
}

impl CurrentIp {
    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Get a reference to the current ip's zone id.
    pub fn zone_id(&self) -> &str {
        self.zone_id.as_str()
    }
}

pub(crate) async fn record_data(client: reqwest::Client, opt: &Opt) -> Result<CurrentIp, Report> {
    let zone_id = crate::cloudflare_zone::zone_id(client.clone(), opt).await?;
    let result: CloudflareResult = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=A&name={}",
            &zone_id, opt.fqdn
        ))
        .bearer_auth(&opt.token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let record = result.records.get(0).ok_or(Report::msg(
        "Did not receive any records for the given Zone and FQDN",
    ))?;

    Ok(CurrentIp {
        zone_id: zone_id,
        id: record.id.clone(),
        ip: Ipv4Addr::from_str(record.content.as_str())?,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdateData {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u8,
    proxied: bool,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct CloudflareDnsUpdateResult {
//     #[serde(rename = "result")]
//     record: DnsRecord,
//     success: bool,
//     errors: Vec<Option<serde_json::Value>>,
//     messages: Vec<Option<serde_json::Value>>,
// }

impl UpdateData {
    fn new(name: &str, new_ip: Ipv4Addr) -> Self {
        Self {
            record_type: "A".to_owned(),
            name: name.to_owned(),
            content: new_ip.to_string(),
            ttl: 1,
            proxied: true,
        }
    }
}

pub(crate) async fn update_record(
    client: reqwest::Client,
    opt: &Opt,
    zone_id: &str,
    record_id: &str,
    new_ip: Ipv4Addr,
) -> Result<(), Report> {
    let update_data = UpdateData::new(&opt.fqdn, new_ip);

    client
        .put(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        ))
        .bearer_auth(&opt.token)
        .json(&update_data)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(())
}
