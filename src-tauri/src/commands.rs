// ai-generated
use crate::session::{BrowserSession, CommandOutcome, SessionSnapshot};
use crate::store::{BookmarkRow, VisitRow};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, EventTarget, State};

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
    let out = state.lock().expect("session").navigate(&tab_id, &input);
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn new_tab(app: AppHandle, state: State<'_, Mutex<BrowserSession>>) -> SessionSnapshot {
    let out = state.lock().expect("session").new_tab();
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn close_tab(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    let out = state.lock().expect("session").close_tab(&tab_id);
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn switch_tab(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    let out = state.lock().expect("session").switch_tab(&tab_id);
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn go_back(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    let out = state.lock().expect("session").go_back(&tab_id);
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn go_forward(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    let out = state.lock().expect("session").go_forward(&tab_id);
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn reload(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    let out = state.lock().expect("session").reload(&tab_id);
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn toggle_bookmark(
    app: AppHandle,
    state: State<'_, Mutex<BrowserSession>>,
    tab_id: String,
) -> SessionSnapshot {
    let out = state.lock().expect("session").toggle_bookmark(&tab_id);
    emit_outcome(&app, &out);
    out.snapshot
}

#[tauri::command]
pub fn list_bookmarks(state: State<'_, Mutex<BrowserSession>>) -> Vec<BookmarkRow> {
    state.lock().expect("session").list_bookmarks()
}

#[tauri::command]
pub fn list_history(
    state: State<'_, Mutex<BrowserSession>>,
    limit: Option<u32>,
) -> Vec<VisitRow> {
    state
        .lock()
        .expect("session")
        .list_history(limit.map(|n| n as usize))
}

#[tauri::command]
pub fn search_history(
    state: State<'_, Mutex<BrowserSession>>,
    query: String,
) -> Vec<VisitRow> {
    state.lock().expect("session").search_history(&query)
}
// ai-generated end
