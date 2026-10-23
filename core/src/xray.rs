//! Встроенный прокси на базе xray-rs: ядро Xray работает внутри приложения и слушает 127.0.0.1:порт.

use crate::proxylink;
use std::net::{SocketAddr, TcpStream};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

const LOG_CAP: usize = 64 * 1024;
static LOG: Mutex<String> = Mutex::new(String::new());

/// Писатель журнала tracing: складывает строки в кольцевой буфер, чтобы их можно было показать в приложении.
#[derive(Clone, Copy)]
struct LogWriter;

impl std::io::Write for LogWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        let mut g = LOG.lock().unwrap_or_else(|e| e.into_inner());
        g.push_str(&String::from_utf8_lossy(b));
        if g.len() > LOG_CAP {
            let mut cut = g.len() - LOG_CAP / 2;
            while !g.is_char_boundary(cut) {
                cut += 1;
            }
            g.drain(..cut);
        }
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogWriter {
    type Writer = LogWriter;
    fn make_writer(&'a self) -> Self::Writer {
        *self
    }
}

fn init_log() {
    let _ = tracing_subscriber::fmt()
        .with_writer(LogWriter)
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

/// Журнал ядра прокси (последние сообщения).
pub fn log_text() -> String {
    LOG.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

struct Running {
    rt: Runtime,
    handles: Vec<JoinHandle<()>>,
    port: u16,
}

static STATE: Mutex<Option<Running>> = Mutex::new(None);

fn lock() -> std::sync::MutexGuard<'static, Option<Running>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Запускает прокси по ссылке (vless/vmess/trojan/ss). Уже работающий экземпляр перезапускается.
pub fn start(link: &str, port: u16) -> Result<(), String> {
    let cfg = proxylink::build_config(link, port)?;
    init_log();
    stop();
    LOG.lock().unwrap_or_else(|e| e.into_inner()).clear();
    log_line(&format!("запуск прокси на 127.0.0.1:{port}"));
    let rt = Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("tyt-xray")
        .enable_all()
        .build()
        .map_err(|e| format!("runtime: {e}"))?;
    let started = rt.block_on(async {
        let inst = xray_core::core::functions::start_instance("json", cfg.as_bytes())
            .map_err(|e| format!("{e}"))?;
        let handles = inst.start().await.map_err(|e| format!("{e}"))?;
        // Экземпляр должен жить, пока работают обработчики: переносим его в фоновую задачу.
        let keep = tokio::spawn(async move {
            let _keep = inst;
            std::future::pending::<()>().await;
        });
        let mut all = handles;
        all.push(keep);
        Ok::<_, String>(all)
    });
    let handles = match started {
        Ok(h) => h,
        Err(e) => {
            log_line(&format!("ошибка запуска: {e}"));
            rt.shutdown_background();
            return Err(e);
        }
    };
    // Ждём, пока локальный порт реально начнёт принимать соединения.
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
            break;
        }
        if Instant::now() > deadline {
            for h in &handles {
                h.abort();
            }
            rt.shutdown_background();
            return Err("прокси не открыл локальный порт".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    *lock() = Some(Running { rt, handles, port });
    Ok(())
}

pub fn stop() {
    if let Some(r) = lock().take() {
        for h in &r.handles {
            h.abort();
        }
        r.rt.shutdown_background();
    }
}

/// Порт работающего прокси или 0.
pub fn running_port() -> u16 {
    lock().as_ref().map(|r| r.port).unwrap_or(0)
}

fn log_line(s: &str) {
    use std::io::Write;
    let _ = writeln!(LogWriter, "[tytcore] {s}");
}
