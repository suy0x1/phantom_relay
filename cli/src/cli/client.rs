use anyhow::Result;

use futures::{
    SinkExt,
    StreamExt,
};

use tokio::net::UnixStream;

use tokio_util::codec::{
    Framed,
    LinesCodec,
};

use crate::{
    ipc::protocol::{
        IPCRequest,
        IPCResponse,
    },

    runtime::commands::RuntimeCommands,
};

pub async fn send_command(
    cmd: RuntimeCommands,
) -> Result<IPCResponse> {

    let stream =
        UnixStream::connect(
            "/run/phantomrelay.sock"
        )
        .await?;

    let mut framed =
        Framed::new(
            stream,
            LinesCodec::new(),
        );

    let request =
        IPCRequest::Runtime(cmd);

    let json =
        serde_json::to_string(
            &request
        )?;

    framed.send(json).await?;

    let line =
        framed
            .next()
            .await
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "daemon disconnected"
                )
            })??;

    let response =
        serde_json::from_str::<IPCResponse>(
            &line
        )?;

    Ok(response)
}