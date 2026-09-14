// ai-generated
use crate::engine::{Engine, LoadError};
use crate::session::BrowserSession;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::webview::WebviewBuilder;
use tauri::{
    AppHandle, Emitter, EventTarget, Listener, LogicalPosition, LogicalSize, Manager, WebviewUrl,
    WindowEvent,
};

pub const CHROME_HEIGHT: f64 = 92.0;

#[derive(Clone, Serialize, Deserialize)]
struct EngineUrlPayload {
    tab_id: String,
    url: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct EngineTitlePayload {
    tab_id: String,
    title: String,
}

pub struct NativeEngine {
    app: AppHandle,
    webview_label: String,
    history: Vec<String>,
    index: Option<usize>,
    visible: bool,
}

impl NativeEngine {
    pub fn new(app: AppHandle, label: String) -> Self {
        if app.get_webview(&label).is_none() {
            if let Some(window) = app.get_window("main") {
                let _ = attach_content_webview(&app, &window, &label);
            }
        }
        Self {
            app,
            webview_label: label,
            history: Vec::new(),
            index: None,
            visible: true,
        }
    }

    fn webview(&self) -> Option<tauri::Webview> {
        self.app.get_webview(&self.webview_label)
    }
}

impl Drop for NativeEngine {
    fn drop(&mut self) {
        if let Some(webview) = self.webview() {
            let _ = webview.close();
        }
    }
}

impl Engine for NativeEngine {
    fn navigate(&mut self, url: &str) -> Result<(), LoadError> {
        let parsed = url::Url::parse(url).map_err(|_| LoadError::InvalidUrl)?;
        let webview = self.webview().ok_or(LoadError::LoadFailed)?;
        webview.navigate(parsed).map_err(|_| LoadError::LoadFailed)?;
        if let Some(i) = self.index {
            self.history.truncate(i + 1);
        }
        self.history.push(url.to_string());
        self.index = Some(self.history.len() - 1);
        Ok(())
    }

    fn go_back(&mut self) -> Result<(), LoadError> {
        match self.index {
            Some(i) if i > 0 => {
                let webview = self.webview().ok_or(LoadError::LoadFailed)?;
                webview
                    .eval("history.back()")
                    .map_err(|_| LoadError::LoadFailed)?;
                self.index = Some(i - 1);
                Ok(())
            }
            _ => Err(LoadError::LoadFailed),
        }
    }

    fn go_forward(&mut self) -> Result<(), LoadError> {
        match self.index {
            Some(i) if i + 1 < self.history.len() => {
                let webview = self.webview().ok_or(LoadError::LoadFailed)?;
                webview
                    .eval("history.forward()")
                    .map_err(|_| LoadError::LoadFailed)?;
                self.index = Some(i + 1);
                Ok(())
            }
            _ => Err(LoadError::LoadFailed),
        }
    }

