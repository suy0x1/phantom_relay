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

use phantom_relay::{
    ipc::protocol::{
        IPCRequest,
        IPCResponse,
    },

    runtime::{
        commands::RuntimeCommands,
        service::Service,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
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
        IPCRequest::Runtime(
            RuntimeCommands::Start(
                Service::DNS
            )
        );

    let json =
        serde_json::to_string(
            &request
        )?;

    framed.send(json).await?;

    if let Some(line) =
        framed.next().await {

        let line = line?;

        let response: IPCResponse =
            serde_json::from_str(
                &line
            )?;

        println!(
            "{:#?}",
            response
        );
    }

    Ok(())
}