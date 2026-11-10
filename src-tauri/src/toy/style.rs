// ai-generated
use crate::toy::dom::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 128,
        b: 0,
        a: 255,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: Color,
    pub background_color: Option<Color>,
    pub font_size: f32,
    pub margin: Edges,
    pub padding: Edges,
    pub width: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selector: Selector,
    pub decls: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    Tag(String),
    Class(String),
    TagClass { tag: String, class: String },
}

pub fn parse_stylesheet(css: &str) -> Stylesheet {
    let stripped = strip_comments(css);
    let mut rules = Vec::new();
    let mut rest = stripped.as_str();
    while let Some(brace) = rest.find('{') {
        let sel_raw = rest[..brace].trim();
        rest = &rest[brace + 1..];
        let end = rest.find('}').unwrap_or(rest.len());
        let body = &rest[..end];
        rest = if end < rest.len() {
            &rest[end + 1..]
        } else {
            ""
        };
        let decls = parse_decls(body);
        for sel in sel_raw.split(',') {
            if let Some(selector) = parse_selector(sel.trim()) {
                rules.push(Rule {
                    selector,
                    decls: decls.clone(),
                });
            }
        }
    }
    Stylesheet { rules }
}

pub fn default_sheet() -> Stylesheet {
    parse_stylesheet(
        r#"
        html, body, h1, h2, h3, p, ul, ol, li, div { display: block; }
        span, a, img { display: inline; }
        body { margin: 8px; background-color: white; color: black; font-size: 16px; }
        h1 { font-size: 32px; margin: 8px 0; }
        h2 { font-size: 24px; margin: 8px 0; }
        h3 { font-size: 18px; margin: 8px 0; }
        a { color: blue; }
        p { margin: 8px 0; }
        li { margin-left: 24px; }
        "#,
    )
}

pub fn collect_author_sheets(root: &Node) -> Vec<Stylesheet> {
    let text = crate::toy::dom::collect_style_text(root);
    if text.trim().is_empty() {
        Vec::new()
    } else {
        vec![parse_stylesheet(&text)]
    }
}

pub fn compute_style(node: &Node, sheets: &[Stylesheet]) -> ComputedStyle {
    let mut style = ComputedStyle {
        display: Display::Inline,
        color: Color::BLACK,
        background_color: None,
        font_size: 16.0,
        margin: Edges::default(),
        padding: Edges::default(),
        width: None,
    };
    let Node::Element { tag, attrs, .. } = node else {
        return style;
    };
    let classes = class_list(attrs);
    let defaults = default_sheet();
    for sheet in std::iter::once(&defaults).chain(sheets) {
        for rule in &sheet.rules {
            if selector_matches(&rule.selector, tag, &classes) {
                for (name, value) in &rule.decls {
                    apply_decl(&mut style, name, value);
                }
            }
        }
    }
    if let Some((_, inline)) = attrs.iter().find(|(k, _)| k == "style") {
        for (name, value) in parse_decls(inline) {
            apply_decl(&mut style, &name, &value);
        }
    }
    style
}

