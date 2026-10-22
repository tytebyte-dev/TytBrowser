# Tyt Browser

Лёгкий браузер для Android (5.0+) на **Kotlin + Rust**: движок страниц — системный WebView,
«мозги» — нативное ядро на Rust. Без AndroidX и сторонних библиотек в приложении, поэтому APK маленький.
Дизайн и сценарии — по макету Aura Lite (см. `design/`).

## Что умеет

- вкладки (обычные и приватные), переключатель вкладок, восстановление вкладок после перезапуска;
- адресная строка сверху или снизу, поиск через DuckDuckGo / Google / Яндекс / Bing, подсказки по истории и закладкам;
- встроенная блокировка рекламы и трекеров (Rust), счётчик заблокированного, свои списки правил;
- режим чтения (Rust вытягивает текст статьи; темы «бумага / сепия / тёмная / чёрная», размер и тип шрифта);
- закладки и быстрый доступ, история с поиском, загрузки (системный DownloadManager);
- поиск по странице, версия для ПК, режим без картинок, принудительно тёмные страницы, масштаб текста;
- очистка ссылок от `utm_*`, `fbclid`, `gclid`… при копировании и «Поделиться»;
- темы: светлая, тёмная, AMOLED; открытие ссылок из других приложений, «Поделиться» в браузер;
- выгрузка фоновых вкладок из памяти.

## Устройство

```
core/   Rust-крейт tytcore (cdylib): omnibox, blocker, reader, suggest + JNI-мост
app/    Android-приложение (Kotlin, программный UI на платформенных виджетах)
design/ исходный React-макет (только референс)
```

| Что | Где |
|---|---|
| Разбор ввода: адрес / поиск / опасные схемы | `core/src/omnibox.rs` |
| Блокировщик (подмножество Adblock Plus + hosts) | `core/src/blocker.rs`, списки в `core/rules/` |
| Режим чтения (упрощённый Readability) | `core/src/reader.rs` |
| Подсказки адресной строки | `core/src/suggest.rs` |
| JNI | `core/src/jni_api.rs` ↔ `app/.../core/NativeCore.kt` |
| Экран браузера | `app/.../BrowserActivity.kt` |

Если `libtytcore.so` не загрузилась, приложение не падает: `Core` переключается на упрощённые
Kotlin-реализации (без блокировки и режима чтения).

## Сборка

Нужны: JDK 17, Android SDK 34, NDK 26, Rust (stable) с таргетами Android и `cargo-ndk`.

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk

cargo test --manifest-path core/Cargo.toml     # тесты ядра
gradle :app:assembleDebug                      # Rust соберётся автоматически (задача cargoNdk)
gradle :app:assembleDebug -PskipRust           # без пересборки Rust, если .so уже в app/src/main/jniLibs
```

Gradle wrapper в репозиторий не добавлен: либо системный `gradle` 8.9+, либо `gradle wrapper --gradle-version 8.9`.
GitHub Actions (`.github/workflows/build.yml`) прогоняет тесты Rust и собирает APK. Тяжёлые слои (cargo-ndk, Android SDK/NDK, Gradle, Rust-таргет) кэшируются: на каждом запуске слой сначала восстанавливается и проверяется, а если его нет или он повреждён, ставится заново и сохраняется. Сброс всех слоёв: поменять `CACHE_SALT` в workflow.
Для подписи релиза задайте `TYT_KEYSTORE`, `TYT_KEYSTORE_PASSWORD`, `TYT_KEY_ALIAS`, `TYT_KEY_PASSWORD`.

## Свои правила блокировки

Положите файлы `ads.txt` / `trackers.txt` во внутреннюю папку приложения `files/filters/`.
Поддерживается: `||example.com^`, `||example.com/path^`, `$third-party`, `@@||…` (исключения),
строки формата hosts и просто домены. Остальной синтаксис Adblock Plus пропускается.

## Ограничения

- Приватные вкладки: нет истории, кэша и DOM-хранилища, но WebView не умеет изолировать cookies —
  они общие с обычными вкладками (кнопка «Очистить данные» удаляет всё).
- Блокировка работает на уровне запросов WebView (домены и префиксы путей), косметических фильтров нет.
- Расширений и синхронизации нет.
