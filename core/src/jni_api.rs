//! JNI-мост. Имена функций должны совпадать с `external fun` в
//! `dev.tytebyte.tytbrowser.core.NativeCore` (все методы — `@JvmStatic`).
//!
//! Ни одна паника не должна пересекать границу JNI, поэтому каждый вызов обёрнут в `catch_unwind`.

use crate::blocker::{self, Category};
use crate::{omnibox, reader, suggest};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring};
use jni::JNIEnv;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn guard<T>(default: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(default)
}

fn get(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(String::from).unwrap_or_default()
}

fn ret(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s)
        .map(|j| j.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_version<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jstring {
    ret(&mut env, crate::VERSION)
}

#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_resolveInput<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    input: JString<'l>,
    engine: JString<'l>,
) -> jstring {
    let input = get(&mut env, &input);
    let engine = get(&mut env, &engine);
    let out = guard(String::from("{\"kind\":\"internal\",\"url\":\"tyt://start\",\"query\":\"\"}"), || {
        omnibox::resolve(&input, &engine).to_json()
    });
    ret(&mut env, &out)
}

#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_cleanUrl<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    url: JString<'l>,
) -> jstring {
    let url = get(&mut env, &url);
    let out = guard(url.clone(), || omnibox::clean_url(&url));
    ret(&mut env, &out)
}

/// Включает/выключает категории блокировки; при первом вызове грузит встроенные списки.
/// Возвращает число загруженных правил.
#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_blockerConfigure<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
    ads: jboolean,
    trackers: jboolean,
) -> jint {
    guard(0, || {
        let lock = blocker::global();
        let mut b = lock.write().unwrap_or_else(|e| e.into_inner());
        b.ads_on = ads != 0;
        b.trackers_on = trackers != 0;
        b.rule_count() as jint
    })
}

/// Подгружает дополнительные правила. `category`: 0 — реклама, 1 — трекеры.
#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_blockerAddRules<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    category: jint,
    text: JString<'l>,
) -> jint {
    let text = get(&mut env, &text);
    guard(0, || {
        let lock = blocker::global();
        let mut b = lock.write().unwrap_or_else(|e| e.into_inner());
        let cat = if category == 1 { Category::Trackers } else { Category::Ads };
        b.add_rules(cat, &text) as jint
    })
}

/// Вердикт для подресурса: 0 — пропустить, 1 — реклама, 2 — трекер.
#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_blockerCheck<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    url: JString<'l>,
    page_url: JString<'l>,
) -> jint {
    let url = get(&mut env, &url);
    let page = get(&mut env, &page_url);
    guard(0, || {
        let lock = blocker::global();
        let b = lock.read().unwrap_or_else(|e| e.into_inner());
        b.check(&url, &page) as jint
    })
}

#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_extractReadable<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    html: JString<'l>,
    base_url: JString<'l>,
) -> jstring {
    let html = get(&mut env, &html);
    let base = get(&mut env, &base_url);
    let out = guard(String::from("{\"ok\":false}"), || reader::extract(&html, &base));
    ret(&mut env, &out)
}

#[no_mangle]
pub extern "system" fn Java_dev_tytebyte_tytbrowser_core_NativeCore_suggest<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    query: JString<'l>,
    entries: JString<'l>,
    limit: jint,
    now: jni::sys::jlong,
) -> jstring {
    let query = get(&mut env, &query);
    let entries = get(&mut env, &entries);
    let out = guard(String::from("[]"), || {
        suggest::suggest(&query, &entries, limit.max(0) as usize, now)
    });
    ret(&mut env, &out)
}
