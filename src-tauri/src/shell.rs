// ai-generated
use crate::engine::{Engine, LoadError};
use crate::session::BrowserSession;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::webview::{DownloadEvent, WebviewBuilder};
use tauri::{
    AppHandle, Emitter, EventTarget, Listener, LogicalPosition, LogicalSize, Manager, WebviewUrl,
    WindowEvent,
};

pub const CHROME_HEIGHT: f64 = 92.0;

pub(crate) fn ensure_content_webview(app: &AppHandle, label: &str) -> Result<(), LoadError> {
    if app.get_webview(label).is_some() {
        return Ok(());
    }
    let window = app.get_window("main").ok_or(LoadError::LoadFailed)?;
    attach_content_webview(app, &window, label).map_err(|_| LoadError::LoadFailed)?;
    if app.get_webview(label).is_none() {
        return Err(LoadError::LoadFailed);
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChromeOverlay {
    None,
    Library,
    Downloads,
}

pub struct OverlayState(pub Mutex<ChromeOverlay>);

pub struct FindOpen(pub AtomicBool);

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

    fn ensure_attached(&self) -> Result<(), LoadError> {
        if self.webview().is_some() {
            return Ok(());
        }
        let window = self.app.get_window("main").ok_or(LoadError::LoadFailed)?;
        attach_content_webview(&self.app, &window, &self.webview_label)
            .map_err(|_| LoadError::LoadFailed)?;
        if self.webview().is_none() {
            return Err(LoadError::LoadFailed);
        }
        Ok(())
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
        self.ensure_attached()?;
        let webview = self.webview().ok_or(LoadError::LoadFailed)?;
        let assigned = webview.navigate(parsed.clone());
        if assigned.is_err() {
            let href = serde_json::to_string(url).map_err(|_| LoadError::LoadFailed)?;
            webview
                .eval(&format!("window.location.replace({href})"))
                .map_err(|_| LoadError::LoadFailed)?;
        }
        let _ = webview.show();
        self.push_history(url);
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
}

fn attach_content_webview(
    app: &AppHandle,
    window: &tauri::Window,
    label: &str,
) -> tauri::Result<()> {
    let (pos, size) = content_slot(window, chrome_height_of(app))?;
    let blank = url::Url::parse("about:blank").expect("about:blank");
    let app_for_nav = app.clone();
    let tab_id = label.to_string();
    let builder = WebviewBuilder::new(label, WebviewUrl::External(blank))
        .on_navigation(move |url| {
            if url.scheme() == "data" || url.scheme() == "about" {
                return true;
            }
            let app = app_for_nav.clone();
            let payload = EngineUrlPayload {
                tab_id: tab_id.clone(),
                url: url.to_string(),
            };
            let _ = std::thread::spawn(move || {
                let _ = app.emit("engine_url", payload);
            });
            true
        })
        .on_document_title_changed({
            let app_for_title = app.clone();
            let title_tab = label.to_string();
            move |_webview, title| {
                let app = app_for_title.clone();
                let payload = EngineTitlePayload {
                    tab_id: title_tab.clone(),
                    title,
                };
                let _ = std::thread::spawn(move || {
                    let _ = app.emit("engine_title", payload);
                });
            }
        })
        .on_download({
            let app_for_dl = app.clone();
            move |_webview, event| match event {
                DownloadEvent::Requested { url, destination } => {
                    crate::downloads::on_requested(&app_for_dl, &url, destination)
                }
                DownloadEvent::Finished { url, path, success } => {
                    crate::downloads::on_finished(&app_for_dl, &url, path, success);
                    true
                }
                _ => true,
            }
        });
    let webview = window.add_child(builder, pos, size)?;
    let _ = webview.show();
    Ok(())
}

const FIND_EXTRA: f64 = 40.0;

fn chrome_height_of(app: &AppHandle) -> f64 {
    let mut height = CHROME_HEIGHT;
    if find_is_open(app) {
        height += FIND_EXTRA;
    }
    height
}

fn window_full_slot(
    window: &tauri::Window,
) -> tauri::Result<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let scale = window.scale_factor()?;
    let logical = window.inner_size()?.to_logical::<f64>(scale);
    Ok((
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(logical.width, logical.height),
    ))
}

fn content_slot(
    window: &tauri::Window,
    chrome_h: f64,
) -> tauri::Result<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let scale = window.scale_factor()?;
    let logical = window.inner_size()?.to_logical::<f64>(scale);
    let height = (logical.height - chrome_h).max(1.0);
    Ok((
        LogicalPosition::new(0.0, chrome_h),
        LogicalSize::new(logical.width, height),
    ))
}

