// ai-generated
use crate::toy::layout::LayoutBox;
use crate::toy::style::Color;
use font8x8::{UnicodeFonts, BASIC_FONTS};
use tiny_skia::{Paint, Pixmap, Rect, Transform};

pub fn paint(root: &LayoutBox, width: u32, height: u32) -> Vec<u8> {
    let mut pixmap = Pixmap::new(width.max(1), height.max(1)).expect("pixmap");
    pixmap.fill(tiny_skia::Color::WHITE);
    paint_box(&mut pixmap, root);
    pixmap.encode_png().unwrap_or_default()
}

#[cfg(test)]
pub fn png_contains_rgb(png: &[u8], r: u8, g: u8, b: u8) -> bool {
    let decoder = png::Decoder::new(std::io::Cursor::new(png));
    let mut reader = decoder.read_info().expect("png info");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("png frame");
    let channels = info.color_type.samples();
    buf.chunks(channels)
        .any(|px| px[0] == r && px[1] == g && px[2] == b)
}

#[cfg(test)]
pub fn sample_color(png: &[u8], x: u32, y: u32) -> [u8; 4] {
    let decoder = png::Decoder::new(std::io::Cursor::new(png));
    let mut reader = decoder.read_info().expect("png info");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("png frame");
    let w = info.width;
    let channels = info.color_type.samples();
    let idx = ((y * w + x) as usize) * channels;
    match channels {
        4 => [buf[idx], buf[idx + 1], buf[idx + 2], buf[idx + 3]],
        3 => [buf[idx], buf[idx + 1], buf[idx + 2], 255],
        1 => [buf[idx], buf[idx], buf[idx], 255],
        _ => [0, 0, 0, 255],
    }
}

fn paint_box(pixmap: &mut Pixmap, node: &LayoutBox) {
    if let Some(bg) = node.style.background_color {
        fill_rect(pixmap, node.x, node.y, node.width, node.height, bg);
    }
    if let Some(text) = &node.text {
        draw_text(
            pixmap,
            text,
            node.x,
            node.y,
            node.style.font_size,
            node.style.color,
        );
    }
    for child in &node.children {
        paint_box(pixmap, child);
    }
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Color) {
    let w = w.max(0.0);
    let h = h.max(0.0);
    if w < 0.5 || h < 0.5 {
        return;
    }
    let Some(rect) = Rect::from_xywh(x, y, w, h) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    paint.anti_alias = false;
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

fn draw_text(pixmap: &mut Pixmap, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let scale = (font_size / 8.0).max(1.0);
    let mut cx = x;
    for ch in text.chars() {
        if let Some(glyph) = BASIC_FONTS.get(ch) {
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if bits & (1 << col) != 0 {
                        fill_rect(
                            pixmap,
                            cx + col as f32 * scale,
                            y + row as f32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
        cx += font_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy::dom::parse_html;
    use crate::toy::layout::layout;

    fn find<'a>(root: &'a LayoutBox, tag: &str) -> &'a LayoutBox {
        fn walk<'a>(n: &'a LayoutBox, tag: &str) -> Option<&'a LayoutBox> {
            if n.tag == tag {
                return Some(n);
            }
            n.children.iter().find_map(|c| walk(c, tag))
        }
        walk(root, tag).unwrap_or_else(|| panic!("missing {tag}"))
    }

    #[test]
    fn canvas_is_white() {
        let tree = parse_html("<body></body>");
        let boxes = layout(&tree, 100.0, &[]);
        let png = paint(&boxes, 100, 80);
        assert_eq!(sample_color(&png, 0, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn paints_block_background() {
        let tree = parse_html(
            r#"<body><div style="background-color: #1f3d34; width: 120px; font-size: 40px">X</div></body>"#,
        );
        let boxes = layout(&tree, 400.0, &[]);
        let png = paint(&boxes, 400, 200);
        let div = find(&boxes, "div");
        let x = (div.x + div.width / 2.0) as u32;
        let y = (div.y + div.height / 2.0) as u32;
        assert_eq!(sample_color(&png, x, y), [0x1f, 0x3d, 0x34, 255]);
    }

    #[test]
    fn class_background_paints() {
        let tree = parse_html(
            r#"<head><style>.lead { background-color: #b56a3a; font-size: 32px; }</style></head>
               <body><p class="lead">Hi</p></body>"#,
        );
        let sheets = crate::toy::style::collect_author_sheets(&tree);
        let boxes = layout(&tree, 400.0, &sheets);
        let png = paint(&boxes, 400, 200);
        let p = find(&boxes, "p");
        let x = (p.x + 8.0) as u32;
        let y = (p.y + p.height / 2.0) as u32;
        assert_eq!(sample_color(&png, x, y), [0xb5, 0x6a, 0x3a, 255]);
    }
}
// ai-generated end
