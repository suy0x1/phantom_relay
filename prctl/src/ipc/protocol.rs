use serde::{Deserialize, Serialize};

use crate::runtime::commands::RuntimeCommands;
use crate::runtime::status::ServiceStatus;
use crate::runtime::metrics::MetricsSnapshot;

/// Request message sent over IPC to the daemon.
///
/// Encapsulates runtime commands to control services and modes.
#[derive(Serialize, Deserialize, Debug)]
pub enum IPCRequest {
    /// Runtime command (start, stop, restart services or modes).
    Runtime(RuntimeCommands),
}

/// Response message sent from the daemon over IPC.
///
/// Indicates success, failure, or returns service status information.
#[derive(Serialize, Deserialize, Debug)]
pub enum IPCResponse {
    /// Successful operation completion.
    Success {
        /// Human-readable success message.
        message: String,
    },

    /// Current status of all managed services.
    Status {
        /// Collection of service status entries.
        services: Vec<ServiceStatus>,
    },

    /// Current daemon configuration.
    Config(
        /// Human-readable configuration output.
        String,
    ),

    /// Connection-related information.
    Conn(
        /// Human-readable connection status output.
        String,
    ),

    /// DNS-related information.
    DNS(
        /// Human-readable DNS status output.
        String,
    ),

    /// Proxy-related information.
    Proxy(
        /// Human-readable proxy status output.
        String,
    ),

    /// Routing-related information.
    Route(
        /// Human-readable routing status output.
        String,
    ),

    /// Snapshot of daemon metrics.
    Metrics {
        /// Collected metrics data.
        data: MetricsSnapshot,
    },

    /// Operation failed with an error.
    Error {
        /// Human-readable error message describing what went wrong.
        message: String,
    },
}