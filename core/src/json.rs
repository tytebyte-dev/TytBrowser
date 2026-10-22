//! Крошечный JSON-писатель: ядру нужен только вывод, поэтому serde не тянем.

use std::fmt::Write;

/// Дописывает `s` в `out` как JSON-строку (с кавычками и экранированием).
pub fn escape_into(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    escape_into(&mut out, s);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_specials() {
        assert_eq!(string("a\"b\\c\n\t"), "\"a\\\"b\\\\c\\n\\t\"");
        assert_eq!(string("\u{1}"), "\"\\u0001\"");
        assert_eq!(string("привет"), "\"привет\"");
    }
}
