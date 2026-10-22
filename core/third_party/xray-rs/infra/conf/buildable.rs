// Module: infra\conf\buildable.rs
// 1:1 Rust implementation corresponding to Go infra\conf\buildable.go

pub trait Buildable {
    type Output;
    fn build(self) -> crate::common::errors::Result<Self::Output>;
}
