// Module: app\policy\config.rs
// 1:1 Rust implementation corresponding to Go app\policy\config.go

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::config_pb::{Policy, PolicyBuffer, PolicyTimeout, Second, SystemPolicy};
use crate::features::policy::{
    Buffer as CoreBuffer, SessionPolicy as CoreSessionPolicy, SystemPolicy as CoreSystemPolicy,
    SystemStats as CoreSystemStats, session_default,
};

impl Second {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.value as u64)
    }
}

pub fn default_policy() -> Policy {
    let p = session_default();
    Policy {
        timeout: Some(PolicyTimeout {
            handshake: Some(Second {
                value: p.timeouts.handshake.as_secs() as u32,
            }),
            connection_idle: Some(Second {
                value: p.timeouts.connection_idle.as_secs() as u32,
            }),
            uplink_only: Some(Second {
                value: p.timeouts.uplink_only.as_secs() as u32,
            }),
            downlink_only: Some(Second {
                value: p.timeouts.downlink_only.as_secs() as u32,
            }),
        }),
        stats: None,
        buffer: Some(PolicyBuffer {
            connection: p.buffer.per_connection,
        }),
    }
}

impl PolicyTimeout {
    pub fn override_with(&mut self, another: &PolicyTimeout) {
        if let Some(h) = &another.handshake {
            self.handshake = Some(Second { value: h.value });
        }
        if let Some(ci) = &another.connection_idle {
            self.connection_idle = Some(Second { value: ci.value });
        }
        if let Some(uo) = &another.uplink_only {
            self.uplink_only = Some(Second { value: uo.value });
        }
        if let Some(do_only) = &another.downlink_only {
            self.downlink_only = Some(Second {
                value: do_only.value,
            });
        }
    }
}

impl Policy {
    pub fn override_with(&mut self, another: &Policy) {
        if let Some(another_timeout) = &another.timeout {
            if let Some(timeout) = &mut self.timeout {
                timeout.override_with(another_timeout);
            } else {
                self.timeout = Some(another_timeout.clone());
            }
        }
        if let Some(another_stats) = &another.stats {
            self.stats = Some(another_stats.clone());
        }
        if let Some(another_buffer) = &another.buffer {
            self.buffer = Some(PolicyBuffer {
                connection: another_buffer.connection,
            });
        }
    }

    pub fn to_core_policy(&self) -> CoreSessionPolicy {
        let mut cp = session_default();

        if let Some(t) = &self.timeout {
            if let Some(ci) = &t.connection_idle {
                cp.timeouts.connection_idle = ci.duration();
            }
            if let Some(h) = &t.handshake {
                cp.timeouts.handshake = h.duration();
            }
            if let Some(dl) = &t.downlink_only {
                cp.timeouts.downlink_only = dl.duration();
            }
            if let Some(ul) = &t.uplink_only {
                cp.timeouts.uplink_only = ul.duration();
            }
        }

        if let Some(s) = &self.stats {
            cp.stats.user_uplink = s.user_uplink;
            cp.stats.user_downlink = s.user_downlink;
            cp.stats.user_online = s.user_online;
        }

        if let Some(b) = &self.buffer {
            cp.buffer.per_connection = b.connection;
        }

        cp
    }
}

impl SystemPolicy {
    pub fn to_core_policy(&self) -> CoreSystemPolicy {
        let stats = match &self.stats {
            Some(s) => CoreSystemStats {
                inbound_uplink: s.inbound_uplink,
                inbound_downlink: s.inbound_downlink,
                outbound_uplink: s.outbound_uplink,
                outbound_downlink: s.outbound_downlink,
            },
            None => CoreSystemStats::default(),
        };

        CoreSystemPolicy {
            stats,
            buffer: CoreBuffer::default(),
        }
    }
}

// Legacy struct for backward compatibility
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyLevelConfig {
    #[serde(default)]
    pub handshake_timeout: Option<u32>,
    #[serde(default)]
    pub conn_idle: Option<u32>,
    #[serde(default)]
    pub uplink_only: Option<u32>,
    #[serde(default)]
    pub downlink_only: Option<u32>,
    #[serde(default)]
    pub stats_user_uplink: bool,
    #[serde(default)]
    pub stats_user_downlink: bool,
    #[serde(default)]
    pub buffer_size: Option<i32>,
}