fn class_list(attrs: &[(String, String)]) -> Vec<String> {
    attrs
        .iter()
        .find(|(k, _)| k == "class")
        .map(|(_, v)| {
            v.split_whitespace()
                .map(|c| c.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

fn selector_matches(sel: &Selector, tag: &str, classes: &[String]) -> bool {
    match sel {
        Selector::Tag(t) => t == tag,
        Selector::Class(c) => classes.iter().any(|x| x == c),
        Selector::TagClass { tag: t, class } => t == tag && classes.iter().any(|x| x == class),
    }
}

fn strip_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("*/") {
            rest = &rest[end + 2..];
        } else {
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

fn parse_selector(raw: &str) -> Option<Selector> {
    if raw.is_empty() {
        return None;
    }
    if let Some(dot) = raw.find('.') {
        let tag = raw[..dot].trim().to_ascii_lowercase();
        let class = raw[dot + 1..].trim().to_ascii_lowercase();
        if class.is_empty() {
            return None;
        }
        if tag.is_empty() {
            Some(Selector::Class(class))
        } else {
            Some(Selector::TagClass { tag, class })
        }
    } else {
        Some(Selector::Tag(raw.to_ascii_lowercase()))
    }
}

fn parse_decls(body: &str) -> Vec<(String, String)> {
    body.split(';')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let (name, value) = part.split_once(':')?;
            Some((name.trim().to_ascii_lowercase(), value.trim().to_string()))
        })
        .collect()
}

fn apply_decl(style: &mut ComputedStyle, name: &str, value: &str) {
    match name {
        "color" => {
            if let Some(c) = parse_color(value) {
                style.color = c;
            }
        }
        "background-color" => {
            if let Some(c) = parse_color(value) {
                style.background_color = Some(c);
            }
        }
        "font-size" => {
            if let Some(px) = parse_px(value) {
                style.font_size = px;
            }
        }
        "width" => style.width = parse_px(value),
        "display" => match value.trim() {
            "block" => style.display = Display::Block,
            "inline" => style.display = Display::Inline,
            "none" => style.display = Display::None,
            _ => {}
        },
        "margin" => {
            if let Some(e) = parse_edges(value) {
                style.margin = e;
            }
        }
        "padding" => {
            if let Some(e) = parse_edges(value) {
                style.padding = e;
            }
        }
        "margin-top" => {
            if let Some(px) = parse_px(value) {
                style.margin.top = px;
            }
        }
        "margin-right" => {
            if let Some(px) = parse_px(value) {
                style.margin.right = px;
            }
        }
        "margin-bottom" => {
            if let Some(px) = parse_px(value) {
                style.margin.bottom = px;
            }
        }
        "margin-left" => {
            if let Some(px) = parse_px(value) {
                style.margin.left = px;
            }
        }
        _ => {}
    }
}

fn parse_px(value: &str) -> Option<f32> {
    let v = value.trim().trim_end_matches("px").trim();
    v.parse().ok()
}

fn parse_edges(value: &str) -> Option<Edges> {
    let parts: Vec<f32> = value.split_whitespace().filter_map(parse_px).collect();
    match parts.as_slice() {
        [a] => Some(Edges {
            top: *a,
            right: *a,
            bottom: *a,
            left: *a,
        }),
        [t, x] => Some(Edges {
            top: *t,
            right: *x,
            bottom: *t,
            left: *x,
        }),
        [t, r, b] => Some(Edges {
            top: *t,
            right: *r,
            bottom: *b,
            left: *r,
        }),
        [t, r, b, l] => Some(Edges {
            top: *t,
            right: *r,
            bottom: *b,
            left: *l,
        }),
        _ => None,
    }
}

fn parse_color(value: &str) -> Option<Color> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::RED),
        "blue" => Some(Color::BLUE),
        "green" => Some(Color::GREEN),
        _ if v.starts_with('#') => parse_hex_color(&v[1..]),
        _ => None,
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color { r, g, b, a: 255 })
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color { r, g, b, a: 255 })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy::dom::parse_html;

    fn find<'a>(root: &'a Node, tag: &str) -> &'a Node {
        fn walk<'a>(n: &'a Node, tag: &str) -> Option<&'a Node> {
            match n {
                Node::Element { tag: t, .. } if t == tag => Some(n),
                Node::Document { children } | Node::Element { children, .. } => {
                    children.iter().find_map(|c| walk(c, tag))
                }
                Node::Text(_) => None,
            }
        }
        walk(root, tag).expect(tag)
    }

    #[test]
    fn tag_rule_sets_heading_color() {
        let sheet = parse_stylesheet("h1 { color: #1f3d34; }");
        let tree = parse_html("<h1>Hi</h1>");
        let style = compute_style(find(&tree, "h1"), &[sheet]);
        assert_eq!(
            style.color,
            Color {
                r: 0x1f,
                g: 0x3d,
                b: 0x34,
                a: 255
            }
        );
        assert_eq!(style.display, Display::Block);
        assert_eq!(style.font_size, 32.0);
    }

    #[test]
    fn class_rule_applies_and_inline_wins() {
        let sheet = parse_stylesheet(".lead { color: red; font-size: 20px; }");
        let tree = parse_html(r#"<p class="lead" style="color: #b56a3a">x</p>"#);
        let style = compute_style(find(&tree, "p"), &[default_sheet(), sheet]);
        assert_eq!(
            style.color,
            Color {
                r: 0xb5,
                g: 0x6a,
                b: 0x3a,
                a: 255
            }
        );
        assert_eq!(style.font_size, 20.0);
        assert_eq!(style.display, Display::Block);
    }

    #[test]
    fn unknown_properties_are_ignored() {
        let sheet = parse_stylesheet("p { flex: 1; color: blue; }");
        let tree = parse_html("<p>x</p>");
        let style = compute_style(find(&tree, "p"), &[sheet]);
        assert_eq!(style.color, Color::BLUE);
    }
}
// ai-generated end
