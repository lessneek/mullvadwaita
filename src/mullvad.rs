use crate::prelude::*;

use std::time::Duration;

use anyhow::{Context, Result};

use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::states::TunnelState;
use tokio::sync::mpsc::{self, Receiver, Sender};

use futures::StreamExt;

#[derive(Debug)]
pub enum Event {
    TunnelState(Box<TunnelState>),
    ConnectingToDaemon,
}

pub fn watch() -> Receiver<Event> {
    let (sender, receiver) = mpsc::channel(100);

    tokio::spawn(async move {
        while !sender.is_closed() && (sender.send(Event::ConnectingToDaemon).await).is_ok() {
            trace!("Starting listening for RPC.");
            match listen(&sender).await {
                Ok(_) => trace!("RPC listening ended Ok."),
                Err(err) => trace!("RPC listening error: {}", err),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    receiver
}

pub fn secure_my_connection() {
    tokio::spawn(async {
        let mut client = get_mullvad_proxy_client().await?;
        anyhow::Ok(client.connect_tunnel().await.context("connect tunnel")?)
    });
}

pub fn disconnect() {
    tokio::spawn(async {
        let mut client = get_mullvad_proxy_client().await?;
        anyhow::Ok(
            client
                .disconnect_tunnel()
                .await
                .context("disconnect tunnel")?,
        )
    });
}

pub fn reconnect() {
    tokio::spawn(async {
        let mut client = get_mullvad_proxy_client().await?;
        anyhow::Ok(
            client
                .reconnect_tunnel()
                .await
                .context("reconnect tunnel")?,
        )
    });
}

async fn get_mullvad_proxy_client() -> Result<MullvadProxyClient> {
    MullvadProxyClient::new()
        .await
        .context("mullvad proxy client connection")
}

async fn listen(sender: &Sender<Event>) -> Result<()> {
    let mut client = get_mullvad_proxy_client().await?;

    let state = client.get_tunnel_state().await?;
    sender.send(Event::TunnelState(Box::new(state))).await?;

    while let Some(event) = client.events_listen().await?.next().await {
        match event? {
            DaemonEvent::TunnelState(new_state) => {
                trace!("New tunnel state: {new_state:#?}");
                sender.send(Event::TunnelState(Box::new(new_state))).await?;
            }
            DaemonEvent::Settings(settings) => {
                trace!("New settings: {settings:#?}");
            }
            DaemonEvent::RelayList(relay_list) => {
                trace!("New relay list: {relay_list:#?}");
            }
            DaemonEvent::AppVersionInfo(app_version_info) => {
                trace!("New app version info: {app_version_info:#?}");
            }
            DaemonEvent::Device(device) => {
                trace!("Device event: {device:#?}");
            }
            DaemonEvent::RemoveDevice(device) => {
                trace!("Remove device event: {device:#?}");
            }
            DaemonEvent::NewAccessMethod(access_method) => {
                trace!("New access method: {access_method:#?}");
            }
        }
    }
    Ok(())
}
