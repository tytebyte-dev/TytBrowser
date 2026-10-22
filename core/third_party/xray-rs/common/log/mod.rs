pub mod access;
pub mod dns;
pub mod log;
pub mod logger;

#[path = "log.pb.rs"]
pub mod log_pb;

#[cfg(test)]
pub mod log_test;
#[cfg(test)]
pub mod logger_test;

pub use access::{
    ACCESS_ACCEPTED, ACCESS_REJECTED, AccessLogMessage, AccessMessage, AccessStatus,
    access_message_from_context, context_with_access_message,
};
pub use dns::{DNS_CACHE_HIT, DNS_CACHE_OPTIMISTE, DNS_QUERIED, DnsLog, DnsStatus, format_dns_log};
pub use log::{
    GeneralMessage, Handler, LogLevel, Message, SyncHandler, get_log_handler, record,
    register_handler,
};
pub use log_pb::Severity;
pub use logger::{
    GeneralLogger, Logger, SeverityLogger, Writer, WriterCreator, create_file_log_writer,
    create_stderr_log_writer, create_stdout_log_writer, new_logger, replace_with_severity_logger,
};
