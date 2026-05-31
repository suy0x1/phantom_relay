use serde::Deserialize;

/// Metadata about a proxy server's geographic location.
///
/// Contains location information fetched from the proxy's geographic database,
/// useful for understanding proxy distribution and latency characteristics.
#[derive(Debug, Clone, Deserialize, Hash, PartialEq, Eq)]
pub struct PorxyMetadata {
    /// Full name of the country where the proxy is located.
    #[serde(rename = "countryName")]
    pub country: String,
    /// ISO country code (e.g., "US", "GB").
    #[serde(rename = "countryCode")]
    pub country_code: String,
    /// Region or state name within the country.
    #[serde(rename = "regionName")]
    pub region: String,
    /// City name where the proxy is located.
    #[serde(rename = "cityName")]
    pub city: String,
}

/// A healthy proxy server that has passed connectivity and latency tests.
///
/// Represents a proxy that is currently available, responsive, and meets latency requirements.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HealthyProxy {
    /// The IP address of the proxy server.
    pub ip: String,
    /// The port on which the proxy is listening.
    pub port: u16,
    /// Geographic metadata about the proxy location.
    pub metadata: PorxyMetadata,
    /// Measured latency in milliseconds to this proxy.
    pub latency: u64,
}
