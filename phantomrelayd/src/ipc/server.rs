use anyhow::Result;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LinesCodec};

use crate::{
    ipc::protocol::{IPCRequest, IPCResponse},
    runtime::controller::RuntimeController,
};

pub async fn start_ipc_server(runtime: Arc<Mutex<RuntimeController>>) -> Result<()> {
    let _ = std::fs::remove_file("/run/phantomrelay.sock");
    let listener = UnixListener::bind("/run/phantomrelay.sock")?;

    loop {
        let (stream, _) = listener.accept().await?;
        let runtime = runtime.clone();

        tokio::spawn(async move {
            let mut framed = Framed::new(stream, LinesCodec::new());

            while let Some(line) = framed.next().await {
                let line = match line {
                    Ok(v) => v,

                    Err(e) => {
                        let response = IPCResponse::Error {
                            message: format!("{}", e),
                        };
                        let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                            "{\"Error\":\"serialization failure\"}".to_string()
                        });
                        let _ = framed.send(json).await;
                        break;
                    }
                };

                let request: IPCRequest = match serde_json::from_str(&line) {
                    Ok(v) => v,

                    Err(e) => {
                        let response = IPCResponse::Error {
                            message: format!("{}", e),
                        };
                        let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                            "{\"Error\":\"serialization failure\"}".to_string()
                        });
                        let _ = framed.send(json).await;
                        continue;
                    }
                };
                let result = match request {
                    IPCRequest::Runtime(cmd) => runtime.lock().await.handle_commands(cmd).await,
                };

                let response = match result {
                    Ok(services) => IPCResponse::Status { services },

                    Err(e) => IPCResponse::Error {
                        message: format!("{}", e),
                    },
                };

                let json = match serde_json::to_string(&response) {
                    Ok(v) => v,

                    Err(e) => {
                        let fallback = IPCResponse::Error {
                            message: format!("{}", e),
                        };
                        serde_json::to_string(&fallback).unwrap_or_else(|_| {
                            "{\"Error\":\"fatal serialization failure\"}".to_string()
                        })
                    }
                };

                if framed.send(json).await.is_err() {
                    break;
                }
            }
        });
    }
}
