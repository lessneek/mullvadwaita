use std::time::Duration;

use anyhow::Result;

use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::{
    access_method::AccessMethodSetting,
    account::{AccountData, AccountToken},
    device::{DeviceEvent, DeviceEventCause, DeviceState, RemoveDeviceEvent},
    relay_constraints::RelaySettings,
    relay_list::RelayList,
    settings::Settings,
    states::TunnelState,
    version::AppVersionInfo,
};
use smart_default::SmartDefault;
use tokio::sync::mpsc::{self, Receiver, Sender};

use futures::StreamExt;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
pub enum Event {
    TunnelState(TunnelState),
    Setting(Settings),
    RelayList(RelayList),
    AppVersionInfo(AppVersionInfo),
    Device(DeviceEvent),
    RemoveDevice(RemoveDeviceEvent),
    AccountData(AccountData),
    NewAccessMethod(AccessMethodSetting),
    ConnectingToDaemon,
}

pub fn events_receiver() -> Receiver<Event> {
    let (sender, receiver) = mpsc::channel(10);

    tokio::spawn(async move {
        while !sender.is_closed() && (sender.send(Event::ConnectingToDaemon).await).is_ok() {
            log::trace!("Starting listening for RPC.");
            match events_listen(&sender).await {
                Result::Ok(_) => log::info!("RPC listening ended Ok."),
                Err(err) => log::debug!("RPC listening error: {}", err),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    receiver
}

async fn events_listen(sender: &Sender<Event>) -> Result<()> {
    let mut client = MullvadProxyClient::new().await?;

    let settings = client.get_settings().await?;
    sender.send(Event::Setting(settings)).await?;

    let state = client.get_tunnel_state().await?;
    sender.send(Event::TunnelState(state)).await?;

    if let Ok(device) = client.get_device().await {
        sender
            .send(Event::Device(DeviceEvent {
                cause: DeviceEventCause::Updated,
                new_state: device,
            }))
            .await?;
    }

    while let Some(event) = client.events_listen().await?.next().await {
        match event? {
            DaemonEvent::TunnelState(new_state) => {
                log::trace!("{new_state:#?}");
                sender.send(Event::TunnelState(new_state)).await?;
            }
            DaemonEvent::Settings(settings) => {
                log::trace!("{settings:#?}");
                sender.send(Event::Setting(settings)).await?;
            }
            DaemonEvent::RelayList(relay_list) => {
                log::trace!("{relay_list:#?}");
                sender.send(Event::RelayList(relay_list)).await?;
            }
            DaemonEvent::AppVersionInfo(app_version_info) => {
                log::trace!("{app_version_info:#?}");
                sender.send(Event::AppVersionInfo(app_version_info)).await?;
            }
            DaemonEvent::Device(device_event) => {
                log::trace!("{device_event:#?}");
                sender.send(Event::Device(device_event)).await?;
            }
            DaemonEvent::RemoveDevice(remove_device_event) => {
                log::trace!("{remove_device_event:#?}");
                sender
                    .send(Event::RemoveDevice(remove_device_event))
                    .await?;
            }
            DaemonEvent::NewAccessMethod(access_method) => {
                log::trace!("{access_method:#?}");
                sender.send(Event::NewAccessMethod(access_method)).await?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, SmartDefault, Clone)]
pub struct DaemonConnector {
    client: Option<MullvadProxyClient>,
}

#[allow(dead_code)]
impl DaemonConnector {
    async fn get_client(&mut self) -> Result<&mut MullvadProxyClient> {
        let client = &mut self.client;

        let client = client.get_or_insert(
            MullvadProxyClient::new()
                .await
                .inspect_err(|e| log::debug!("{e:#?}"))?,
        );

        Ok(client)
    }

    pub async fn login_account(&mut self, account: AccountToken) -> Result<()> {
        Ok(self.get_client().await?.login_account(account).await?)
    }

    pub async fn logout_account(&mut self) -> Result<()> {
        Ok(self.get_client().await?.logout_account().await?)
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

    pub async fn get_account_data(&mut self, account: String) -> Result<AccountData> {
        Ok(self.get_client().await?.get_account_data(account).await?)
    }

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
        Ok(self
            .get_client()
            .await?
            .set_block_when_disconnected(state)
            .await?)
    }

    pub async fn set_enable_ipv6(&mut self, state: bool) -> Result<()> {
        Ok(self.get_client().await?.set_enable_ipv6(state).await?)
    }

    pub async fn get_tunnel_state(&mut self) -> Result<TunnelState> {
        Ok(self.get_client().await?.get_tunnel_state().await?)
    }

    pub async fn get_device(&mut self) -> Result<DeviceState> {
        Ok(self.get_client().await?.get_device().await?)
    }

    pub async fn get_account_history(&mut self) -> Result<Option<AccountToken>> {
        Ok(self.get_client().await?.get_account_history().await?)
    }

    pub async fn clear_account_history(&mut self) -> Result<()> {
        Ok(self.get_client().await?.clear_account_history().await?)
    }

    pub async fn create_new_account(&mut self) -> Result<AccountToken> {
        Ok(self.get_client().await?.create_new_account().await?)
    }

    pub async fn set_relay_settings(&mut self, update: RelaySettings) -> Result<()> {
        Ok(self.get_client().await?.set_relay_settings(update).await?)
    }
}
