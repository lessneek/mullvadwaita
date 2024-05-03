use mullvad_types::relay_constraints::Constraint;
use talpid_types::net::TunnelType;
use tr::tr;

pub enum ViewType<E> {
    Label(String),
    Entry(String, Box<dyn Fn(String) -> Option<E>>),
}

pub trait ViewElement: Sized {
    fn get_view_type(&self) -> ViewType<Self>;
}

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

impl ViewElement for TunnelProtocol {
    fn get_view_type(&self) -> ViewType<Self> {
        use TunnelProtocol::*;
        match self {
            Automatic => ViewType::Label(tr!("Automatic")),
            WireGuard => ViewType::Label(tr!("WireGuard")),
            OpenVPN => ViewType::Label(tr!("OpenVPN")),
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

