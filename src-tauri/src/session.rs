// ai-generated
use crate::classify::{classify_input, ClassifyResult};
use crate::engine::{Engine, EngineEvent, EngineKind, LoadError};
use crate::store::{BookmarkRow, PersistMode, Store, VisitRow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon_url: String,
    pub engine: EngineKind,
    pub load_error: Option<LoadError>,
}

pub struct TabManager {
    next_id: u64,
    tabs: Vec<Tab>,
    active: String,
    engines: HashMap<String, Box<dyn Engine>>,
    factory: Box<dyn Fn(&str, EngineKind) -> Box<dyn Engine> + Send>,
}

impl TabManager {
    pub fn new(factory: Box<dyn Fn(&str, EngineKind) -> Box<dyn Engine> + Send>) -> Self {
        let mut m = Self {
            next_id: 0,
            tabs: Vec::new(),
            active: String::new(),
            engines: HashMap::new(),
            factory,
        };
        m.spawn_blank();
        m
    }

    pub fn new_tab(&mut self) -> String {
        self.spawn_blank()
    }

    pub fn close_tab(&mut self, id: &str) {
        let idx = self.tabs.iter().position(|t| t.id == id);
        let Some(idx) = idx else {
            return;
        };
        let was_active = self.active == id;
        self.tabs.remove(idx);
        self.engines.remove(id);
        if self.tabs.is_empty() {
            self.spawn_blank();
            return;
        }
        if was_active {
            let next = if idx == 0 { 0 } else { idx - 1 };
            let next_id = self.tabs[next].id.clone();
            self.activate(&next_id);
        }
    }

    pub fn switch_tab(&mut self, id: &str) -> Result<(), LoadError> {
        if !self.tabs.iter().any(|t| t.id == id) {
            return Err(LoadError::LoadFailed);
        }
        self.activate(id);
        Ok(())
    }

    pub fn navigate_url(&mut self, id: &str, url: &str) -> Result<(), LoadError> {
        let engine = self.engines.get_mut(id).ok_or(LoadError::LoadFailed)?;
        match engine.navigate(url) {
            Ok(()) => {
                let painted = self.engines.get(id).and_then(|e| e.last_title());
                self.write_tab(id, |t| {
                    t.url = url.to_string();
                    t.title = painted.filter(|s| !s.is_empty()).unwrap_or_else(|| {
                        if url == "about:blank" {
                            "New Tab".to_string()
                        } else {
                            url.to_string()
                        }
                    });
                    t.load_error = None;
                });
                Ok(())
            }
            Err(err) => {
                self.write_tab(id, |t| {
                    t.url = url.to_string();
                    t.load_error = Some(err);
                });
                Err(err)
            }
        }
    }

    pub fn go_back(&mut self, id: &str) -> Result<(), LoadError> {
        self.engines
            .get_mut(id)
            .ok_or(LoadError::LoadFailed)?
            .go_back()?;
        self.sync_url_from_engine(id);
        Ok(())
    }

    pub fn go_forward(&mut self, id: &str) -> Result<(), LoadError> {
        self.engines
            .get_mut(id)
            .ok_or(LoadError::LoadFailed)?
            .go_forward()?;
        self.sync_url_from_engine(id);
        Ok(())
    }

    pub fn reload(&mut self, id: &str) -> Result<(), LoadError> {
        self.engines
            .get_mut(id)
            .ok_or(LoadError::LoadFailed)?
            .reload()
    }

    pub fn set_engine(&mut self, id: &str, kind: EngineKind) -> Result<(), LoadError> {
        let url = self
            .tab(id)
            .map(|t| t.url.clone())
            .ok_or(LoadError::LoadFailed)?;
        let visible = self.active == id;
        self.engines.remove(id);
        let mut engine = (self.factory)(id, kind);
        engine.set_visible(visible);
        let nav = engine.navigate(&url);
        self.engines.insert(id.to_string(), engine);
        let painted = self.engines.get(id).and_then(|e| e.last_title());
        self.write_tab(id, |t| {
            t.engine = kind;
            t.load_error = nav.as_ref().err().copied();
            if nav.is_ok() {
                if let Some(title) = painted {
                    t.title = title;
                }
            }
        });
        nav
    }

