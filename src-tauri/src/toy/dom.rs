// ai-generated
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Document {
        children: Vec<Node>,
    },
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Text(String),
}

const ALLOWED: &[&str] = &[
    "html", "head", "title", "body", "h1", "h2", "h3", "p", "a", "ul", "ol", "li", "div", "span",
    "img", "style",
];

const VOID: &[&str] = &["img"];

const DROP_WITH_CHILDREN: &[&str] = &["script", "noscript", "iframe", "object", "embed"];

pub fn parse_html(input: &str) -> Node {
    let mut p = Parser { s: input, i: 0 };
    p.skip_doctype();
    let children = p.parse_nodes(None);
    Node::Document { children }
}

pub fn document_title(root: &Node) -> Option<String> {
    fn walk(n: &Node) -> Option<String> {
        match n {
            Node::Element { tag, children, .. } if tag == "title" => {
                Some(text_content_of_children(children).trim().to_string())
            }
            Node::Document { children } | Node::Element { children, .. } => {
                children.iter().find_map(walk)
            }
            Node::Text(_) => None,
        }
    }
    walk(root).filter(|s| !s.is_empty())
}

pub fn collect_style_text(root: &Node) -> String {
    fn walk(n: &Node, out: &mut String) {
        match n {
            Node::Element { tag, children, .. } if tag == "style" => {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&text_content_of_children(children));
            }
            Node::Document { children } | Node::Element { children, .. } => {
                for c in children {
                    walk(c, out);
                }
            }
            Node::Text(_) => {}
        }
    }
    let mut out = String::new();
    walk(root, &mut out);
    out
}

