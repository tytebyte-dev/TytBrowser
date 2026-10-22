pub mod api;
pub mod balancer_info;
pub mod balancer_override;
pub mod inbound_user;
pub mod inbound_user_add;
pub mod inbound_user_count;
pub mod inbound_user_remove;
pub mod inbounds_add;
pub mod inbounds_list;
pub mod inbounds_remove;
pub mod logger_restart;
pub mod outbounds_add;
pub mod outbounds_list;
pub mod outbounds_remove;
pub mod rules_add;
pub mod rules_list;
pub mod rules_remove;
pub mod shared;
pub mod source_ip_block;
pub mod stats_get;
pub mod stats_get_all_online_users;
pub mod stats_online;
pub mod stats_online_ip_list;
pub mod stats_query;
pub mod stats_sys;

pub use api::cmd_api;
pub use shared::ApiClientConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_command_hierarchy() {
        let api_cmd = cmd_api();
        assert_eq!(api_cmd.name, "api");
        assert_eq!(api_cmd.subcommands.len(), 23);

        // Test running stats query through subcommand dispatch
        let out = api_cmd
            .execute(&["query-stats", "-s=127.0.0.1:10086"])
            .unwrap();
        assert!(out.contains("127.0.0.1:10086"));

        // Test client config parsing
        let cfg = ApiClientConfig::parse(&["-s=10.0.0.1:8080"]);
        assert_eq!(cfg.server, "10.0.0.1:8080");
        assert_eq!(cfg.timeout, 5);
    }
}
