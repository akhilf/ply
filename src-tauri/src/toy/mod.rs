// ai-generated
pub mod dom;
pub mod fetch;
pub mod layout;
pub mod paint;
pub mod style;

use crate::engine::{Engine, LoadError};
use crate::toy::dom::{document_title, parse_html};
use crate::toy::fetch::fetch_document;
use crate::toy::layout::layout;
use crate::toy::paint::paint;
use crate::toy::style::collect_author_sheets;
use serde::Serialize;
use tauri::{Emitter, Manager};

pub struct RenderedPage {
    pub title: String,
    pub png: Vec<u8>,
}

pub fn render_document(html: &str, viewport_width: f32, viewport_height: f32) -> RenderedPage {
    let tree = parse_html(html);
    let title = document_title(&tree).unwrap_or_default();
    let sheets = collect_author_sheets(&tree);
    let boxes = layout(&tree, viewport_width, &sheets);
    let png = paint(
        &boxes,
        viewport_width.max(1.0) as u32,
        viewport_height.max(1.0) as u32,
    );
    RenderedPage { title, png }
}

pub struct ToyEngine {
    app: Option<tauri::AppHandle>,
    webview_label: String,
    history: Vec<String>,
    index: Option<usize>,
    visible: bool,
    viewport: (f32, f32),
    last_title: String,
}

#[derive(Clone, Serialize)]
struct EngineTitlePayload {
    tab_id: String,
    title: String,
}

impl ToyEngine {
    pub fn new(app: tauri::AppHandle, label: String) -> Self {
        Self {
            app: Some(app),
            webview_label: label,
            history: Vec::new(),
            index: None,
            visible: true,
            viewport: (800.0, 600.0),
            last_title: String::new(),
        }
    }

    #[cfg(test)]
    pub fn headless() -> Self {
        Self {
            app: None,
            webview_label: "headless".into(),
            history: Vec::new(),
            index: None,
            visible: true,
            viewport: (400.0, 300.0),
            last_title: String::new(),
        }
    }

    fn push_history(&mut self, url: &str) {
        if self.current_url().as_deref() == Some(url) {
            return;
        }
        if let Some(i) = self.index {
            self.history.truncate(i + 1);
        }
        self.history.push(url.to_string());
        self.index = Some(self.history.len() - 1);
    }

    fn present(&mut self, url: &str) -> Result<(), LoadError> {
        self.refresh_viewport();
        let html = fetch_document(url)?;
        let page = render_document(&html, self.viewport.0, self.viewport.1);
        self.last_title = page.title.clone();
        self.show_png(&page)?;
        if let Some(app) = &self.app {
            let _ = app.emit(
                "engine_title",
                EngineTitlePayload {
                    tab_id: self.webview_label.clone(),
                    title: if page.title.is_empty() {
                        url.to_string()
                    } else {
                        page.title
                    },
                },
            );
        }
        Ok(())
    }

    fn refresh_viewport(&mut self) {
        let Some(app) = &self.app else {
            return;
        };
        let Some(window) = app.get_window("main") else {
            return;
        };
        let Ok(scale) = window.scale_factor() else {
            return;
        };
        let Ok(size) = window.inner_size() else {
            return;
        };
        let logical = size.to_logical::<f64>(scale);
        self.viewport = (
            logical.width as f32,
            (logical.height - crate::shell::CHROME_HEIGHT).max(1.0) as f32,
        );
    }

    fn show_png(&self, page: &RenderedPage) -> Result<(), LoadError> {
        let Some(app) = &self.app else {
            return Ok(());
        };
        crate::shell::ensure_content_webview(app, &self.webview_label)?;
        let webview = app
            .get_webview(&self.webview_label)
            .ok_or(LoadError::LoadFailed)?;
        let frame = painted_frame_url(page)?;
        if webview.navigate(frame.clone()).is_err() {
            let href = serde_json::to_string(frame.as_str()).map_err(|_| LoadError::LoadFailed)?;
            webview
                .eval(&format!("window.location.replace({href})"))
                .map_err(|_| LoadError::LoadFailed)?;
        }
        if self.visible {
            let _ = webview.show();
        } else {
            let _ = webview.hide();
        }
        Ok(())
    }
}

