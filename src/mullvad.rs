use crate::prelude::*;

use std::time::Duration;

use anyhow::{Ok, Result};

use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::{
    account::AccountData, device::DeviceState, settings::Settings, states::TunnelState,
};
use smart_default::SmartDefault;
use tokio::sync::mpsc::{self, Receiver, Sender};

use futures::StreamExt;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Event {
    TunnelState(TunnelState),
    DeviceState(DeviceState),
    AccountData(AccountData),
    Setting(Settings),
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
    let mut client = MullvadProxyClient::new().await?;

    let state = client.get_tunnel_state().await?;
    sender.send(Event::TunnelState(state)).await?;

    let device = client.get_device().await?;

    if let Some(account_token) = device.get_account().map(|acc| acc.account_token.clone()) {
        let account_data = client.get_account_data(account_token.clone()).await?;
        sender.send(Event::AccountData(account_data)).await?;
        sender.send(Event::DeviceState(device)).await?;
    }

    let settings = client.get_settings().await?;
    sender.send(Event::Setting(settings)).await?;

    while let Some(event) = client.events_listen().await?.next().await {
        match event? {
            DaemonEvent::TunnelState(new_state) => {
                trace!("New tunnel state: {new_state:#?}");
                sender.send(Event::TunnelState(new_state)).await?;
            }
            DaemonEvent::Settings(settings) => {
                trace!("New settings: {settings:#?}");
                sender.send(Event::Setting(settings)).await?;
            }
            DaemonEvent::RelayList(relay_list) => {
                trace!("New relay list: {relay_list:#?}");
            }
            DaemonEvent::AppVersionInfo(app_version_info) => {
                trace!("New app version info: {app_version_info:#?}");
            }
            DaemonEvent::Device(device) => {
                trace!("Device event: {device:#?}");
                sender.send(Event::DeviceState(device.new_state)).await?
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

    #[allow(dead_code)]
    pub async fn get_account_data(&mut self, account: String) -> Result<AccountData> {
        Ok(self.get_client().await?.get_account_data(account).await?)
    }

    #[allow(dead_code)]
    pub async fn get_settings(&mut self) -> Result<Settings> {
        Ok(self.get_client().await?.get_settings().await?)
    }

    pub async fn set_auto_connect(&mut self, state: bool) -> Result<()> {
        Ok(self.get_client().await?.set_auto_connect(state).await?)
    }

    pub async fn set_allow_lan(&mut self, state: bool) -> Result<()> {
        Ok(self.get_client().await?.set_allow_lan(state).await?)
    }

    pub async fn set_block_when_disconnected(&mut self, state: bool) -> Result<()> {
        Ok(self.get_client().await?.set_block_when_disconnected(state).await?)
    }

    pub async fn set_enable_ipv6(&mut self, state: bool) -> Result<()> {
        Ok(self.get_client().await?.set_enable_ipv6(state).await?)
    }
}