    fn sync_url_from_engine(&mut self, id: &str) {
        let url = self.engines.get(id).and_then(|e| e.current_url());
        if let Some(url) = url {
            self.write_tab(id, |t| {
                t.url = url;
                t.load_error = None;
            });
        }
    }

    pub fn apply_event(&mut self, id: &str, event: EngineEvent) {
        match event {
            EngineEvent::UrlChanged { url } => {
                if let Some(engine) = self.engines.get_mut(id) {
                    engine.observe_url(&url);
                }
                self.write_tab(id, |t| {
                    if t.title == "New Tab" && url != "about:blank" {
                        t.title = url.clone();
                    }
                    t.url = url;
                    t.load_error = None;
                });
            }
            EngineEvent::TitleChanged { title } => self.write_tab(id, |t| {
                if t.url == "about:blank" && title.is_empty() {
                    t.title = "New Tab".to_string();
                } else {
                    t.title = title;
                }
            }),
            EngineEvent::LoadFailed { kind } => self.write_tab(id, |t| {
                t.load_error = Some(kind);
            }),
        }
    }

    pub fn active_tab_id(&self) -> &str {
        &self.active
    }

    pub fn tab(&self, id: &str) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn can_go_back(&self) -> bool {
        self.engines
            .get(&self.active)
            .map(|e| e.can_go_back())
            .unwrap_or(false)
    }

    pub fn can_go_forward(&self) -> bool {
        self.engines
            .get(&self.active)
            .map(|e| e.can_go_forward())
            .unwrap_or(false)
    }

    fn alloc(&mut self) -> String {
        self.next_id += 1;
        format!("tab-{}", self.next_id)
    }

    fn spawn_blank(&mut self) -> String {
        let id = self.alloc();
        let mut engine = (self.factory)(&id, EngineKind::Native);
        let _ = engine.navigate("about:blank");
        let tab = Tab {
            id: id.clone(),
            title: "New Tab".to_string(),
            url: "about:blank".to_string(),
            favicon_url: String::new(),
            engine: EngineKind::Native,
            load_error: None,
        };
        self.tabs.push(tab);
        self.engines.insert(id.clone(), engine);
        self.activate(&id);
        id
    }

    fn activate(&mut self, id: &str) {
        self.active = id.to_string();
        let ids: Vec<String> = self.engines.keys().cloned().collect();
        for key in ids {
            if let Some(engine) = self.engines.get_mut(&key) {
                engine.set_visible(key == id);
            }
        }
    }