fn text_content_of_children(children: &[Node]) -> String {
    let mut s = String::new();
    for c in children {
        match c {
            Node::Text(t) => s.push_str(t),
            Node::Element { children, .. } | Node::Document { children } => {
                s.push_str(&text_content_of_children(children));
            }
        }
    }
    s
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &'a str {
        &self.s[self.i..]
    }

    fn skip_doctype(&mut self) {
        self.skip_ws();
        let r = self.rest();
        if r.len() >= 9 && r[..9].eq_ignore_ascii_case("<!doctype") {
            if let Some(end) = r.find('>') {
                self.i += end + 1;
            }
        }
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.rest().chars().next() {
            if c.is_whitespace() {
                self.i += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn parse_nodes(&mut self, until: Option<&str>) -> Vec<Node> {
        let mut nodes = Vec::new();
        while !self.rest().is_empty() {
            if self.rest().starts_with("</") {
                if let Some(tag) = until {
                    if self.peek_close_tag().as_deref() == Some(tag) {
                        self.consume_close_tag();
                        break;
                    }
                }
                self.consume_close_tag();
                break;
            }
            if self.rest().starts_with("<!--") {
                self.skip_comment();
                continue;
            }
            if self.rest().starts_with('<') {
                nodes.extend(self.parse_element());
                continue;
            }
            if let Some(text) = self.parse_text() {
                nodes.push(text);
            }
        }
        nodes
    }

    fn peek_close_tag(&self) -> Option<String> {
        let r = self.rest();
        if !r.starts_with("</") {
            return None;
        }
        let after = &r[2..];
        let name_end = after
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
            .unwrap_or(after.len());
        if name_end == 0 {
            return None;
        }
        Some(after[..name_end].to_ascii_lowercase())
    }

    fn consume_close_tag(&mut self) {
        if let Some(end) = self.rest().find('>') {
            self.i += end + 1;
        } else {
            self.i = self.s.len();
        }
    }

    fn skip_comment(&mut self) {
        if let Some(end) = self.rest().find("-->") {
            self.i += end + 3;
        } else {
            self.i = self.s.len();
        }
    }

    fn parse_text(&mut self) -> Option<Node> {
        let r = self.rest();
        let end = r.find('<').unwrap_or(r.len());
        if end == 0 {
            return None;
        }
        let raw = &r[..end];
        self.i += end;
        let decoded = decode_entities(raw);
        if decoded.is_empty() {
            None
        } else {
            Some(Node::Text(decoded))
        }
    }

    fn parse_element(&mut self) -> Vec<Node> {
        debug_assert!(self.rest().starts_with('<'));
        self.i += 1;
        if self.rest().starts_with('/') {
            self.consume_close_tag_after_slash();
            return Vec::new();
        }
        let tag = self.parse_tag_name();
        if tag.is_empty() {
            if let Some(end) = self.rest().find('>') {
                self.i += end + 1;
            }
            return Vec::new();
        }
        let attrs = self.parse_attrs();
        let self_closing = self.consume_open_end();
        if DROP_WITH_CHILDREN.contains(&tag.as_str()) {
            if !self_closing && !VOID.contains(&tag.as_str()) {
                let _ = self.parse_nodes(Some(tag.as_str()));
            }
            return Vec::new();
        }
        let children = if self_closing || VOID.contains(&tag.as_str()) {
            Vec::new()
        } else {
            self.parse_nodes(Some(tag.as_str()))
        };
        if ALLOWED.contains(&tag.as_str()) {
            vec![Node::Element {
                tag,
                attrs,
                children,
            }]
        } else {
            children
        }
    }

    fn consume_close_tag_after_slash(&mut self) {
        if let Some(end) = self.rest().find('>') {
            self.i += end + 1;
        } else {
            self.i = self.s.len();
        }
    }

    fn parse_tag_name(&mut self) -> String {
        let r = self.rest();
        let end = r
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
            .unwrap_or(r.len());
        let name = r[..end].to_ascii_lowercase();
        self.i += end;
        name
    }

    fn parse_attrs(&mut self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        loop {
            self.skip_ws();
            let r = self.rest();
            if r.is_empty() || r.starts_with('>') || r.starts_with("/>") {
                break;
            }
            let name_end = r
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                .unwrap_or(r.len());
            if name_end == 0 {
                self.i += r.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                continue;
            }
            let name = r[..name_end].to_ascii_lowercase();
            self.i += name_end;
            self.skip_ws();
            let value = if self.rest().starts_with('=') {
                self.i += 1;
                self.skip_ws();
                self.parse_attr_value()
            } else {
                String::new()
            };
            attrs.push((name, value));
        }
        attrs
    }

    fn parse_attr_value(&mut self) -> String {
        let r = self.rest();
        if r.starts_with('"') || r.starts_with('\'') {
            let quote = r.as_bytes()[0] as char;
            self.i += 1;
            let inner = self.rest();
            if let Some(end) = inner.find(quote) {
                let v = decode_entities(&inner[..end]);
                self.i += end + 1;
                return v;
            }
            self.i = self.s.len();
            return decode_entities(inner);
        }
        let end = r
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(r.len());
        let v = decode_entities(&r[..end]);
        self.i += end;
        v
    }

    fn consume_open_end(&mut self) -> bool {
        self.skip_ws();
        if self.rest().starts_with("/>") {
            self.i += 2;
            true
        } else if self.rest().starts_with('>') {
            self.i += 1;
            false
        } else {
            false
        }
    }
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el<'a>(root: &'a Node, tag: &str) -> &'a Node {
        fn walk<'a>(n: &'a Node, tag: &str) -> Option<&'a Node> {
            match n {
                Node::Element { tag: t, .. } if t == tag => Some(n),
                Node::Document { children } | Node::Element { children, .. } => {
                    children.iter().find_map(|c| walk(c, tag))
                }
                Node::Text(_) => None,
            }
        }
        walk(root, tag).unwrap_or_else(|| panic!("missing <{tag}>"))
    }

    fn text_of(node: &Node) -> String {
        match node {
            Node::Text(s) => s.clone(),
            Node::Document { children } | Node::Element { children, .. } => {
                children.iter().map(text_of).collect()
            }
        }
    }

    #[test]
    fn doctype_is_ignored_and_title_is_read() {
        let tree = parse_html(
            r#"<!DOCTYPE html>
            <html><head><title>Hello</title></head>
            <body><h1>Hi</h1></body></html>"#,
        );
        assert_eq!(document_title(&tree).as_deref(), Some("Hello"));
        assert_eq!(text_of(el(&tree, "h1")).trim(), "Hi");
    }

    #[test]
    fn unknown_tags_are_skipped_children_kept() {
        let tree = parse_html("<body><section><p>kept</p></section></body>");
        match el(&tree, "body") {
            Node::Element { children, .. } => {
                assert!(children
                    .iter()
                    .all(|c| !matches!(c, Node::Element { tag, .. } if tag == "section")));
                assert!(children
                    .iter()
                    .any(|c| matches!(c, Node::Element { tag, .. } if tag == "p")));
            }
            _ => panic!("body"),
        }
        assert_eq!(text_of(el(&tree, "p")).trim(), "kept");
    }

    #[test]
    fn collects_style_text_and_anchor_attrs() {
        let tree = parse_html(
            r#"<head><style>h1 { color: #1f3d34; }</style></head>
               <body><p class="lead">x</p><a href="https://example.com/">n</a></body>"#,
        );
        assert!(collect_style_text(&tree).contains("color: #1f3d34"));
        match el(&tree, "a") {
            Node::Element { attrs, .. } => {
                assert_eq!(
                    attrs
                        .iter()
                        .find(|(k, _)| k == "href")
                        .map(|(_, v)| v.as_str()),
                    Some("https://example.com/")
                );
            }
            _ => panic!("a"),
        }
        match el(&tree, "p") {
            Node::Element { attrs, .. } => {
                assert_eq!(
                    attrs
                        .iter()
                        .find(|(k, _)| k == "class")
                        .map(|(_, v)| v.as_str()),
                    Some("lead")
                );
            }
            _ => panic!("p"),
        }
    }

    #[test]
    fn script_is_dropped_entirely() {
        let tree = parse_html("<body><script>alert(1)</script><p>ok</p></body>");
        let body_text = text_of(el(&tree, "body"));
        assert!(!body_text.contains("alert"));
        assert_eq!(text_of(el(&tree, "p")).trim(), "ok");
    }
}
// ai-generated end