    fn reload(&mut self) -> Result<(), LoadError> {
        let webview = self.webview().ok_or(LoadError::LoadFailed)?;
        webview
            .eval("location.reload()")
            .map_err(|_| LoadError::LoadFailed)?;
        Ok(())
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if let Some(webview) = self.webview() {
            if visible {
                let _ = webview.show();
            } else {
                let _ = webview.hide();
            }
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
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
}

fn attach_content_webview(
    app: &AppHandle,
    window: &tauri::Window,
    label: &str,
) -> tauri::Result<()> {
    let (pos, size) = content_slot(window)?;
    let blank = url::Url::parse("about:blank").expect("about:blank");
    let app_for_nav = app.clone();
    let tab_id = label.to_string();
    let builder = WebviewBuilder::new(label, WebviewUrl::External(blank))
        .on_navigation(move |url| {
            let _ = app_for_nav.emit(
                "engine_url",
                EngineUrlPayload {
                    tab_id: tab_id.clone(),
                    url: url.to_string(),
                },
            );
            true
        })
        .on_document_title_changed({
            let app_for_title = app.clone();
            let title_tab = label.to_string();
            move |_webview, title| {
                let _ = app_for_title.emit(
                    "engine_title",
                    EngineTitlePayload {
                        tab_id: title_tab.clone(),
                        title,
                    },
                );
            }
        });
    window.add_child(builder, pos, size)?;
    Ok(())
}

fn content_slot(window: &tauri::Window) -> tauri::Result<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let scale = window.scale_factor()?;
    let logical = window.inner_size()?.to_logical::<f64>(scale);
    let height = (logical.height - CHROME_HEIGHT).max(1.0);
    Ok((
        LogicalPosition::new(0.0, CHROME_HEIGHT),
        LogicalSize::new(logical.width, height),
    ))
}

fn chrome_slot(window: &tauri::Window) -> tauri::Result<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let scale = window.scale_factor()?;
    let logical = window.inner_size()?.to_logical::<f64>(scale);
    Ok((
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(logical.width, CHROME_HEIGHT),
    ))
}

pub fn layout_webviews(app: &AppHandle) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    if let (Ok((cpos, csize)), Some(chrome)) = (chrome_slot(&window), app.get_webview("chrome")) {
        let _ = chrome.set_position(cpos);
        let _ = chrome.set_size(csize);
    }
    let Ok((pos, size)) = content_slot(&window) else {
        return;
    };
    let labels: Vec<String> = {
        let state = app.state::<Mutex<BrowserSession>>();
        let Ok(session) = state.lock() else {
            return;
        };
        session.snapshot().tabs.iter().map(|t| t.id.clone()).collect()
    };
    for label in labels {
        if let Some(webview) = app.get_webview(&label) {
            let _ = webview.set_position(pos);
            let _ = webview.set_size(size);
        }
    }
}

fn emit_session(app: &AppHandle, out: crate::session::CommandOutcome) {
    if out.emit {
        let _ = app.emit_to(
            EventTarget::webview("chrome"),
            "session_changed",
            &out.snapshot,
        );
    }
    if let Some(notice) = out.notice {
        let _ = app.emit_to(EventTarget::webview("chrome"), "persist_notice", notice);
    }
}

fn apply_engine_event(app: &AppHandle, tab_id: &str, event: crate::engine::EngineEvent) {
    let out = {
        let state = app.state::<Mutex<BrowserSession>>();
        let Ok(mut session) = state.try_lock() else {
            return;
        };
        session.apply_event(tab_id, event)
    };
    emit_session(app, out);
}

pub fn create_main_window(app: &AppHandle) -> tauri::Result<()> {
    let window = tauri::window::WindowBuilder::new(app, "main")
        .title("Ply")
        .inner_size(1100.0, 760.0)
        .build()?;

    let (cpos, csize) = chrome_slot(&window)?;
    let chrome = WebviewBuilder::new("chrome", WebviewUrl::App("index.html".into()));
    window.add_child(chrome, cpos, csize)?;

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::temp_dir());
    let _ = std::fs::create_dir_all(&data_dir);
    let store = Store::open_path(&data_dir.join("ply.sqlite"));
    let factory_app = app.clone();
    let session = BrowserSession::new(
        store,
        Box::new(move |id| Box::new(NativeEngine::new(factory_app.clone(), id.to_string()))),
    );
    app.manage(Mutex::new(session));

    let listen_app = app.clone();
    let _ = app.listen("engine_url", move |event| {
        let Ok(payload) = serde_json::from_str::<EngineUrlPayload>(event.payload()) else {
            return;
        };
        apply_engine_event(
            &listen_app,
            &payload.tab_id,
            crate::engine::EngineEvent::UrlChanged { url: payload.url },
        );
    });

    let title_app = app.clone();
    let _ = app.listen("engine_title", move |event| {
        let Ok(payload) = serde_json::from_str::<EngineTitlePayload>(event.payload()) else {
            return;
        };
        apply_engine_event(
            &title_app,
            &payload.tab_id,
            crate::engine::EngineEvent::TitleChanged {
                title: payload.title,
            },
        );
    });

    let layout_app = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Resized(_)) {
            layout_webviews(&layout_app);
        }
    });

    Ok(())
}
// ai-generated end
