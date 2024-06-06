use std::str::FromStr;

use mullvad_types::constraints::Constraint;
use talpid_types::net::TunnelType;
use tr::tr;

use crate::{
    if_let_map,
    ui::variant_selector::{entry_variant, label_variant, EntryConverter},
};

use super::variant_selector::{Unique, Variant, VariantValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TunnelProtocol {
    Automatic,
    WireGuard,
    OpenVPN,
}

impl TunnelProtocol {
    pub fn get_all_variants() -> Vec<Variant<Self>> {
        use TunnelProtocol::*;
        vec![
            label_variant(Automatic, tr!("Automatic")),
            label_variant(WireGuard, tr!("WireGuard")),
            label_variant(OpenVPN, tr!("OpenVPN")),
        ]
    }
}

impl VariantValue for TunnelProtocol {}

impl Unique for TunnelProtocol {
    type Id = usize;

    fn get_id(&self) -> Self::Id {
        use TunnelProtocol::*;
        match self {
            Automatic => 0,
            WireGuard => 1,
            OpenVPN => 2,
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

pub const ALLOWED_WIRE_GUARD_PORTS: &str = "53, 123, 443, 4000-33433, 33565-51820, 52001-60000";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WireGuardPort {
    Automatic,
    Port51820,
    Port53,
    Custom(u16),
}

impl WireGuardPort {
    pub fn get_all_variants() -> Vec<Variant<Self>> {
        use WireGuardPort::*;
        vec![
            label_variant(Automatic, tr!("Automatic")),
            label_variant(Port51820, tr!("51820")),
            label_variant(Port53, tr!("53")),
            entry_variant(
                Custom(123),
                tr!("Custom"),
                tr!("Custom WireGuard port"),
                EntryConverter::new(
                    Box::new(|s| s.parse::<WireGuardPort>()),
                    Box::new(|port| if_let_map!(port to Custom(port) => port.to_string())),
                ),
            ),
        ]
    }
}

impl VariantValue for WireGuardPort {}

impl Unique for WireGuardPort {
    type Id = u8;

    fn get_id(&self) -> Self::Id {
        use WireGuardPort::*;
        match self {
            Automatic => 0,
            Port51820 => 1,
            Port53 => 2,
            Custom(_) => 3,
        }
    }
}

impl FromStr for WireGuardPort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u16>()
            .ok()
            .filter(|port| matches!(port, 53 | 123 | 443 | 4000..=33433 | 33565..=51820 | 52001..=60000))
            .map(|port| {
                WireGuardPort::Custom(port)
            })
            .ok_or(tr!(
                "A WireGuard port must be any value inside the valid ranges: {}.",
                ALLOWED_WIRE_GUARD_PORTS
            ))
    }
}

impl From<WireGuardPort> for Constraint<u16> {
    fn from(value: WireGuardPort) -> Self {
        match value {
            WireGuardPort::Automatic => Constraint::Any,
            WireGuardPort::Port51820 => Constraint::Only(51820),
            WireGuardPort::Port53 => Constraint::Only(53),
            WireGuardPort::Custom(custom_port) => Constraint::Only(custom_port),
        }
    }
}

impl From<Constraint<u16>> for WireGuardPort {
    fn from(value: Constraint<u16>) -> Self {
        match value {
            Constraint::Any => WireGuardPort::Automatic,
            Constraint::Only(port) => match port {
                51820 => WireGuardPort::Port51820,
                53 => WireGuardPort::Port53,
                _ => WireGuardPort::Custom(port),
            },
        }
    }
}
