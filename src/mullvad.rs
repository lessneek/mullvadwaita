use std::time::Duration;

use tokio::{
    sync::watch::{self, Receiver},
    time::sleep,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    Connected,
    Disconnected,
    Connecting,
    WaitingForService,
}

pub fn watch() -> Receiver<Status> {
    let (sender, receiver) = watch::channel(Status::WaitingForService);

    tokio::spawn(async move {
        while !sender.is_closed() {
            sleep(Duration::from_secs(3)).await;

            sender.send(Status::Connected).unwrap();

            sleep(Duration::from_secs(3)).await;

            sender.send(Status::Disconnected).unwrap();
        }
    });

    receiver
}
