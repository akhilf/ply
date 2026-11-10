// ai-generated
use crate::toy::dom::Node;
use crate::toy::style::{compute_style, ComputedStyle, Display, Stylesheet};

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutBox {
    pub tag: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub style: ComputedStyle,
    pub text: Option<String>,
    pub href: Option<String>,
    pub src: Option<String>,
    pub children: Vec<LayoutBox>,
}

pub fn layout(root: &Node, viewport_width: f32, sheets: &[Stylesheet]) -> LayoutBox {
    let mut cursor_y = 0.0;
    let children = layout_flow(root, 0.0, &mut cursor_y, viewport_width, sheets);
    LayoutBox {
        tag: "#root".into(),
        x: 0.0,
        y: 0.0,
        width: viewport_width,
        height: cursor_y,
        style: compute_style(root, sheets),
        text: None,
        href: None,
        src: None,
        children,
    }
}

pub fn line_height(font_size: f32) -> f32 {
    font_size * 1.25
}

pub fn glyph_width(font_size: f32) -> f32 {
    font_size
}

fn layout_flow(
    node: &Node,
    content_x: f32,
    cursor_y: &mut f32,
    content_width: f32,
    sheets: &[Stylesheet],
) -> Vec<LayoutBox> {
    match node {
        Node::Document { children } => {
            let mut out = Vec::new();
            for child in children {
                out.extend(layout_flow(
                    child,
                    content_x,
                    cursor_y,
                    content_width,
                    sheets,
                ));
            }
            out
        }
        Node::Element { tag, .. }
            if matches!(tag.as_str(), "head" | "style" | "title" | "script") =>
        {
            Vec::new()
        }
        Node::Element { .. } => {
            let style = compute_style(node, sheets);
            match style.display {
                Display::None => Vec::new(),
                Display::Block => vec![layout_block(
                    node,
                    content_x,
                    cursor_y,
                    content_width,
                    sheets,
                    style,
                )],
                Display::Inline => layout_inline_as_blockish(
                    node,
                    content_x,
                    cursor_y,
                    content_width,
                    sheets,
                    style,
                ),
            }
        }
        Node::Text(text) => {
            if text.trim().is_empty() {
                Vec::new()
            } else {
                line_boxes(
                    text,
                    content_x,
                    cursor_y,
                    content_width,
                    &compute_style(&Node::Document { children: vec![] }, sheets),
                    None,
                    None,
                )
            }
        }
    }
}

fn layout_block(
    node: &Node,
    parent_x: f32,
    cursor_y: &mut f32,
    parent_width: f32,
    sheets: &[Stylesheet],
    style: ComputedStyle,
) -> LayoutBox {
    let (tag, attrs, kids) = match node {
        Node::Element {
            tag,
            attrs,
            children,
        } => (tag.clone(), attrs, children),
        _ => ("#block".into(), &Vec::new(), &Vec::new()),
    };
    let x = parent_x + style.margin.left;
    let width = style
        .width
        .unwrap_or((parent_width - style.margin.left - style.margin.right).max(0.0));
    let y = *cursor_y + style.margin.top;
    let inner_x = x + style.padding.left;
    let inner_w = (width - style.padding.left - style.padding.right).max(0.0);
    let mut inner_y = y + style.padding.top;
    let children = layout_mixed(kids, attrs, inner_x, &mut inner_y, inner_w, sheets, &style);
    let content_h = (inner_y - (y + style.padding.top)).max(0.0);
    let height = style.padding.top + content_h + style.padding.bottom;
    *cursor_y = y + height + style.margin.bottom;
    LayoutBox {
        tag,
        x,
        y,
        width,
        height,
        style,
        text: None,
        href: attr(attrs, "href"),
        src: attr(attrs, "src"),
        children,
    }
}

fn layout_mixed(
    kids: &[Node],
    _parent_attrs: &[(String, String)],
    inner_x: f32,
    inner_y: &mut f32,
    inner_w: f32,
    sheets: &[Stylesheet],
    parent_style: &ComputedStyle,
) -> Vec<LayoutBox> {
    let mut out = Vec::new();
    let mut inline_run: Vec<&Node> = Vec::new();
    for child in kids {
        if is_block_child(child, sheets) {
            flush_inlines(
                &mut inline_run,
                &mut out,
                inner_x,
                inner_y,
                inner_w,
                sheets,
                parent_style,
            );
            out.extend(layout_flow(child, inner_x, inner_y, inner_w, sheets));
        } else if is_skippable(child) {
            continue;
        } else {
            inline_run.push(child);
        }
    }
    flush_inlines(
        &mut inline_run,
        &mut out,
        inner_x,
        inner_y,
        inner_w,
        sheets,
        parent_style,
    );
    out
}

fn is_skippable(node: &Node) -> bool {
    match node {
        Node::Text(s) => s.trim().is_empty(),
        Node::Element { tag, .. } => matches!(tag.as_str(), "head" | "style" | "title" | "script"),
        Node::Document { .. } => false,
    }
}

fn is_block_child(node: &Node, sheets: &[Stylesheet]) -> bool {
    match node {
        Node::Element { .. } => compute_style(node, sheets).display == Display::Block,
        _ => false,
    }
}

