use mullvad_types::states::TunnelState;

pub trait TunnelStateExt {
    fn is_connecting_or_connected(&self) -> bool;
}

impl TunnelStateExt for TunnelState {
    fn is_connecting_or_connected(&self) -> bool {
        use TunnelState::*;
        matches!(self, Connected { .. } | Connecting { .. })
    }
}
