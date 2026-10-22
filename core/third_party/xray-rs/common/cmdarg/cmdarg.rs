// Module: common\cmdarg\cmdarg.rs
// 1:1 Rust implementation corresponding to Go common\cmdarg\cmdarg.go

#[derive(Debug, Clone, Default)]
pub struct ArgList(pub Vec<String>);

impl ArgList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, arg: impl Into<String>) {
        self.0.push(arg.into());
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}
