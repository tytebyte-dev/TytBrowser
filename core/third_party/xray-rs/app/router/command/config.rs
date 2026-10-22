use crate::common::net::{Address, Destination, Network};
use crate::common::protocol::{SessionContext, User};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct CommandRoutingContext {
    pub inbound_tag: String,
    pub source_ip: Option<IpAddr>,
    pub source_port: u16,
    pub target_ip: Option<IpAddr>,
    pub target_domain: Option<String>,
    pub target_port: u16,
    pub network: Network,
    pub user: Option<String>,
}

impl CommandRoutingContext {
    pub fn to_session_context(&self) -> SessionContext {
        let addr = if let Some(domain) = &self.target_domain {
            Address::Domain(domain.clone())
        } else if let Some(ip) = self.target_ip {
            Address::from(ip)
        } else {
            Address::Domain("localhost".into())
        };

        let dest = Destination::new(addr, self.target_port);
        let mut session = SessionContext::new(&self.inbound_tag, dest);
        if let Some(user_email) = &self.user {
            session.user = Some(User {
                id: Uuid::nil(),
                email: user_email.clone(),
                level: 0,
            });
        }
        session
    }
}
