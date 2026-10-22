//! Режим чтения: выделяет основной текст статьи из HTML (упрощённый Readability).
//!
//! Вход — HTML страницы, выход — JSON:
//! `{"ok":true,"title":"…","byline":"…","site":"host","words":123,"blocks":[{"t":"h","l":2,"x":"…"},…]}`
//! Типы блоков: `h` (заголовок, уровень `l`), `p`, `li`, `q` (цитата), `pre`, `cap` (подпись).
//! Если текста слишком мало: `{"ok":false}`.

use crate::json;
use crate::url_util;

const MAX_HTML: usize = 4 * 1024 * 1024;
const MAX_DEPTH: usize = 400;
const MAX_BLOCKS: usize = 4000;
const MAX_CHARS: usize = 400_000;
const MIN_ARTICLE_CHARS: usize = 140;

// ───────────────────────────── DOM ─────────────────────────────

#[derive(Default, Clone)]
struct Node {
    tag: String, // "" — текстовый узел, "#root" — корень
    text: String,
    idc: String, // id + class в нижнем регистре
    hidden: bool,
    itemprop_author: bool,
    parent: usize,
    children: Vec<usize>,
    // агрегаты (считаются после разбора)
    tlen: usize,
    llen: usize,
    commas: usize,
}

#[derive(Default)]
struct Doc {
    nodes: Vec<Node>,
    title: String,
    og_title: String,
    meta_author: String,
    site_name: String,
}

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr", "keygen", "command",
];

const RAW_TEXT: &[&str] = &["script", "style", "textarea", "title", "noscript", "template"];

const SKIP: &[&str] = &[
    "script", "style", "noscript", "template", "nav", "aside", "footer", "form", "button",
    "select", "option", "iframe", "svg", "canvas", "menu", "dialog", "object", "embed", "audio",
    "video", "map", "label", "input", "textarea", "head",
];

const BLOCK_START_CLOSES_P: &[&str] = &[
    "address", "article", "aside", "blockquote", "div", "dl", "fieldset", "figure", "footer",
    "form", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "main", "nav", "ol", "p", "pre",
    "section", "table", "ul", "li",
];

const BLOCKISH: &[&str] = &[
    "address", "article", "aside", "blockquote", "dd", "details", "div", "dl", "dt", "fieldset",
    "figcaption", "figure", "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr",
    "li", "main", "nav", "ol", "p", "pre", "section", "table", "tbody", "thead", "tfoot", "tr",
    "td", "th", "ul", "body", "html", "summary", "center",
];