fn chrome_slot(
    window: &tauri::Window,
    chrome_h: f64,
) -> tauri::Result<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let scale = window.scale_factor()?;
    let logical = window.inner_size()?.to_logical::<f64>(scale);
    Ok((
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(logical.width, chrome_h),
    ))
}

fn tab_labels_and_active(app: &AppHandle) -> (Vec<String>, String) {
    let state = app.state::<Mutex<BrowserSession>>();
    let Ok(session) = state.lock() else {
        return (Vec::new(), String::new());
    };
    let snap = session.snapshot();
    (
        snap.tabs.iter().map(|t| t.id.clone()).collect(),
        snap.active_tab_id,
    )
}

fn overlay_of(app: &AppHandle) -> ChromeOverlay {
    app.try_state::<OverlayState>()
        .and_then(|s| s.0.lock().ok().map(|g| *g))
        .unwrap_or(ChromeOverlay::None)
}

fn set_overlay(app: &AppHandle, overlay: ChromeOverlay) {
    if let Some(state) = app.try_state::<OverlayState>() {
        if let Ok(mut guard) = state.0.lock() {
            *guard = overlay;
        }
    }
    layout_webviews(app);
}

pub fn set_library_open(app: &AppHandle, open: bool) {
    let current = overlay_of(app);
    let next = if open {
        ChromeOverlay::Library
    } else if current == ChromeOverlay::Library {
        ChromeOverlay::None
    } else {
        current
    };
    set_overlay(app, next);
}

pub fn set_downloads_open(app: &AppHandle, open: bool) {
    let current = overlay_of(app);
    let next = if open {
        ChromeOverlay::Downloads
    } else if current == ChromeOverlay::Downloads {
        ChromeOverlay::None
    } else {
        current
    };
    set_overlay(app, next);
}

fn find_is_open(app: &AppHandle) -> bool {
    app.try_state::<FindOpen>()
        .map(|s| s.0.load(Ordering::Relaxed))
        .unwrap_or(false)
}

pub fn set_find_open(app: &AppHandle, open: bool) {
    if let Some(state) = app.try_state::<FindOpen>() {
        state.0.store(open, Ordering::Relaxed);
    }
    layout_webviews(app);
}

pub fn layout_webviews(app: &AppHandle) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    let overlay_open = overlay_of(app) != ChromeOverlay::None;
    let (labels, active) = tab_labels_and_active(app);
    if overlay_open {
        if let (Ok((pos, size)), Some(chrome)) =
            (window_full_slot(&window), app.get_webview("chrome"))
        {
            let _ = chrome.set_position(pos);
            let _ = chrome.set_size(size);
        }
        for label in labels {
            if let Some(webview) = app.get_webview(&label) {
                let _ = webview.hide();
            }
        }
        return;
    }
    let chrome_h = chrome_height_of(app);
    if let (Ok((cpos, csize)), Some(chrome)) =
        (chrome_slot(&window, chrome_h), app.get_webview("chrome"))
    {
        let _ = chrome.set_position(cpos);
        let _ = chrome.set_size(csize);
    }
    let Ok((pos, size)) = content_slot(&window, chrome_h) else {
        return;
    };
    for label in labels {
        if let Some(webview) = app.get_webview(&label) {
            let _ = webview.set_position(pos);
            let _ = webview.set_size(size);
            if label == active {
                let _ = webview.show();
            } else {
                let _ = webview.hide();
            }
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

    let (cpos, csize) = chrome_slot(&window, CHROME_HEIGHT)?;
    let chrome = WebviewBuilder::new("chrome", WebviewUrl::App("index.html".into()));
    let chrome = window.add_child(chrome, cpos, csize)?;
    let _ = chrome.set_auto_resize(false);

    let data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    let _ = std::fs::create_dir_all(&data_dir);
    let store = Store::open_path(&data_dir.join("ply.sqlite"));
    app.manage(OverlayState(Mutex::new(ChromeOverlay::None)));
    app.manage(FindOpen(AtomicBool::new(false)));
    app.manage(Mutex::new(crate::downloads::DownloadManager::new()));
    let factory_app = app.clone();
    let session = BrowserSession::new(
        store,
        Box::new(move |id, kind| match kind {
            crate::engine::EngineKind::Native => {
                Box::new(NativeEngine::new(factory_app.clone(), id.to_string()))
            }
            crate::engine::EngineKind::Toy => Box::new(crate::toy::ToyEngine::new(
                factory_app.clone(),
                id.to_string(),
            )),
        }),
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
        if matches!(
            event,
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
        ) {
            layout_webviews(&layout_app);
        }
    });

    layout_webviews(app);

    Ok(())
}
// ai-generated end
