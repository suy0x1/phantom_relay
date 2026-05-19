use crate::runtime::service::Service;

pub enum RuntimeCommands {
    Start(Service),
    Stop(Service),
    Restart(Service),
    Status,
    Shutdown,
}