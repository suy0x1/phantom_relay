use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Hash, PartialEq, Eq)]
pub struct PorxyMetadata {
    #[serde(rename = "countryName")]
    pub country: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "regionName")]
    pub region: String,
    #[serde(rename = "cityName")]
    pub city: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HealthyProxy {
    pub ip: String,
    pub port: u16,
    pub metadata: PorxyMetadata,
    pub latency: u128,
}
