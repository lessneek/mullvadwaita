use crate::prelude::*;

use std::time::Duration;

use anyhow::{Context, Ok, Result};

use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::states::TunnelState;
use smart_default::SmartDefault;
use tokio::sync::mpsc::{self, Receiver, Sender};

use futures::StreamExt;

#[derive(Debug)]
pub enum Event {
    TunnelState(Box<TunnelState>),
    ConnectingToDaemon,
}

pub fn events_receiver() -> Receiver<Event> {
    let (sender, receiver) = mpsc::channel(10);

    tokio::spawn(async move {
        while !sender.is_closed() && (sender.send(Event::ConnectingToDaemon).await).is_ok() {
            trace!("Starting listening for RPC.");
            match events_listen(&sender).await {
                Result::Ok(_) => trace!("RPC listening ended Ok."),
                Err(err) => trace!("RPC listening error: {}", err),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    receiver
}

async fn events_listen(sender: &Sender<Event>) -> Result<()> {
    let mut client = MullvadProxyClient::new()
        .await
        .context("mullvad proxy client connection")?;

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

#[derive(Debug, SmartDefault)]
pub struct DaemonConnector {
    client: Option<MullvadProxyClient>,
}

impl DaemonConnector {
    async fn get_client(&mut self) -> Result<&mut MullvadProxyClient> {
        let client = &mut self.client;

        let client = client.get_or_insert(
            MullvadProxyClient::new()
                .await
                .context("mullvad proxy client connection")
                .inspect_err(|e| debug!("{e:#?}"))?,
        );

        Ok(client)
    }

    pub async fn secure_my_connection(&mut self) -> Result<bool> {
        Ok(self.get_client().await?.connect_tunnel().await?)
    }

    pub async fn disconnect(&mut self) -> Result<bool> {
        Ok(self.get_client().await?.disconnect_tunnel().await?)
    }

    pub async fn reconnect(&mut self) -> Result<bool> {
        Ok(self.get_client().await?.reconnect_tunnel().await?)
    }
}
