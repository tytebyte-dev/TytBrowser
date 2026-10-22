pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod handler;
pub mod net;
pub mod platform;
pub mod routes;
pub mod runner;
pub mod socket_helpers;
pub mod stack;
pub mod stack_gvisor;
pub mod stack_gvisor_endpoint;
pub mod tun;
pub mod tun_android;
pub mod tun_darwin;
pub mod tun_default;
pub mod tun_linux;
pub mod tun_windows;
pub mod udp_fullcone;

pub use config::TunConfig;
pub use handler::TunHandler;
pub use net::{
    DEFAULT_OUTBOUND_INTERFACE, OutboundInterface, get_outbound_interface, init_net_config,
};
pub use runner::TunRunner;
pub use runner::TunRunner as WindowsTunDevice;
pub use stack::TunStack;
pub use stack_gvisor::GVisorStack;
pub use stack_gvisor_endpoint::GVisorEndpoint;
pub use tun::{DEFAULT_TUN_MTU, PROTOCOL_NAME};
pub use udp_fullcone::UdpNatTable;
