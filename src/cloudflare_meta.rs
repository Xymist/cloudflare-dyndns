use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    auto_added: bool,
    managed_by_apps: bool,
    managed_by_argo_tunnel: bool,
    source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloudflareResultInfo {
    page: i64,
    per_page: i64,
    count: i64,
    total_count: i64,
    total_pages: i64,
}