fn is_in(list: &[&str], s: &str) -> bool {
    list.contains(&s)
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        rest = &rest[pos..];
        // максимум 12 символов до ';'
        let semi = rest.char_indices().take(14).find(|(_, c)| *c == ';').map(|(i, _)| i);
        let decoded = semi.and_then(|end| {
            let name = &rest[1..end];
            let ch: Option<String> = if let Some(num) = name.strip_prefix('#') {
                let code = if let Some(hex) = num.strip_prefix(['x', 'X']) {
                    u32::from_str_radix(hex, 16).ok()
                } else {
                    num.parse::<u32>().ok()
                };
                code.and_then(char::from_u32).map(|c| c.to_string())
            } else {
                let s = match name {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    "nbsp" => "\u{a0}",
                    "mdash" => "—",
                    "ndash" => "–",
                    "hellip" => "…",
                    "laquo" => "«",
                    "raquo" => "»",
                    "ldquo" => "“",
                    "rdquo" => "”",
                    "lsquo" => "‘",
                    "rsquo" => "’",
                    "bdquo" => "„",
                    "copy" => "©",
                    "reg" => "®",
                    "trade" => "™",
                    "deg" => "°",
                    "middot" => "·",
                    "bull" => "•",
                    "euro" => "€",
                    "rarr" => "→",
                    "larr" => "←",
                    "times" => "×",
                    "shy" => "",
                    _ => return None,
                };
                Some(s.to_string())
            };
            ch.map(|c| (c, end))
        });
        match decoded {
            Some((c, end)) => {
                out.push_str(&c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

struct Attrs {
    id: String,
    class: String,
    property: String,
    name: String,
    content: String,
    itemprop: String,
    hidden: bool,
}

fn parse_attrs(s: &str) -> Attrs {
    let mut a = Attrs {
        id: String::new(),
        class: String::new(),
        property: String::new(),
        name: String::new(),
        content: String::new(),
        itemprop: String::new(),
        hidden: false,
    };
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b'/') {
            i += 1;
        }
        let ns = i;
        while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'=' && b[i] != b'/' {
            i += 1;
        }
        if ns == i {
            i += 1;
            continue;
        }
        let name = s[ns..i].to_ascii_lowercase();
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if i < b.len() && b[i] == b'=' {
            i += 1;
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < b.len() && (b[i] == b'"' || b[i] == b'\'') {
                let q = b[i];
                i += 1;
                let vs = i;
                while i < b.len() && b[i] != q {
                    i += 1;
                }
                value = s[vs..i].to_string();
                i += 1;
            } else {
                let vs = i;
                while i < b.len() && !b[i].is_ascii_whitespace() {
                    i += 1;
                }
                value = s[vs..i].to_string();
            }
        }
        match name.as_str() {
            "id" => a.id = value.to_lowercase(),
            "class" => a.class = value.to_lowercase(),
            "property" => a.property = value.to_lowercase(),
            "name" => a.name = value.to_lowercase(),
            "content" => a.content = decode_entities(&value),
            "itemprop" => a.itemprop = value.to_lowercase(),
            "hidden" => a.hidden = true,
            "aria-hidden" => {
                if value.eq_ignore_ascii_case("true") {
                    a.hidden = true
                }
            }
            "style" => {
                let v: String = value.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase();
                if v.contains("display:none") || v.contains("visibility:hidden") {
                    a.hidden = true;
                }
            }
            _ => {}
        }
    }
    a
}

fn parse(html: &str) -> Doc {
    let html = if html.len() > MAX_HTML {
        let mut end = MAX_HTML;
        while !html.is_char_boundary(end) {
            end -= 1;
        }
        &html[..end]
    } else {
        html
    };
    let lower = html.to_ascii_lowercase();
    let b = html.as_bytes();

    let mut doc = Doc::default();
    doc.nodes.push(Node { tag: "#root".into(), ..Default::default() });
    let mut stack: Vec<usize> = vec![0];

    let mut i = 0;
    let mut text_start = 0;

    macro_rules! flush_text {
        ($end:expr) => {{
            if $end > text_start {
                let t = decode_entities(&html[text_start..$end]);
                if !t.is_empty() {
                    let parent = *stack.last().unwrap();
                    let idx = doc.nodes.len();
                    doc.nodes.push(Node { text: t, parent, ..Default::default() });
                    doc.nodes[parent].children.push(idx);
                }
            }
        }};
    }

    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        let rest = &html[i..];
        if rest.starts_with("<!--") {
            flush_text!(i);
            i = match html[i + 4..].find("-->") {
                Some(p) => i + 4 + p + 3,
                None => b.len(),
            };
            text_start = i;
            continue;
        }
        if rest.starts_with("<![CDATA[") {
            flush_text!(i);
            i = match html[i..].find("]]>") {
                Some(p) => i + p + 3,
                None => b.len(),
            };
            text_start = i;
            continue;
        }
        let next = b.get(i + 1).copied().unwrap_or(0);
        if next == b'!' || next == b'?' {
            flush_text!(i);
            i = match html[i..].find('>') {
                Some(p) => i + p + 1,
                None => b.len(),
            };
            text_start = i;
            continue;
        }
        if next == b'/' {
            // закрывающий тег
            let ns = i + 2;
            let mut ne = ns;
            while ne < b.len() && (b[ne].is_ascii_alphanumeric() || b[ne] == b'-') {
                ne += 1;
            }
            if ne == ns {
                i += 1;
                continue;
            }
            flush_text!(i);
            let name = lower[ns..ne].to_string();
            i = match html[ne..].find('>') {
                Some(p) => ne + p + 1,
                None => b.len(),
            };
            text_start = i;
            if is_in(VOID, &name) {
                continue;
            }
            if let Some(pos) = stack.iter().rposition(|&n| doc.nodes[n].tag == name) {
                if pos > 0 {
                    stack.truncate(pos);
                }
            }
            continue;
        }
        if next.is_ascii_alphabetic() {
            // открывающий тег
            let ns = i + 1;
            let mut ne = ns;
            while ne < b.len() && (b[ne].is_ascii_alphanumeric() || b[ne] == b'-' || b[ne] == b':') {
                ne += 1;
            }
            // конец тега с учётом кавычек
            let mut j = ne;
            let mut quote = 0u8;
            while j < b.len() {
                let c = b[j];
                if quote != 0 {
                    if c == quote {
                        quote = 0;
                    }
                } else if c == b'"' || c == b'\'' {
                    quote = c;
                } else if c == b'>' {
                    break;
                }
                j += 1;
            }
            flush_text!(i);
            let name = lower[ns..ne].to_string();
            let attr_src = &html[ne..j.min(b.len())];
            let self_closing = attr_src.trim_end().ends_with('/');
            let attrs = parse_attrs(attr_src);
            i = (j + 1).min(b.len());
            text_start = i;

            if name == "meta" {
                if (attrs.property == "og:title") && doc.og_title.is_empty() {
                    doc.og_title = attrs.content.clone();
                } else if (attrs.name == "author" || attrs.property == "article:author")
                    && doc.meta_author.is_empty()
                    && !attrs.content.starts_with("http")
                {
                    doc.meta_author = attrs.content.clone();
                } else if attrs.property == "og:site_name" && doc.site_name.is_empty() {
                    doc.site_name = attrs.content.clone();
                }
                continue;
            }

            if is_in(RAW_TEXT, &name) && !self_closing {
                let close = format!("</{name}");
                let end = lower[i..].find(&close).map(|p| i + p).unwrap_or(b.len());
                if name == "title" && doc.title.is_empty() {
                    doc.title = collapse(&decode_entities(&html[i..end]));
                }
                i = match html[end..].find('>') {
                    Some(p) => end + p + 1,
                    None => b.len(),
                };
                text_start = i;
                continue;
            }

            // неявные закрытия
            if is_in(BLOCK_START_CLOSES_P, &name) {
                if let Some(&top) = stack.last() {
                    if doc.nodes[top].tag == "p" {
                        stack.pop();
                    }
                }
            }
            match name.as_str() {
                "li" => {
                    for k in (1..stack.len()).rev() {
                        let t = doc.nodes[stack[k]].tag.as_str();
                        if t == "ul" || t == "ol" {
                            break;
                        }
                        if t == "li" {
                            stack.truncate(k);
                            break;
                        }
                    }
                }
                "td" | "th" => {
                    if let Some(&top) = stack.last() {
                        let t = doc.nodes[top].tag.as_str();
                        if t == "td" || t == "th" {
                            stack.pop();
                        }
                    }
                }
                "tr" => {
                    if let Some(pos) = stack.iter().rposition(|&n| doc.nodes[n].tag == "tr") {
                        let between_table = stack[pos..]
                            .iter()
                            .all(|&n| matches!(doc.nodes[n].tag.as_str(), "tr" | "td" | "th"));
                        if between_table && pos > 0 {
                            stack.truncate(pos);
                        }
                    }
                }
                _ => {}
            }

            if name == "img" || name == "hr" || name == "wbr" {
                continue;
            }
            let parent = *stack.last().unwrap();
            let idx = doc.nodes.len();
            doc.nodes.push(Node {
                tag: name.clone(),
                idc: format!("{} {}", attrs.id, attrs.class),
                hidden: attrs.hidden,
                itemprop_author: attrs.itemprop.contains("author"),
                parent,
                ..Default::default()
            });
            doc.nodes[parent].children.push(idx);
            if !is_in(VOID, &name) && !self_closing && stack.len() < MAX_DEPTH {
                stack.push(idx);
            }
            continue;
        }
        // одиночный '<' в тексте
        i += 1;
    }
    flush_text!(b.len());

    // агрегаты: дети всегда имеют больший индекс, чем родитель
    for idx in (1..doc.nodes.len()).rev() {
        let (tag_is_a, own_len, own_commas) = {
            let n = &doc.nodes[idx];
            if n.tag.is_empty() {
                let c = collapse(&n.text);
                (false, c.chars().count(), c.matches([',', '，']).count())
            } else {
                (n.tag == "a", 0, 0)
            }
        };
        if doc.nodes[idx].tag.is_empty() {
            doc.nodes[idx].tlen = own_len;
            doc.nodes[idx].commas = own_commas;
        }
        if tag_is_a {
            doc.nodes[idx].llen = doc.nodes[idx].tlen;
        }
        let (t, l, c, p) = {
            let n = &doc.nodes[idx];
            (n.tlen, n.llen, n.commas, n.parent)
        };
        let parent = &mut doc.nodes[p];
        parent.tlen += t;
        parent.llen += l;
        parent.commas += c;
    }
    doc
}

// ───────────────────────── Текст ─────────────────────────

fn collapse(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for c in s.chars() {
        if c.is_whitespace() || c == '\u{a0}' || c == '\u{200b}' || c == '\u{feff}' {
            space = true;
        } else {
            if space && !out.is_empty() {
                out.push(' ');
            }
            space = false;
            out.push(c);
        }
    }
    out
}

const BR_MARK: char = '\u{1}';

impl Doc {
    fn skip_node(&self, idx: usize, chosen: usize) -> bool {
        let n = &self.nodes[idx];
        if n.hidden || is_in(SKIP, &n.tag) {
            return true;
        }
        if idx != chosen && !n.tag.is_empty() && n.tag != "body" && n.tag != "html" {
            let w = class_weight(&n.idc);
            if w < 0 {
                return true;
            }
        }
        false
    }

    fn flatten(&self, idx: usize, chosen: usize, out: &mut String, depth: usize) {
        if depth > MAX_DEPTH + 4 {
            return;
        }
        let n = &self.nodes[idx];
        if n.tag.is_empty() {
            out.push_str(&n.text);
            return;
        }
        if n.tag == "br" {
            out.push(BR_MARK);
            return;
        }
        if idx != chosen && self.skip_node(idx, chosen) {
            return;
        }
        for &c in &n.children {
            self.flatten(c, chosen, out, depth + 1);
        }
        if is_in(BLOCKISH, &n.tag) {
            out.push(' ');
        }
    }

    fn flat_text(&self, idx: usize, chosen: usize) -> String {
        let mut raw = String::new();
        self.flatten(idx, chosen, &mut raw, 0);
        let parts: Vec<String> = raw.split(BR_MARK).map(collapse).filter(|s| !s.is_empty()).collect();
        parts.join("\n")
    }

    fn pre_text(&self, idx: usize, out: &mut String) {
        let n = &self.nodes[idx];
        if n.tag.is_empty() {
            out.push_str(&n.text);
        } else if n.tag == "br" {
            out.push('\n');
        } else {
            for &c in &n.children {
                self.pre_text(c, out);
            }
        }
    }

    fn has_block_child(&self, idx: usize) -> bool {
        self.nodes[idx].children.iter().any(|&c| {
            let t = self.nodes[c].tag.as_str();
            matches!(
                t,
                "p" | "ul" | "ol" | "div" | "blockquote" | "pre" | "table" | "h1" | "h2" | "h3" | "h4" | "h5"
                    | "h6" | "figure" | "section" | "article" | "dl"
            )
        })
    }
}

fn class_weight(idc: &str) -> i32 {
    const NEG: &[&str] = &[
        "comment", "combx", "contact", "footer", "footnote", "masthead", "outbrain", "promo",
        "related", "scroll", "shoutbox", "sidebar", "sponsor", "shopping", "tags", "tool",
        "widget", "nav", "menu", "advert", "banner", "popup", "share", "social", "cookie",
        "subscribe", "newsletter", "breadcrumb", "pagination", "paywall", "modal", "toolbar",
        "recommend", "hidden", "skip-link", "yandex_rtb", "adfox",
    ];
    const POS: &[&str] = &[
        "article", "body", "content", "entry", "main", "page", "post", "text", "blog", "story",
        "hentry", "markdown", "prose",
    ];
    let mut w = 0;
    if NEG.iter().any(|p| idc.contains(p)) {
        w -= 25;
    }
    if POS.iter().any(|p| idc.contains(p)) {
        w += 25;
    }
    w
}

// ───────────────────────── Выбор контейнера ─────────────────────────

fn pick_container(doc: &Doc) -> usize {
    let n = doc.nodes.len();
    let mut score = vec![0f32; n];
    let mut seen = vec![false; n];

    let init = |idx: usize, score: &mut Vec<f32>, seen: &mut Vec<bool>| {
        if seen[idx] {
            return;
        }
        seen[idx] = true;
        let node = &doc.nodes[idx];
        let tag_bonus = match node.tag.as_str() {
            "article" => 10.0,
            "main" => 8.0,
            "div" => 5.0,
            "pre" | "td" | "blockquote" | "section" => 3.0,
            "address" | "ol" | "ul" | "dl" | "dd" | "dt" | "li" | "form" => -3.0,
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "th" => -5.0,
            _ => 0.0,
        };
        score[idx] = tag_bonus + class_weight(&node.idc) as f32;
    };

    for idx in 1..n {
        let node = &doc.nodes[idx];
        if !matches!(node.tag.as_str(), "p" | "pre" | "td" | "blockquote") || node.hidden {
            continue;
        }
        if node.tlen < 25 {
            continue;
        }
        let parent = node.parent;
        if parent == 0 {
            continue;
        }
        let contrib = 1.0 + node.commas as f32 + (node.tlen as f32 / 100.0).min(3.0);
        init(parent, &mut score, &mut seen);
        score[parent] += contrib;
        let grand = doc.nodes[parent].parent;
        if grand != 0 {
            init(grand, &mut score, &mut seen);
            score[grand] += contrib / 2.0;
        }
    }

    let final_score = |idx: usize| -> f32 {
        let node = &doc.nodes[idx];
        let density = node.llen as f32 / (node.tlen.max(1)) as f32;
        score[idx] * (1.0 - density.min(1.0))
    };

    let mut best = 0usize;
    let mut best_score = f32::MIN;
    for idx in 1..n {
        if !seen[idx] || matches!(doc.nodes[idx].tag.as_str(), "body" | "html") {
            continue;
        }
        let s = final_score(idx);
        if s > best_score {
            best_score = s;
            best = idx;
        }
    }
    if best == 0 {
        // Фолбэк: <article>/<main> или body.
        for want in ["article", "main", "body"] {
            if let Some(i) = (1..n).find(|&i| doc.nodes[i].tag == want && doc.nodes[i].tlen > 0) {
                return i;
            }
        }
        return 0;
    }

    // Поднимаемся к родителю, если он набрал сопоставимо (абзацы разложены по соседним блокам).
    let mut cur = best;
    loop {
        let p = doc.nodes[cur].parent;
        if p == 0 || !seen[p] || matches!(doc.nodes[p].tag.as_str(), "body" | "html") {
            break;
        }
        if final_score(p) >= final_score(cur) * 0.5 && doc.nodes[p].llen * 2 < doc.nodes[p].tlen.max(1) {
            cur = p;
        } else {
            break;
        }
    }
    cur
}

// ───────────────────────── Сборка блоков ─────────────────────────

struct Block {
    kind: &'static str,
    level: u8,
    text: String,
}

struct Ctx<'a> {
    doc: &'a Doc,
    chosen: usize,
    blocks: Vec<Block>,
    chars: usize,
}

