use reqwest::Client;

use crate::collector::manager::HealthyProxy;
use crate::collector::manager::PorxyMetadata;

#[derive(Debug, Clone)]
pub struct RouteContext {
    pub proxy: HealthyProxy,
    pub client: Client
}

impl RouteContext {
    pub fn dummy() -> Self {
        Self{
            proxy: HealthyProxy {
                ip: "None".to_string(),
                port: 0,
                latency: 0,
                metadata: PorxyMetadata { country: "None".to_string(), country_code: "None".to_string(), region: "None".to_string(), city: "None".to_string()}
            },
            client: Client::default(),

        }
    }
}