fn flush_inlines(
    run: &mut Vec<&Node>,
    out: &mut Vec<LayoutBox>,
    x: f32,
    y: &mut f32,
    width: f32,
    sheets: &[Stylesheet],
    parent_style: &ComputedStyle,
) {
    if run.is_empty() {
        return;
    }
    for node in run.drain(..) {
        match node {
            Node::Text(text) => out.extend(line_boxes(text, x, y, width, parent_style, None, None)),
            Node::Element {
                tag,
                attrs,
                children,
            } => {
                let style = compute_style(node, sheets);
                if tag == "img" {
                    let h = style.font_size.max(16.0);
                    let w = style.width.unwrap_or(h);
                    out.push(LayoutBox {
                        tag: tag.clone(),
                        x,
                        y: *y,
                        width: w,
                        height: h,
                        style,
                        text: None,
                        href: None,
                        src: attr(attrs, "src"),
                        children: Vec::new(),
                    });
                    *y += h;
                    continue;
                }
                let text = text_of(children);
                let href = attr(attrs, "href");
                out.extend(line_boxes(&text, x, y, width, &style, href, None));
            }
            Node::Document { .. } => {}
        }
    }
}

fn layout_inline_as_blockish(
    node: &Node,
    content_x: f32,
    cursor_y: &mut f32,
    content_width: f32,
    _sheets: &[Stylesheet],
    style: ComputedStyle,
) -> Vec<LayoutBox> {
    match node {
        Node::Element {
            tag,
            attrs,
            children,
        } if tag == "img" => {
            let h = style.font_size.max(16.0);
            let w = style.width.unwrap_or(h);
            let box_ = LayoutBox {
                tag: tag.clone(),
                x: content_x,
                y: *cursor_y,
                width: w,
                height: h,
                style,
                text: None,
                href: None,
                src: attr(attrs, "src"),
                children: Vec::new(),
            };
            *cursor_y += h;
            vec![box_]
        }
        Node::Element {
            tag: _,
            attrs,
            children,
        } => {
            let text = text_of(children);
            line_boxes(
                &text,
                content_x,
                cursor_y,
                content_width,
                &style,
                attr(attrs, "href"),
                attr(attrs, "src"),
            )
        }
        _ => Vec::new(),
    }
}

fn line_boxes(
    text: &str,
    x: f32,
    y: &mut f32,
    max_width: f32,
    style: &ComputedStyle,
    href: Option<String>,
    src: Option<String>,
) -> Vec<LayoutBox> {
    let collapsed = collapse_ws(text);
    if collapsed.is_empty() {
        return Vec::new();
    }
    let gw = glyph_width(style.font_size).max(1.0);
    let lh = line_height(style.font_size);
    let cols = (max_width / gw).floor().max(1.0) as usize;
    let lines = wrap_text(&collapsed, cols);
    let mut out = Vec::new();
    for line in lines {
        let w = (line.chars().count() as f32 * gw).min(max_width);
        out.push(LayoutBox {
            tag: "#text".into(),
            x,
            y: *y,
            width: w,
            height: lh,
            style: style.clone(),
            text: Some(line),
            href: href.clone(),
            src: src.clone(),
            children: Vec::new(),
        });
        *y += lh;
    }
    out
}

fn wrap_text(text: &str, cols: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split(' ') {
        if word.is_empty() {
            continue;
        }
        if current.is_empty() {
            push_chunks(&mut lines, &mut current, word, cols);
            continue;
        }
        if current.len() + 1 + word.len() <= cols {
            current.push(' ');
            current.push_str(word);
        } else if word.len() > cols {
            lines.push(std::mem::take(&mut current));
            push_chunks(&mut lines, &mut current, word, cols);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn push_chunks(lines: &mut Vec<String>, current: &mut String, word: &str, cols: usize) {
    let chars: Vec<char> = word.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let room = cols.saturating_sub(current.len());
        if room == 0 {
            lines.push(std::mem::take(current));
            continue;
        }
        let take = room.min(chars.len() - i);
        current.extend(chars[i..i + take].iter().copied());
        i += take;
        if current.len() == cols && i < chars.len() {
            lines.push(std::mem::take(current));
        }
    }
}

fn collapse_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn text_of(children: &[Node]) -> String {
    let mut s = String::new();
    for c in children {
        match c {
            Node::Text(t) => s.push_str(t),
            Node::Element { children, .. } | Node::Document { children } => {
                s.push_str(&text_of(children));
            }
        }
    }
    s
}

fn attr(attrs: &[(String, String)], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy::dom::parse_html;

    fn find<'a>(root: &'a LayoutBox, tag: &str) -> &'a LayoutBox {
        fn walk<'a>(n: &'a LayoutBox, tag: &str) -> Option<&'a LayoutBox> {
            if n.tag == tag {
                return Some(n);
            }
            n.children.iter().find_map(|c| walk(c, tag))
        }
        walk(root, tag).unwrap_or_else(|| panic!("missing layout {tag}"))
    }

    #[test]
    fn body_is_inset_by_default_margin() {
        let tree = parse_html("<body><p>Hi</p></body>");
        let root = layout(&tree, 400.0, &[]);
        let body = find(&root, "body");
        assert_eq!(body.x, 8.0);
        assert_eq!(body.width, 384.0);
    }

    #[test]
    fn heading_then_paragraph_stack() {
        let tree = parse_html("<body><h1>A</h1><p>B</p></body>");
        let root = layout(&tree, 400.0, &[]);
        let h1 = find(&root, "h1");
        let p = find(&root, "p");
        assert!(p.y >= h1.y + h1.height - 0.01);
    }

    #[test]
    fn long_text_wraps_to_taller_box() {
        let tree = parse_html("<body><p>xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx</p></body>");
        let root = layout(&tree, 400.0, &[]);
        let p = find(&root, "p");
        assert!(p.height > line_height(16.0) + 1.0);
    }

    #[test]
    fn list_item_indents() {
        let tree = parse_html("<body><ul><li>one</li></ul></body>");
        let root = layout(&tree, 400.0, &[]);
        let li = find(&root, "li");
        assert_eq!(li.x, 32.0);
    }
}
// ai-generated end
