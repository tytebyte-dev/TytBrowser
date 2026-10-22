// Module: common\log\logger.rs
// 1:1 Rust implementation corresponding to Go common\log\logger.go

use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::log::{Handler, LogLevel, Message, register_handler};
use super::log_pb::Severity;
use crate::common::platform;

/// Writer is the interface for writing logs.
pub trait Writer: Send + Sync {
    fn write_str(&mut self, s: &str) -> std::io::Result<()>;
    fn close(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// WriterCreator is a factory function to create Writers.
pub type WriterCreator = Arc<dyn Fn() -> Option<Box<dyn Writer>> + Send + Sync>;

struct StdoutWriter;
impl Writer for StdoutWriter {
    fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(s.as_bytes())?;
        handle.flush()
    }
}

struct StderrWriter;
impl Writer for StderrWriter {
    fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        handle.write_all(s.as_bytes())?;
        handle.flush()
    }
}

struct FileWriter {
    file: std::fs::File,
}
impl Writer for FileWriter {
    fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.file.write_all(s.as_bytes())?;
        self.file.flush()
    }
    fn close(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/// CreateStdoutLogWriter returns a WriterCreator that creates LogWriter for stdout.
pub fn create_stdout_log_writer() -> WriterCreator {
    Arc::new(|| Some(Box::new(StdoutWriter)))
}

/// CreateStderrLogWriter returns a WriterCreator that creates LogWriter for stderr.
pub fn create_stderr_log_writer() -> WriterCreator {
    Arc::new(|| Some(Box::new(StderrWriter)))
}

/// CreateFileLogWriter returns a WriterCreator that creates LogWriter for the given file path.
pub fn create_file_log_writer(path: &str) -> std::io::Result<WriterCreator> {
    // Test open / create with append permissions
    let _ = OpenOptions::new().create(true).append(true).open(path)?;

    let path_buf = path.to_string();
    Ok(Arc::new(move || {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path_buf)
            .ok()
            .map(|f| Box::new(FileWriter { file: f }) as Box<dyn Writer>)
    }))
}

/// GeneralLogger handles log messages asynchronously through a bounded queue.
pub struct GeneralLogger {
    sender: SyncSender<String>,
    done: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl GeneralLogger {
    pub fn new(creator: WriterCreator) -> Arc<Self> {
        let (tx, rx) = sync_channel::<String>(128);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();

        let worker = thread::spawn(move || {
            let mut writer = match creator() {
                Some(w) => w,
                None => return,
            };

            let rx: Receiver<String> = rx;
            loop {
                match rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(msg) => {
                        let line = format!("{}{}", msg, platform::line_separator());
                        let _ = writer.write_str(&line);
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if done_clone.load(Ordering::SeqCst) {
                            // Drain remaining messages before exiting
                            while let Ok(msg) = rx.try_recv() {
                                let line = format!("{}{}", msg, platform::line_separator());
                                let _ = writer.write_str(&line);
                            }
                            let _ = writer.close();
                            return;
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        let _ = writer.close();
                        return;
                    }
                }
            }
        });

        Arc::new(Self {
            sender: tx,
            done,
            worker: Mutex::new(Some(worker)),
        })
    }

    pub fn close(&self) {
        self.done.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.worker.lock()
            && let Some(handle) = guard.take()
        {
            let _ = handle.join();
        }
    }
}

impl Handler for GeneralLogger {
    fn handle(&self, msg: &dyn Message) {
        let _ = self.sender.try_send(msg.to_log_string());
    }
}

impl Drop for GeneralLogger {
    fn drop(&mut self) {
        self.close();
    }
}

/// NewLogger returns a generic log handler that can handle all types of messages.
pub fn new_logger(creator: WriterCreator) -> Arc<GeneralLogger> {
    GeneralLogger::new(creator)
}

/// SeverityLogger wraps a GeneralLogger with level filtering.
pub struct SeverityLogger {
    inner: Arc<GeneralLogger>,
    log_level: Severity,
}

impl SeverityLogger {
    pub fn new(inner: Arc<GeneralLogger>, log_level: Severity) -> Self {
        Self { inner, log_level }
    }
}

impl Handler for SeverityLogger {
    fn handle(&self, msg: &dyn Message) {
        if let Some(sev) = msg.severity() {
            if sev.number() <= self.log_level.number() {
                self.inner.handle(msg);
            }
        } else {
            self.inner.handle(msg);
        }
    }
}

/// ReplaceWithSeverityLogger configures stdout logging at the given severity.
pub fn replace_with_severity_logger(severity: Severity) {
    let general = new_logger(create_stdout_log_writer());
    let severity_logger = SeverityLogger::new(general, severity);
    register_handler(Arc::new(severity_logger));
}

/// Legacy Logger struct for backward compatibility
pub struct Logger {
    level: AtomicUsize,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level: AtomicUsize::new(level.number() as usize),
        }
    }

    pub fn set_level(&self, level: LogLevel) {
        self.level.store(level.number() as usize, Ordering::SeqCst);
    }

    pub fn is_enabled(&self, level: LogLevel) -> bool {
        (level.number() as usize) <= self.level.load(Ordering::Relaxed)
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        if self.is_enabled(level) {
            println!("[{}] {}", level.as_str(), msg);
        }
    }
}