fn painted_frame_url(page: &RenderedPage) -> Result<url::Url, LoadError> {
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &page.png);
    url::Url::parse(&format!("data:image/png;base64,{b64}")).map_err(|_| LoadError::LoadFailed)
}

impl Drop for ToyEngine {
    fn drop(&mut self) {
        if let Some(app) = &self.app {
            if let Some(webview) = app.get_webview(&self.webview_label) {
                let _ = webview.close();
            }
        }
    }
}

impl Engine for ToyEngine {
    fn navigate(&mut self, url: &str) -> Result<(), LoadError> {
        self.present(url)?;
        self.push_history(url);
        Ok(())
    }

    fn go_back(&mut self) -> Result<(), LoadError> {
        match self.index {
            Some(i) if i > 0 => {
                self.index = Some(i - 1);
                let url = self.history[i - 1].clone();
                self.present(&url)
            }
            _ => Err(LoadError::LoadFailed),
        }
    }

    fn go_forward(&mut self) -> Result<(), LoadError> {
        match self.index {
            Some(i) if i + 1 < self.history.len() => {
                self.index = Some(i + 1);
                let url = self.history[i + 1].clone();
                self.present(&url)
            }
            _ => Err(LoadError::LoadFailed),
        }
    }

    fn reload(&mut self) -> Result<(), LoadError> {
        let url = self.current_url().ok_or(LoadError::LoadFailed)?;
        self.present(&url)
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if let Some(app) = &self.app {
            if let Some(webview) = app.get_webview(&self.webview_label) {
                if visible {
                    let _ = webview.show();
                } else {
                    let _ = webview.hide();
                }
            }
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn observe_url(&mut self, url: &str) {
        self.push_history(url);
    }

    fn can_go_back(&self) -> bool {
        self.index.map(|i| i > 0).unwrap_or(false)
    }

    fn can_go_forward(&self) -> bool {
        self.index
            .map(|i| i + 1 < self.history.len())
            .unwrap_or(false)
    }

    fn current_url(&self) -> Option<String> {
        self.index.map(|i| self.history[i].clone())
    }

    fn last_title(&self) -> Option<String> {
        if self.last_title.is_empty() {
            None
        } else {
            Some(self.last_title.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_start_fixture_has_title_and_png() {
        let html = fetch_document("ply://start").unwrap();
        let page = render_document(&html, 400.0, 300.0);
        assert_eq!(page.title, "Ply toy");
        assert!(page.png.len() > 100);
    }

    #[test]
    fn start_fixture_png_contains_heading_ink() {
        let html = fetch_document("ply://start").unwrap();
        let page = render_document(&html, 400.0, 300.0);
        assert!(
            crate::toy::paint::png_contains_rgb(&page.png, 0x1f, 0x3d, 0x34),
            "expected spruce heading pixels in the painted PNG"
        );
    }

    #[test]
    fn painted_frame_is_a_png_data_url() {
        let html = fetch_document("ply://start").unwrap();
        let page = render_document(&html, 400.0, 300.0);
        let url = painted_frame_url(&page).unwrap();
        assert_eq!(url.scheme(), "data");
        assert!(url.as_str().starts_with("data:image/png;base64,"));
        assert!(url.as_str().len() > 100);
    }

    #[test]
    fn headless_toy_keeps_its_own_history() {
        let mut e = ToyEngine::headless();
        e.navigate("about:blank").unwrap();
        e.navigate("ply://start").unwrap();
        assert!(e.can_go_back());
        e.go_back().unwrap();
        assert_eq!(e.current_url().as_deref(), Some("about:blank"));
        e.go_forward().unwrap();
        assert_eq!(e.current_url().as_deref(), Some("ply://start"));
    }
}
// ai-generated end
