// ai-generated
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum EngineKind {
    Native,
    Toy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadError {
    InvalidUrl,
    Unreachable,
    LoadFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EngineEvent {
    UrlChanged { url: String },
    TitleChanged { title: String },
    LoadFailed { kind: LoadError },
}

pub trait Engine {
    fn navigate(&mut self, url: &str) -> Result<(), LoadError>;
    fn go_back(&mut self) -> Result<(), LoadError>;
    fn go_forward(&mut self) -> Result<(), LoadError>;
    fn reload(&mut self) -> Result<(), LoadError>;
    fn set_visible(&mut self, visible: bool);
    fn can_go_back(&self) -> bool;
    fn can_go_forward(&self) -> bool;
    fn current_url(&self) -> Option<String>;
}

pub struct FakeEngine {
    history: Vec<String>,
    index: Option<usize>,
    navigations: Vec<String>,
    reload_count: u32,
    #[allow(dead_code)]
    visible: bool,
}

impl FakeEngine {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            index: None,
            navigations: Vec::new(),
            reload_count: 0,
            visible: true,
        }
    }

    pub fn navigations(&self) -> &[String] {
        &self.navigations
    }

    pub fn reload_count(&self) -> u32 {
        self.reload_count
    }
}

impl Engine for FakeEngine {
    fn navigate(&mut self, url: &str) -> Result<(), LoadError> {
        self.navigations.push(url.to_string());
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
                self.index = Some(i - 1);
                Ok(())
            }
            _ => Err(LoadError::LoadFailed),
        }
    }

    fn go_forward(&mut self) -> Result<(), LoadError> {
        match self.index {
            Some(i) if i + 1 < self.history.len() => {
                self.index = Some(i + 1);
                Ok(())
            }
            _ => Err(LoadError::LoadFailed),
        }
    }

    fn reload(&mut self) -> Result<(), LoadError> {
        if self.index.is_none() {
            return Err(LoadError::LoadFailed);
        }
        self.reload_count += 1;
        Ok(())
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_then_back_and_forward() {
        let mut e = FakeEngine::new();
        e.navigate("about:blank").unwrap();
        e.set_visible(true);
        assert!(!e.can_go_back());
        e.navigate("https://a.test/").unwrap();
        assert!(e.can_go_back());
        assert!(!e.can_go_forward());
        e.go_back().unwrap();
        assert!(!e.can_go_back());
        assert!(e.can_go_forward());
        assert_eq!(e.current_url().as_deref(), Some("about:blank"));
        e.go_forward().unwrap();
        assert!(e.can_go_back());
        assert_eq!(
            e.navigations(),
            &["about:blank".to_string(), "https://a.test/".to_string()]
        );
    }

    #[test]
    fn reload_does_not_add_history() {
        let mut e = FakeEngine::new();
        e.navigate("https://a.test/").unwrap();
        e.reload().unwrap();
        assert!(!e.can_go_back());
        assert_eq!(e.reload_count(), 1);
        assert_eq!(e.navigations().len(), 1);
    }

    #[test]
    fn go_back_at_start_is_load_failed() {
        let mut e = FakeEngine::new();
        e.navigate("https://a.test/").unwrap();
        let err = e.go_back().unwrap_err();
        assert_eq!(err, LoadError::LoadFailed);
    }
}
// ai-generated end