impl Ctx<'_> {
    fn full(&self) -> bool {
        self.blocks.len() >= MAX_BLOCKS || self.chars >= MAX_CHARS
    }

    fn push(&mut self, kind: &'static str, level: u8, text: String) {
        let text = text.trim().to_string();
        if text.chars().count() < 2 && kind != "li" {
            return;
        }
        if text.is_empty() {
            return;
        }
        if let Some(last) = self.blocks.last() {
            if last.text == text && last.kind == kind {
                return;
            }
        }
        self.chars += text.len();
        self.blocks.push(Block { kind, level, text });
    }

    fn walk(&mut self, idx: usize, depth: usize) {
        if self.full() || depth > MAX_DEPTH + 4 {
            return;
        }
        let doc = self.doc;
        let node = &doc.nodes[idx];
        if node.tag.is_empty() {
            return;
        }
        if idx != self.chosen && doc.skip_node(idx, self.chosen) {
            return;
        }
        match node.tag.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = node.tag.as_bytes()[1] - b'0';
                let t = doc.flat_text(idx, self.chosen);
                self.push("h", level, t.replace('\n', " "));
            }
            "p" | "dt" | "dd" | "summary" => {
                let t = doc.flat_text(idx, self.chosen);
                self.push("p", 0, t);
            }
            "figcaption" | "caption" => {
                let t = doc.flat_text(idx, self.chosen);
                self.push("cap", 0, t);
            }
            "pre" => {
                let mut s = String::new();
                doc.pre_text(idx, &mut s);
                let s = s.trim_matches('\n').to_string();
                if !s.trim().is_empty() {
                    self.chars += s.len();
                    self.blocks.push(Block { kind: "pre", level: 0, text: s });
                }
            }
            "blockquote" => {
                if doc.has_block_child(idx) {
                    let start = self.blocks.len();
                    self.walk_children(idx, depth);
                    for b in &mut self.blocks[start..] {
                        if b.kind == "p" {
                            b.kind = "q";
                        }
                    }
                } else {
                    let t = doc.flat_text(idx, self.chosen);
                    self.push("q", 0, t);
                }
            }
            "li" => {
                if doc.has_block_child(idx) {
                    self.walk_children(idx, depth);
                } else {
                    let t = doc.flat_text(idx, self.chosen);
                    self.push("li", 0, t);
                }
            }
            "tr" => {
                let cells: Vec<String> = node
                    .children
                    .iter()
                    .filter(|&&c| matches!(doc.nodes[c].tag.as_str(), "td" | "th"))
                    .map(|&c| doc.flat_text(c, self.chosen).replace('\n', " "))
                    .filter(|s| !s.is_empty())
                    .collect();
                if !cells.is_empty() {
                    self.push("p", 0, cells.join(" | "));
                }
            }
            _ => self.walk_children(idx, depth),
        }
    }

    /// Обход детей контейнера: подряд идущие строчные узлы склеиваются в абзац.
    fn walk_children(&mut self, idx: usize, depth: usize) {
        let doc = self.doc;
        let mut run = String::new();
        for &c in &doc.nodes[idx].children {
            if self.full() {
                return;
            }
            let child = &doc.nodes[c];
            let inline = child.tag.is_empty() || (!is_in(BLOCKISH, &child.tag) && child.tag != "br" || child.tag == "br");
            let inline = inline && !matches!(child.tag.as_str(), "figcaption" | "caption" | "dt" | "dd" | "summary");
            if inline {
                if child.tag.is_empty() {
                    run.push_str(&child.text);
                } else if child.tag == "br" {
                    run.push(BR_MARK);
                } else if !doc.skip_node(c, self.chosen) {
                    doc.flatten(c, self.chosen, &mut run, 0);
                }
            } else {
                self.flush_run(&mut run);
                self.walk(c, depth + 1);
            }
        }
        self.flush_run(&mut run);
    }

    fn flush_run(&mut self, run: &mut String) {
        if run.is_empty() {
            return;
        }
        let parts: Vec<String> = run.split(BR_MARK).map(collapse).filter(|s| !s.is_empty()).collect();
        let t = parts.join("\n");
        run.clear();
        if t.chars().count() >= 20 {
            self.push("p", 0, t);
        }
    }
}

