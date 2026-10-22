// Module: common\serial\string.rs
// 1:1 Rust implementation corresponding to Go common\serial\string.go

pub trait StringValue {
    fn to_str_value(&self) -> String;
}

impl StringValue for &str {
    fn to_str_value(&self) -> String {
        self.to_string()
    }
}

impl StringValue for String {
    fn to_str_value(&self) -> String {
        self.clone()
    }
}

impl StringValue for &String {
    fn to_str_value(&self) -> String {
        (*self).clone()
    }
}

impl StringValue for &[u8] {
    fn to_str_value(&self) -> String {
        let items: Vec<String> = self.iter().map(|b| b.to_string()).collect();
        format!("[{}]", items.join(" "))
    }
}

impl<const N: usize> StringValue for [u8; N] {
    fn to_str_value(&self) -> String {
        (&self[..]).to_str_value()
    }
}

impl StringValue for Vec<u8> {
    fn to_str_value(&self) -> String {
        self.as_slice().to_str_value()
    }
}

impl StringValue for i32 {
    fn to_str_value(&self) -> String {
        self.to_string()
    }
}

impl StringValue for u32 {
    fn to_str_value(&self) -> String {
        self.to_string()
    }
}

impl StringValue for i64 {
    fn to_str_value(&self) -> String {
        self.to_string()
    }
}

impl StringValue for u64 {
    fn to_str_value(&self) -> String {
        self.to_string()
    }
}

impl StringValue for bool {
    fn to_str_value(&self) -> String {
        self.to_string()
    }
}

impl StringValue for &dyn std::error::Error {
    fn to_str_value(&self) -> String {
        self.to_string()
    }
}

pub fn to_string<T: std::fmt::Display>(val: T) -> String {
    val.to_string()
}

pub fn to_string_val(val: &dyn StringValue) -> String {
    val.to_str_value()
}

pub fn concat_strings(slices: &[&str]) -> String {
    slices.join("")
}

pub fn concat(values: &[&dyn StringValue]) -> String {
    let mut out = String::new();
    for v in values {
        out.push_str(&v.to_str_value());
    }
    out
}
