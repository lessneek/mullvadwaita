use mullvad_types::states::TunnelState;
use talpid_types::tunnel::ActionAfterDisconnect;
use TunnelState::*;

pub trait TunnelStateExt {
    fn is_connecting_or_connected(&self) -> bool;
    fn is_connecting_or_reconnecting(&self) -> bool;
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
}