    fn write_tab(&mut self, id: &str, f: impl FnOnce(&mut Tab)) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            f(tab);
        }
    }

    #[cfg(test)]
    pub fn engine_visible(&self, id: &str) -> bool {
        self.engines
            .get(id)
            .map(|e| e.is_visible())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionSnapshot {
    pub tabs: Vec<Tab>,
    pub active_tab_id: String,
    pub current_url: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_bookmarked: bool,
    pub load_error: Option<LoadError>,
    pub persist_mode: PersistMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeKind {
    Error,
    #[allow(dead_code)]
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistNotice {
    pub kind: NoticeKind,
    pub message: String,
}

pub struct CommandOutcome {
    pub snapshot: SessionSnapshot,
    pub notice: Option<PersistNotice>,
    pub emit: bool,
}

pub struct BrowserSession {
    tabs: TabManager,
    store: Store,
}

impl BrowserSession {
    pub fn new(
        store: Store,
        factory: Box<dyn Fn(&str, EngineKind) -> Box<dyn Engine> + Send>,
    ) -> Self {
        Self {
            tabs: TabManager::new(factory),
            store,
        }
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        let active_id = self.tabs.active_tab_id().to_string();
        let active = self.tabs.tab(&active_id);
        let current_url = active.map(|t| t.url.clone()).unwrap_or_default();
        let load_error = active.and_then(|t| t.load_error);
        let is_bookmarked = current_url != "about:blank" && self.store.is_bookmarked(&current_url);
        SessionSnapshot {
            tabs: self.tabs.tabs().to_vec(),
            active_tab_id: active_id,
            current_url,
            can_go_back: self.tabs.can_go_back(),
            can_go_forward: self.tabs.can_go_forward(),
            is_bookmarked,
            load_error,
            persist_mode: self.store.persist_mode(),
        }
    }

    pub fn navigate(&mut self, tab_id: &str, input: &str) -> CommandOutcome {
        match classify_input(input) {
            ClassifyResult::NoOp => CommandOutcome {
                snapshot: self.snapshot(),
                notice: None,
                emit: false,
            },
            ClassifyResult::Url(url) => {
                let previous = self.current_url_of(tab_id);
                let _ = self.tabs.navigate_url(tab_id, &url);
                let title = self
                    .tabs
                    .tab(tab_id)
                    .map(|t| t.title.clone())
                    .unwrap_or_else(|| url.clone());
                let notice = self.maybe_visit(&previous, &url, &title);
                CommandOutcome {
                    snapshot: self.snapshot(),
                    notice,
                    emit: true,
                }
            }
        }
    }

    pub fn new_tab(&mut self) -> CommandOutcome {
        self.tabs.new_tab();
        self.emit_ok(None)
    }

    pub fn close_tab(&mut self, tab_id: &str) -> CommandOutcome {
        self.tabs.close_tab(tab_id);
        self.emit_ok(None)
    }

    pub fn switch_tab(&mut self, tab_id: &str) -> CommandOutcome {
        let _ = self.tabs.switch_tab(tab_id);
        self.emit_ok(None)
    }

    pub fn go_back(&mut self, tab_id: &str) -> CommandOutcome {
        let previous = self.current_url_of(tab_id);
        let _ = self.tabs.go_back(tab_id);
        let notice = self.visit_active(tab_id, &previous);
        self.emit_ok(notice)
    }

    pub fn go_forward(&mut self, tab_id: &str) -> CommandOutcome {
        let previous = self.current_url_of(tab_id);
        let _ = self.tabs.go_forward(tab_id);
        let notice = self.visit_active(tab_id, &previous);
        self.emit_ok(notice)
    }

    pub fn reload(&mut self, tab_id: &str) -> CommandOutcome {
        let _ = self.tabs.reload(tab_id);
        self.emit_ok(None)
    }

    pub fn set_engine(&mut self, tab_id: &str, kind: EngineKind) -> CommandOutcome {
        let previous = self.current_url_of(tab_id);
        let _ = self.tabs.set_engine(tab_id, kind);
        let notice = self.visit_active(tab_id, &previous);
        self.emit_ok(notice)
    }

    pub fn toggle_bookmark(&mut self, tab_id: &str) -> CommandOutcome {
        let Some(tab) = self.tabs.tab(tab_id) else {
            return self.emit_ok(None);
        };
        if tab.url == "about:blank" {
            return self.emit_ok(None);
        }
        let url = tab.url.clone();
        let title = tab.title.clone();
        let notice = match self.store.toggle_bookmark(&url, &title) {
            Ok(_) => None,
            Err(_) => Some(PersistNotice {
                kind: NoticeKind::Error,
                message: "Couldn't save bookmark".to_string(),
            }),
        };
        self.emit_ok(notice)
    }

    pub fn list_bookmarks(&self) -> Vec<BookmarkRow> {
        self.store.list_bookmarks()
    }

    pub fn list_history(&self, limit: Option<usize>) -> Vec<VisitRow> {
        self.store.list_history(limit.unwrap_or(100))
    }

    pub fn search_history(&self, query: &str) -> Vec<VisitRow> {
        self.store.search_history(query)
    }

    pub fn apply_event(&mut self, tab_id: &str, event: EngineEvent) -> CommandOutcome {
        let previous = self.current_url_of(tab_id);
        self.tabs.apply_event(tab_id, event);
        let notice = self.visit_active(tab_id, &previous);
        self.emit_ok(notice)
    }

    fn current_url_of(&self, tab_id: &str) -> String {
        self.tabs
            .tab(tab_id)
            .map(|t| t.url.clone())
            .unwrap_or_default()
    }

    fn visit_active(&mut self, tab_id: &str, previous: &str) -> Option<PersistNotice> {
        let tab = self.tabs.tab(tab_id)?;
        let url = tab.url.clone();
        let title = tab.title.clone();
        self.maybe_visit(previous, &url, &title)
    }

    fn maybe_visit(&mut self, previous: &str, url: &str, title: &str) -> Option<PersistNotice> {
        let is_http = url.starts_with("http://") || url.starts_with("https://");
        if !is_http || url == previous {
            return None;
        }
        match self.store.append_visit(url, title) {
            Ok(()) => None,
            Err(_) => Some(PersistNotice {
                kind: NoticeKind::Error,
                message: "Couldn't save history".to_string(),
            }),
        }
    }

    fn emit_ok(&self, notice: Option<PersistNotice>) -> CommandOutcome {
        CommandOutcome {
            snapshot: self.snapshot(),
            notice,
            emit: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::FakeEngine;

    fn mgr() -> TabManager {
        TabManager::new(Box::new(|_id, _kind| Box::new(FakeEngine::new())))
    }

    #[test]
    fn set_engine_reloads_without_history() {
        let mut m = mgr();
        let id = m.active_tab_id().to_string();
        m.navigate_url(&id, "https://a.test/").unwrap();
        m.navigate_url(&id, "https://b.test/").unwrap();
        assert!(m.can_go_back());
        m.set_engine(&id, EngineKind::Toy).unwrap();
        assert_eq!(m.tab(&id).unwrap().engine, EngineKind::Toy);
        assert_eq!(m.tab(&id).unwrap().url, "https://b.test/");
        assert!(!m.can_go_back());
    }

    #[test]
    fn set_engine_unknown_tab_fails() {
        let mut m = mgr();
        let err = m.set_engine("tab-999", EngineKind::Toy).unwrap_err();
        assert_eq!(err, LoadError::LoadFailed);
    }

    #[test]
    fn starts_with_one_blank_tab() {
        let m = mgr();
        assert_eq!(m.tabs().len(), 1);
        let t = m.tab(m.active_tab_id()).unwrap();
        assert_eq!(t.url, "about:blank");
        assert_eq!(t.title, "New Tab");
        assert_eq!(t.id, "tab-1");
    }

    #[test]
    fn new_tab_activates_and_increments_id() {
        let mut m = mgr();
        let id = m.new_tab();
        assert_eq!(id, "tab-2");
        assert_eq!(m.active_tab_id(), "tab-2");
        assert_eq!(m.tabs().len(), 2);
    }

    #[test]
    fn switch_hides_previous_engine() {
        let mut m = mgr();
        let first = m.active_tab_id().to_string();
        let second = m.new_tab();
        assert!(m.engine_visible(&second));
        assert!(!m.engine_visible(&first));
        m.switch_tab(&first).unwrap();
        assert!(m.engine_visible(&first));
        assert!(!m.engine_visible(&second));
    }

    #[test]
    fn close_last_tab_opens_blank() {
        let mut m = mgr();
        let id = m.active_tab_id().to_string();
        m.close_tab(&id);
        assert_eq!(m.tabs().len(), 1);
        assert_eq!(m.tab(m.active_tab_id()).unwrap().url, "about:blank");
        assert_ne!(m.active_tab_id(), id);
    }

    #[test]
    fn close_active_selects_neighbor() {
        let mut m = mgr();
        let first = m.active_tab_id().to_string();
        let second = m.new_tab();
        m.close_tab(&second);
        assert_eq!(m.active_tab_id(), first);
        assert_eq!(m.tabs().len(), 1);
    }

    #[test]
    fn navigate_and_back_flags_match_engine() {
        let mut m = mgr();
        let id = m.active_tab_id().to_string();
        m.navigate_url(&id, "https://a.test/").unwrap();
        assert_eq!(m.tab(&id).unwrap().url, "https://a.test/");
        assert!(m.can_go_back());
        m.go_back(&id).unwrap();
        assert!(m.can_go_forward());
        assert_eq!(m.tab(&id).unwrap().url, "about:blank");
        m.go_forward(&id).unwrap();
        assert_eq!(m.tab(&id).unwrap().url, "https://a.test/");
    }

    #[test]
    fn reload_does_not_change_back_flag() {
        let mut m = mgr();
        let id = m.active_tab_id().to_string();
        m.navigate_url(&id, "https://a.test/").unwrap();
        assert!(m.can_go_back());
        m.reload(&id).unwrap();
        assert!(m.can_go_back());
    }

    #[test]
    fn unknown_tab_is_load_failed() {
        let mut m = mgr();
        let err = m.switch_tab("tab-999").unwrap_err();
        assert_eq!(err, LoadError::LoadFailed);
    }

    #[test]
    fn apply_event_updates_title() {
        let mut m = mgr();
        let id = m.active_tab_id().to_string();
        m.navigate_url(&id, "https://a.test/").unwrap();
        m.apply_event(
            &id,
            EngineEvent::TitleChanged {
                title: "Hi".to_string(),
            },
        );
        assert_eq!(m.tab(&id).unwrap().title, "Hi");
    }

    #[test]
    fn in_page_url_change_enables_back() {
        let mut m = mgr();
        let id = m.active_tab_id().to_string();
        assert!(!m.can_go_back());
        m.apply_event(
            &id,
            EngineEvent::UrlChanged {
                url: "https://a.test/".to_string(),
            },
        );
        assert_eq!(m.tab(&id).unwrap().url, "https://a.test/");
        assert!(m.can_go_back());
    }

    fn sess() -> (tempfile::TempDir, BrowserSession) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open_path(&dir.path().join("ply.sqlite"));
        let session =
            BrowserSession::new(store, Box::new(|_id, _kind| Box::new(FakeEngine::new())));
        (dir, session)
    }

    #[test]
    fn noop_empty_navigate() {
        let (_dir, mut s) = sess();
        let before = s.snapshot().current_url;
        let out = s.navigate(&s.snapshot().active_tab_id.clone(), "  ");
        assert!(!out.emit);
        assert_eq!(out.snapshot.current_url, before);
    }

    #[test]
    fn navigate_records_visit() {
        let (_dir, mut s) = sess();
        let id = s.snapshot().active_tab_id.clone();
        let out = s.navigate(&id, "example.com");
        assert!(out.emit);
        assert_eq!(out.snapshot.current_url, "https://example.com/");
        assert_eq!(s.list_history(None).len(), 1);
    }

    #[test]
    fn bookmark_blank_noop() {
        let (_dir, mut s) = sess();
        let id = s.snapshot().active_tab_id.clone();
        s.toggle_bookmark(&id);
        assert!(s.list_bookmarks().is_empty());
    }

    #[test]
    fn memory_store_notice_on_visit() {
        let mut s = BrowserSession::new(
            Store::open_memory_only(),
            Box::new(|_id, _kind| Box::new(FakeEngine::new())),
        );
        assert_eq!(s.snapshot().persist_mode, PersistMode::Memory);
        let id = s.snapshot().active_tab_id.clone();
        let out = s.navigate(&id, "https://a.test/");
        assert_eq!(out.notice.unwrap().message, "Couldn't save history");
        assert_eq!(out.snapshot.current_url, "https://a.test/");
    }

    #[test]
    fn session_commands_roundtrip() {
        let (_dir, mut s) = sess();
        let first = s.snapshot().active_tab_id.clone();
        let created = s.new_tab();
        let second = created.snapshot.active_tab_id.clone();
        s.navigate(&second, "https://a.test/");
        s.switch_tab(&first);
        s.close_tab(&second);
        let id = s.snapshot().active_tab_id.clone();
        s.navigate(&id, "https://b.test/");
        s.go_back(&id);
        s.go_forward(&id);
        s.reload(&id);
        assert!(s.search_history("b").len() >= 1);
        s.apply_event(
            &id,
            EngineEvent::TitleChanged {
                title: "Beta".to_string(),
            },
        );
        assert_eq!(s.snapshot().tabs[0].title, "Beta");
    }
}
// ai-generated end
