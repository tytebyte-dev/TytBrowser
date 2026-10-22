pub struct Arg<'a> {
    raw: &'a [&'a str],
}

impl<'a> Arg<'a> {
    pub fn new(raw: &'a [&'a str]) -> Self {
        Self { raw }
    }

    pub fn get(&self, idx: usize) -> Option<&'a str> {
        self.raw.get(idx).copied()
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn find_flag(&self, name: &str) -> Option<&'a str> {
        for (i, &arg) in self.raw.iter().enumerate() {
            if arg == name {
                return self.raw.get(i + 1).copied();
            }
            if let Some(stripped) = arg.strip_prefix(name)
                && let Some(val) = stripped.strip_prefix('=')
            {
                return Some(val);
            }
        }
        None
    }
}
