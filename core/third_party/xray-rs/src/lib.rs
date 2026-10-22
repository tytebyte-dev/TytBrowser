#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]

#[path = "../common/mod.rs"]
pub mod common;

#[path = "../transport/mod.rs"]
pub mod transport;

#[path = "../proxy/mod.rs"]
pub mod proxy;

#[path = "../features/mod.rs"]
pub mod features;

#[path = "../app/mod.rs"]
pub mod app;

#[path = "../infra/mod.rs"]
pub mod infra;

#[path = "../core/mod.rs"]
pub mod core;

#[path = "../testing/mod.rs"]
pub mod testing;

#[path = "../main/mod.rs"]
pub mod main;
