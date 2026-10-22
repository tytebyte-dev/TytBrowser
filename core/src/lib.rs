//! tytcore — нативное ядро TytBrowser.
//!
//! Kotlin-часть отвечает за интерфейс и WebView, а всё «думающее» живёт здесь:
//!   * [`omnibox`]  — что ввёл пользователь: адрес, поиск или внутренняя страница; чистка ссылок;
//!   * [`blocker`]  — блокировка рекламы и трекеров на уровне запросов WebView;
//!   * [`reader`]   — режим чтения (выделение статьи из HTML);
//!   * [`suggest`]  — подсказки по истории и закладкам;
//!   * [`proxylink`] и `xray` — встроенный прокси (ссылки vless/vmess/trojan/ss, ядро xray-rs).
//! Связь с Kotlin — через JNI в модуле `jni_api` (класс `dev.tytebyte.tytbrowser.core.NativeCore`).

pub mod blocker;
pub mod json;
pub mod omnibox;
pub mod proxylink;
pub mod reader;
pub mod suggest;
pub mod url_util;
#[cfg(feature = "xray")]
pub mod xray;

mod jni_api;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
