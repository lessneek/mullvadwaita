use crate::tr;
use mullvad_types::{
    location::GeoIpLocation,
    states::TunnelState::{self, *},
};
use std::fmt::Write;
use talpid_types::{net::TunnelEndpoint, tunnel::ActionAfterDisconnect};

pub trait TunnelStateExt {
    fn is_connecting_or_connected(&self) -> bool;
    fn is_connecting_or_reconnecting(&self) -> bool;
    fn get_endpoint(&self) -> Option<&TunnelEndpoint>;
    fn get_location(&self) -> Option<&GeoIpLocation>;
    fn get_tunnel_state_label(&self) -> String;
    fn get_country(&self) -> Option<String>;
    fn get_city(&self) -> Option<String>;
    fn get_hostname(&self) -> Option<String>;
    fn get_tunnel_protocol(&self) -> Option<String>;
    fn get_tunnel_in(&self) -> Option<String>;
    fn get_tunnel_out(&self) -> Option<String>;
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

    fn get_tunnel_state_label(&self) -> String {
        use TunnelState::*;
        match self {
            Connected { endpoint, .. } => {
                if endpoint.quantum_resistant {
                    tr!(
                        // Creating a secure connection that isn't breakable by quantum computers.
                        "QUANTUM SECURE CONNECTION"
                    )
                } else {
                    tr!("SECURE CONNECTION")
                }
            }
            Connecting { endpoint, .. } => {
                if endpoint.quantum_resistant {
                    tr!("CREATING QUANTUM SECURE CONNECTION")
                } else {
                    tr!("CREATING SECURE CONNECTION")
                }
            }
            Disconnected { locked_down, .. } => {
                if *locked_down {
                    tr!("BLOCKED CONNECTION")
                } else {
                    tr!("UNSECURED CONNECTION")
                }
            }
            Disconnecting(ActionAfterDisconnect::Nothing | ActionAfterDisconnect::Block) => {
                tr!("DISCONNECTING")
            }
            Disconnecting(ActionAfterDisconnect::Reconnect) => {
                tr!("CREATING SECURE CONNECTION")
            }
            Error(error_state) => {
                if error_state.is_blocking() {
                    tr!("BLOCKED CONNECTION")
                } else {
                    tr!("UNSECURED CONNECTION")
                }
            }
        }
    }

    fn get_country(&self) -> Option<String> {
        self.get_location().map(|location| location.country.clone())
    }

    fn get_city(&self) -> Option<String> {
        self.get_location()
            .and_then(|location| location.city.clone())
    }

    fn get_hostname(&self) -> Option<String> {
        self.get_location().and_then(|location| {
            location.hostname.as_ref().map(|new_hostname| {
                let mut hostname = String::new();
                hostname.push_str(new_hostname);
                if let Some(via) = location
                    .bridge_hostname
                    .as_ref()
                    .or(location.obfuscator_hostname.as_ref())
                    .or(location.entry_hostname.as_ref())
                {
                    if via != new_hostname {
                        let _ = write!(hostname, " via {via}");
                    }
                }
                hostname
            })
        })
    }

    fn get_tunnel_protocol(&self) -> Option<String> {
        self.get_endpoint().map(|te| {
            let mut tp = te.tunnel_type.to_string();
            if let Some(proxy) = te.proxy {
                let _ = write!(&mut tp, " via {}", proxy.proxy_type);
            } else if let Some(obf) = te.obfuscation {
                let _ = write!(&mut tp, " via {}", obf.obfuscation_type);
            }
            tp
        })
    }

    fn get_tunnel_in(&self) -> Option<String> {
        self.get_endpoint().and_then(|te| {
            te.proxy
                .map(|pep| pep.endpoint.to_string())
                .or(te.obfuscation.map(|oep| oep.endpoint.to_string()))
                .or(te.entry_endpoint.map(|eep| eep.to_string()))
                .or(Some(te.endpoint.to_string()))
        })
    }

    fn get_tunnel_out(&self) -> Option<String> {
        self.get_location().map(|loc| {
            let mut out = String::new();
            if let Some(ipv4) = loc.ipv4 {
                let _ = write!(&mut out, "{}", ipv4).ok();
            }
            if let Some(ipv6) = loc.ipv6 {
                if !out.is_empty() {
                    out.push('\n');
                }
                let _ = write!(&mut out, "{}", ipv6).ok();
            }
            if out.is_empty() {
                out.push_str("...");
            }
            out
        })
    }
}

pub(crate) trait ToStr {
    fn to_str(&self) -> &str;
}

impl ToStr for Option<String> {
    fn to_str(self: &Option<String>) -> &str {
        self.as_ref().map(|ss| ss.as_str()).unwrap_or_default()
    }
}
