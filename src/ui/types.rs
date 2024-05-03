use mullvad_types::relay_constraints::Constraint;
use talpid_types::net::TunnelType;
use tr::tr;

use crate::ui::radio_buttons_list::VariantType;

use super::radio_buttons_list::RadioButtonsListVariant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TunnelProtocol {
    Automatic,
    WireGuard,
    OpenVPN,
}

impl TunnelProtocol {
    pub fn get_all_variants() -> Vec<TunnelProtocol> {
        use TunnelProtocol::*;
        vec![Automatic, WireGuard, OpenVPN]
    }
}

impl RadioButtonsListVariant for TunnelProtocol {
    fn get_variant_type(&self) -> VariantType<Self> {
        use TunnelProtocol::*;
        match self {
            Automatic => VariantType::Label(tr!("Automatic")),
            WireGuard => VariantType::Label(tr!("WireGuard")),
            OpenVPN => VariantType::Label(tr!("OpenVPN")),
        }
    }
}

impl From<Constraint<TunnelType>> for TunnelProtocol {
    fn from(value: Constraint<TunnelType>) -> Self {
        match value {
            Constraint::Any => TunnelProtocol::Automatic,
            Constraint::Only(TunnelType::Wireguard) => TunnelProtocol::WireGuard,
            Constraint::Only(TunnelType::OpenVpn) => TunnelProtocol::OpenVPN,
        }
    }
}

impl From<TunnelProtocol> for Constraint<TunnelType> {
    fn from(val: TunnelProtocol) -> Self {
        match val {
            TunnelProtocol::Automatic => Constraint::Any,
            TunnelProtocol::WireGuard => Constraint::Only(TunnelType::Wireguard),
            TunnelProtocol::OpenVPN => Constraint::Only(TunnelType::OpenVpn),
        }
    }
}

