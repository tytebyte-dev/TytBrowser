// Module: core\mod.rs

pub mod annotations;
pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod context;
pub mod core;
pub mod format;
pub mod functions;
pub mod mocks;
pub mod proto;
pub mod xray;

#[cfg(test)]
pub mod context_test;
#[cfg(test)]
pub mod functions_test;
#[cfg(test)]
pub mod xray_test;

pub use annotations::{Annotation, ApiStability};
pub use config::{
    ConfigFormat, ConfigLoader, ConfigSource, CoreConfig, get_extension, get_format,
    get_format_by_extension, get_merged_config, load_config, register_config_loader,
};
pub use context::{
    CoreContext, from_context, must_from_context, to_background_detached_context, to_context,
};
pub use core::{
    BUILD, CODENAME, INTRO, Instance, Server, VERSION_X, VERSION_Y, VERSION_Z, version,
    version_statement,
};
pub use functions::{create_object, dial, dial_udp, start_instance};
pub use mocks::create_mock_instance;
pub use proto::PROTO_CORE_VERSION;
pub use xray::server_type;