/// Извлекает статью из HTML. `base_url` нужен только для имени сайта.
pub fn extract(html: &str, base_url: &str) -> String {
    let doc = parse(html);
    let chosen = pick_container(&doc);
    if chosen == 0 || doc.nodes[chosen].tlen < MIN_ARTICLE_CHARS {
        return "{\"ok\":false}".to_string();
    }

    let mut ctx = Ctx { doc: &doc, chosen, blocks: Vec::new(), chars: 0 };
    ctx.walk(chosen, 0);
    if ctx.blocks.is_empty() {
        return "{\"ok\":false}".to_string();
    }
    let total_chars: usize = ctx.blocks.iter().map(|b| b.text.chars().count()).sum();
    if total_chars < MIN_ARTICLE_CHARS {
        return "{\"ok\":false}".to_string();
    }
    let words: usize = ctx.blocks.iter().map(|b| b.text.split_whitespace().count()).sum();

    let title = if !doc.og_title.trim().is_empty() {
        collapse(&doc.og_title)
    } else if !doc.title.is_empty() {
        doc.title.clone()
    } else {
        ctx.blocks.iter().find(|b| b.kind == "h").map(|b| b.text.clone()).unwrap_or_default()
    };

    let mut byline = collapse(&doc.meta_author);
    if byline.is_empty() {
        for idx in 1..doc.nodes.len() {
            let n = &doc.nodes[idx];
            if (n.itemprop_author || n.idc.contains("byline") || n.idc.contains("author"))
                && (3..=80).contains(&n.tlen)
                && !n.hidden
            {
                let t = doc.flat_text(idx, usize::MAX).replace('\n', " ");
                if !t.is_empty() {
                    byline = t;
                    break;
                }
            }
        }
    }

    let site = if !doc.site_name.is_empty() {
        doc.site_name.clone()
    } else {
        url_util::host_of(base_url)
    };

    let mut out = String::with_capacity(total_chars + ctx.blocks.len() * 24 + 128);
    out.push_str("{\"ok\":true,\"title\":");
    json::escape_into(&mut out, &title);
    out.push_str(",\"byline\":");
    json::escape_into(&mut out, &byline);
    out.push_str(",\"site\":");
    json::escape_into(&mut out, &site);
    out.push_str(&format!(",\"words\":{words},\"blocks\":["));
    for (i, b) in ctx.blocks.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"t\":\"");
        out.push_str(b.kind);
        out.push('"');
        if b.kind == "h" {
            out.push_str(&format!(",\"l\":{}", b.level));
        }
        out.push_str(",\"x\":");
        json::escape_into(&mut out, &b.text);
        out.push('}');
    }
    out.push_str("]}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTICLE: &str = r##"<!DOCTYPE html>
<html lang="ru"><head>
<title>Оптимизация WebView &mdash; Хабр</title>
<meta property="og:title" content="Оптимизация WebView под Android 10">
<meta name="author" content="Иван Петров">
<meta property="og:site_name" content="Хабр">
<script>var x = "<p>не должно попасть</p>";</script>
<style>.a { color: red }</style>
</head><body>
<header class="site-header"><a href="/">Хабр</a><nav><a href="/1">Статьи</a><a href="/2">Хабы</a></nav></header>
<div class="layout">
  <aside class="sidebar"><h3>Популярное</h3><ul><li><a href="/x">Читайте ещё</a></li><li><a href="/y">Ещё одна ссылка</a></li></ul></aside>
  <article class="post-content">
    <h1>Оптимизация WebView под Android 10</h1>
    <p>Современные устройства на Android 10 Go Edition имеют ограниченный объём оперативной памяти, поэтому браузер должен экономно использовать ресурсы, не теряя при этом скорость.</p>
    <h2>Что делать с изображениями</h2>
    <p>Блокировка изображений &laquo;на лету&raquo; экономит до 80% трафика, а также позволяет открывать тяжёлые страницы &amp; сохранять заряд батареи. Это особенно важно, запятые тоже считаются.</p>
    <ul><li>Отключите автозагрузку картинок</li><li>Включите сжатие трафика</li></ul>
    <blockquote>Память — самый дорогой ресурс мобильного браузера на старом устройстве.</blockquote>
    <pre>val settings = webView.settings
settings.loadsImagesAutomatically = false</pre>
    <p>Первая строка<br>вторая строка абзаца, достаточно длинная, чтобы пройти все пороги отсева текста.</p>
    <div class="share-buttons"><a href="#">Поделиться</a><a href="#">Твитнуть</a></div>
    <div class="comments"><p>Комментарий пользователя номер один, который не должен попасть в режим чтения вообще.</p></div>
  </article>
</div>
<footer><p>© 2026 Все права защищены, контакты и политика конфиденциальности.</p></footer>
</body></html>"##;

    #[test]
    fn decodes_entities() {
        assert_eq!(decode_entities("a &amp; b &lt;c&gt; &#8212; &#x41; &unknown; &"), "a & b <c> — A &unknown; &");
        assert_eq!(decode_entities("&laquo;x&raquo;"), "«x»");
    }

    #[test]
    fn extracts_article() {
        let json = extract(ARTICLE, "https://habr.com/ru/articles/1");
        assert!(json.starts_with("{\"ok\":true"), "{json}");
        assert!(json.contains("\"title\":\"Оптимизация WebView под Android 10\""));
        assert!(json.contains("\"byline\":\"Иван Петров\""));
        assert!(json.contains("\"site\":\"Хабр\""));
        assert!(json.contains("Что делать с изображениями"));
        assert!(json.contains("«на лету»"));
        assert!(json.contains("\"t\":\"li\""));
        assert!(json.contains("\"t\":\"q\""));
        assert!(json.contains("\"t\":\"pre\""));
        assert!(json.contains("Первая строка\\nвторая строка"));
        // мусор не попал
        assert!(!json.contains("Популярное"));
        assert!(!json.contains("Поделиться"));
        assert!(!json.contains("Комментарий пользователя"));
        assert!(!json.contains("не должно попасть"));
        assert!(!json.contains("Все права защищены"));
    }

    #[test]
    fn rejects_short_pages() {
        assert_eq!(extract("<html><body><p>Привет</p></body></html>", "https://a.com"), "{\"ok\":false}");
        assert_eq!(extract("", "https://a.com"), "{\"ok\":false}");
        assert_eq!(extract("<<<>>><", "https://a.com"), "{\"ok\":false}");
    }

    #[test]
    fn handles_div_soup_without_semantic_tags() {
        let html = format!(
            "<html><body><div id='wrap'><div class='content'>{}</div><div class='sidebar'><a href='#'>x</a></div></div></body></html>",
            (1..=4)
                .map(|i| format!("<p>Абзац номер {i}: достаточно длинный текст для проверки, запятые, точки, слова.</p>"))
                .collect::<String>()
        );
        let json = extract(&html, "https://example.com/");
        assert!(json.starts_with("{\"ok\":true"), "{json}");
        assert!(json.contains("Абзац номер 3"));
        assert!(!json.contains("\"x\":\"x\""));
    }

    #[test]
    fn survives_garbage_and_unclosed_tags() {
        let html = "<div><p>текст без закрытия <b>жирный <i>курсив<p>второй абзац ".repeat(50);
        let _ = extract(&html, "https://example.com/");
        let deep = "<div>".repeat(5000) + "<p>очень глубоко вложенный абзац с достаточно длинным текстом внутри для теста</p>";
        let _ = extract(&deep, "https://example.com/");
        let _ = extract("<a href=\"unterminated", "https://example.com/");
        let _ = extract("<!-- unterminated", "https://example.com/");
        let _ = extract("<p>unicode 😀 текст &#128512; &#xZZ; &#99999999;</p>", "https://example.com/");
    }

    #[test]
    fn implicit_li_and_table_rows() {
        let html = format!(
            "<html><body><article><p>{}</p><ul><li>один<li>два<li>три</ul><table><tr><td>a<td>b<tr><td>c<td>d</table></article></body></html>",
            "Достаточно длинный вводный абзац, который проходит пороги, и содержит запятые. ".repeat(3)
        );
        let json = extract(&html, "https://example.com/");
        assert!(json.contains("{\"t\":\"li\",\"x\":\"два\"}"), "{json}");
        assert!(json.contains("a | b"), "{json}");
        assert!(json.contains("c | d"), "{json}");
    }
}
