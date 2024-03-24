use mullvad_types::{location::GeoIpLocation, states::TunnelState};
use talpid_types::{net::TunnelEndpoint, tunnel::ActionAfterDisconnect};
use TunnelState::*;

pub trait TunnelStateExt {
    fn is_connecting_or_connected(&self) -> bool;
    fn is_connecting_or_reconnecting(&self) -> bool;
    fn get_endpoint(&self) -> Option<&TunnelEndpoint>;
    fn get_location(&self) -> Option<&GeoIpLocation>;
    fn get_country(&self) -> String;
    fn get_city(&self) -> String;
}

impl TunnelStateExt for TunnelState {
    fn is_connecting_or_connected(&self) -> bool {
        self.is_connected() || self.is_connecting_or_reconnecting()
    }

    fn is_connecting_or_reconnecting(&self) -> bool {
        matches!(
            self,
            Connecting { .. } | Disconnecting(ActionAfterDisconnect::Reconnect)
        )
    }

    fn get_location(&self) -> Option<&GeoIpLocation> {
        match self {
            Disconnected { location, .. }
            | Connecting { location, .. }
            | Connected { location, .. } => location.as_ref(),
            _ => None,
        }
    }

    fn get_endpoint(&self) -> Option<&TunnelEndpoint> {
        match self {
            Connecting { endpoint, .. } | Connected { endpoint, .. } => Some(endpoint),
            _ => None,
        }
    }

    fn get_country(&self) -> String {
        if let Some(location) = self.get_location() {
            return location.country.clone();
        }
        String::new()
    }

    fn get_city(&self) -> String {
        if let Some(location) = self.get_location() {
            if let Some(city) = &location.city {
                return city.clone();
            }
        }
        String::new()
    }
}
