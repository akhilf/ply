// ai-generated
use crate::engine::{Engine, EngineEvent, EngineKind, LoadError};
use serde::Serialize;
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
    factory: Box<dyn Fn() -> Box<dyn Engine> + Send>,
}

impl TabManager {
    pub fn new(factory: Box<dyn Fn() -> Box<dyn Engine> + Send>) -> Self {
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
                self.write_tab(id, |t| {
                    t.url = url.to_string();
                    t.title = if url == "about:blank" {
                        "New Tab".to_string()
                    } else {
                        url.to_string()
                    };
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
            EngineEvent::UrlChanged { url } => self.write_tab(id, |t| {
                t.url = url;
                t.load_error = None;
            }),
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
        let mut engine = (self.factory)();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::FakeEngine;

    fn mgr() -> TabManager {
        TabManager::new(Box::new(|| Box::new(FakeEngine::new())))
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
}
// ai-generated end
