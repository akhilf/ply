// ai-generated
use crate::downloads::{DownloadItem, DownloadManager, DownloadStatus};
use crate::session::{BrowserSession, CommandOutcome, SessionSnapshot};
use crate::store::{BookmarkRow, VisitRow};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, EventTarget, Manager, State};
use tauri_plugin_opener::OpenerExt;

fn emit_outcome(app: &AppHandle, out: &CommandOutcome) {
    if out.emit {
        let _ = app.emit_to(
            EventTarget::webview("chrome"),
            "session_changed",
            &out.snapshot,
        );
    }
    if let Some(notice) = &out.notice {
        let _ = app.emit_to(EventTarget::webview("chrome"), "persist_notice", notice);
    }
}

fn with_session(
    app: &AppHandle,
    state: &State<'_, Mutex<BrowserSession>>,
    f: impl FnOnce(&mut BrowserSession) -> CommandOutcome,
) -> SessionSnapshot {
    let out = {
        let mut session = state.lock().expect("session");
        f(&mut session)
    };
    crate::shell::layout_webviews(app);
    emit_outcome(app, &out);
    out.snapshot
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, Mutex<BrowserSession>>) -> SessionSnapshot {
    state.lock().expect("session").snapshot()
}

#[tauri::command]
pub fn navigate(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
    input: String,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.navigate(&tab_id, &input))
}

#[tauri::command]
pub fn new_tab(app: AppHandle, state: State<'_, Mutex<BrowserSession>>) -> SessionSnapshot {
    with_session(&app, &state, BrowserSession::new_tab)
}

#[tauri::command]
pub fn close_tab(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.close_tab(&tab_id))
}

#[tauri::command]
pub fn switch_tab(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.switch_tab(&tab_id))
}

#[tauri::command]
pub fn go_back(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.go_back(&tab_id))
}

#[tauri::command]
pub fn go_forward(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.go_forward(&tab_id))
}

#[tauri::command]
pub fn reload(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.reload(&tab_id))
}

#[tauri::command]
pub fn toggle_bookmark(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.toggle_bookmark(&tab_id))
}

#[tauri::command]
pub fn list_bookmarks(state: State<'_, Mutex<BrowserSession>>) -> Vec<BookmarkRow> {
    state.lock().expect("session").list_bookmarks()
}

#[tauri::command]
pub fn list_history(state: State<'_, Mutex<BrowserSession>>, limit: Option<u32>) -> Vec<VisitRow> {
    state
        .lock()
        .expect("session")
        .list_history(limit.map(|n| n as usize))
}

#[tauri::command]
pub fn search_history(state: State<'_, Mutex<BrowserSession>>, query: String) -> Vec<VisitRow> {
    state.lock().expect("session").search_history(&query)
}

#[tauri::command]
pub fn set_library_open(app: AppHandle, open: bool) {
    crate::shell::set_library_open(&app, open);
}

#[tauri::command]
pub fn set_engine(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
    engine: crate::engine::EngineKind,
) -> SessionSnapshot {
    with_session(&app, &state, |session| session.set_engine(&tab_id, engine))
}

#[tauri::command]
pub fn list_downloads(state: State<'_, Mutex<DownloadManager>>) -> Vec<DownloadItem> {
    state.lock().expect("downloads").list()
}

#[tauri::command]
pub fn set_downloads_open(app: AppHandle, open: bool) {
    crate::shell::set_downloads_open(&app, open);
}

#[tauri::command]
pub fn open_download(
    app: AppHandle,
    state: State<'_, Mutex<DownloadManager>>,
    id: u64,
) -> Result<(), String> {
    let path = managed_download_path(&app, &state, id)?;
    let path = path
        .to_str()
        .ok_or_else(|| "invalid file path".to_string())?
        .to_string();
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|_| "couldn't open file".to_string())
}

#[tauri::command]
pub fn reveal_download(
    app: AppHandle,
    state: State<'_, Mutex<DownloadManager>>,
    id: u64,
) -> Result<(), String> {
    let path = managed_download_path(&app, &state, id)?;
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|_| "couldn't show file".to_string())
}

#[tauri::command]
pub fn clear_downloads(
    app: AppHandle,
    state: State<'_, Mutex<DownloadManager>>,
) -> Vec<DownloadItem> {
    let items = {
        let mut mgr = state.lock().expect("downloads");
        mgr.clear_finished();
        mgr.list()
    };
    crate::downloads::emit_list(&app);
    items
}

#[tauri::command]
pub fn set_find_open(app: AppHandle, open: bool) {
    crate::shell::set_find_open(&app, open);
}

#[tauri::command]
pub fn find_in_page(app: AppHandle, tab_id: String, query: String, backwards: bool) {
    let Some(webview) = app.get_webview(&tab_id) else {
        return;
    };
    let encoded = serde_json::to_string(&query).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        r#"(function() {{
  const q = {encoded};
  const backwards = {backwards};
  if (!q) {{
    const sel = window.getSelection && window.getSelection();
    if (sel) sel.removeAllRanges();
    return;
  }}
  window.find(q, false, backwards, true, false, false, false);
}})()"#
    );
    let _ = webview.eval(&script);
}

fn managed_download_path(
    app: &AppHandle,
    state: &State<'_, Mutex<DownloadManager>>,
    id: u64,
) -> Result<PathBuf, String> {
    let item = {
        let mgr = state.lock().expect("downloads");
        mgr.get(id)
            .cloned()
            .ok_or_else(|| "unknown download".to_string())?
    };
    if item.status != DownloadStatus::Done {
        return Err("file isn’t ready yet".to_string());
    }
    let folder = crate::downloads::download_folder(app)?;
    let path = PathBuf::from(&item.path);
    if !crate::downloads::path_is_under(&folder, &path) {
        return Err("file is outside the downloads folder".to_string());
    }
    Ok(path)
}
// ai-generated end
