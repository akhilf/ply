// ai-generated
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, EventTarget, Manager};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Saving,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadItem {
    pub id: u64,
    pub url: String,
    pub filename: String,
    pub path: String,
    pub status: DownloadStatus,
    pub started_at: i64,
}

pub struct DownloadManager {
    next_id: u64,
    items: Vec<DownloadItem>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            items: Vec::new(),
        }
    }

    pub fn begin(&mut self, url: String, path: PathBuf) -> DownloadItem {
        self.next_id += 1;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "download".to_string());
        let item = DownloadItem {
            id: self.next_id,
            url,
            filename,
            path: path.to_string_lossy().into_owned(),
            status: DownloadStatus::Saving,
            started_at: now_secs(),
        };
        self.items.insert(0, item.clone());
        item
    }

    pub fn finish(
        &mut self,
        url: &str,
        path: Option<&Path>,
        success: bool,
    ) -> Option<DownloadItem> {
        let status = if success {
            DownloadStatus::Done
        } else {
            DownloadStatus::Failed
        };
        if let Some(path) = path {
            let path_s = path.to_string_lossy();
            if let Some(item) = self
                .items
                .iter_mut()
                .find(|item| item.status == DownloadStatus::Saving && item.path == path_s)
            {
                item.status = status;
                return Some(item.clone());
            }
        }
        let item = self
            .items
            .iter_mut()
            .find(|item| item.status == DownloadStatus::Saving && item.url == url)?;
        item.status = status;
        Some(item.clone())
    }

    pub fn list(&self) -> Vec<DownloadItem> {
        self.items.clone()
    }

    pub fn get(&self, id: u64) -> Option<&DownloadItem> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn clear_finished(&mut self) {
        self.items
            .retain(|item| item.status == DownloadStatus::Saving);
    }
}

pub fn should_accept_url(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}

pub fn filename_from_url(url: &Url) -> String {
    let last = url
        .path_segments()
        .and_then(|segments| {
            segments
                .rev()
                .find(|part| !part.is_empty())
                .map(|part| percent_decode_str(part).decode_utf8_lossy().into_owned())
        })
        .unwrap_or_default();
    sanitize_filename(&last)
}

pub fn sanitize_filename(raw: &str) -> String {
    let trimmed = raw.trim();
    let leaf = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed).trim();
    let mut out = String::new();
    for ch in leaf.chars() {
        if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*') {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    let out = out.trim_matches('.').trim().to_string();
    if out.is_empty() || out == "." || out == ".." {
        return "download".to_string();
    }
    if out.chars().count() > 180 {
        out.chars().take(180).collect()
    } else {
        out
    }
}

pub fn unique_destination(dir: &Path, filename: &str) -> PathBuf {
    let safe = sanitize_filename(filename);
    let path = dir.join(&safe);
    if !path.exists() {
        return path;
    }
    let stem = Path::new(&safe)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let ext = Path::new(&safe).extension().and_then(|s| s.to_str());
    for n in 1..10_000 {
        let candidate = match ext {
            Some(ext) => dir.join(format!("{stem} ({n}).{ext}")),
            None => dir.join(format!("{stem} ({n})")),
        };
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!("{stem}-{}.part", std::process::id()))
}

pub fn download_folder(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().app_data_dir().map(|p| p.join("downloads")))
        .map_err(|_| "downloads folder unavailable".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|_| "couldn't create downloads folder".to_string())?;
    Ok(dir)
}

pub fn path_is_under(dir: &Path, file: &Path) -> bool {
    let Ok(dir) = dir.canonicalize() else {
        return false;
    };
    let Ok(file) = file.canonicalize() else {
        return false;
    };
    file.starts_with(dir)
}

pub fn on_requested(app: &AppHandle, url: &Url, destination: &mut PathBuf) -> bool {
    if !should_accept_url(url) {
        return false;
    }
    let Ok(dir) = download_folder(app) else {
        return false;
    };
    let suggested = destination
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| filename_from_url(url));
    let dest = unique_destination(&dir, &suggested);
    *destination = dest.clone();
    {
        let state = app.state::<Mutex<DownloadManager>>();
        let Ok(mut mgr) = state.lock() else {
            return false;
        };
        mgr.begin(url.to_string(), dest);
    }
    emit_list(app);
    true
}

pub fn on_finished(app: &AppHandle, url: &Url, path: Option<PathBuf>, success: bool) {
    {
        let state = app.state::<Mutex<DownloadManager>>();
        let Ok(mut mgr) = state.lock() else {
            return;
        };
        mgr.finish(&url.to_string(), path.as_deref(), success);
    }
    emit_list(app);
}

pub fn emit_list(app: &AppHandle) {
    let items = {
        let state = app.state::<Mutex<DownloadManager>>();
        let Ok(mgr) = state.lock() else {
            return;
        };
        mgr.list()
    };
    let _ = app.emit_to(EventTarget::webview("chrome"), "downloads_changed", &items);
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_http_family() {
        assert!(should_accept_url(
            &Url::parse("https://a.test/file.pdf").unwrap()
        ));
        assert!(should_accept_url(
            &Url::parse("http://a.test/file.pdf").unwrap()
        ));
        assert!(!should_accept_url(
            &Url::parse("data:text/plain,hi").unwrap()
        ));
        assert!(!should_accept_url(&Url::parse("file:///tmp/x").unwrap()));
        assert!(!should_accept_url(&Url::parse("ply://start").unwrap()));
    }

    #[test]
    fn filename_from_url_uses_last_segment() {
        let url = Url::parse("https://a.test/docs/report.pdf?dl=1").unwrap();
        assert_eq!(filename_from_url(&url), "report.pdf");
    }

    #[test]
    fn filename_from_url_decodes_and_falls_back() {
        let encoded = Url::parse("https://a.test/files/foo%20bar.zip").unwrap();
        assert_eq!(filename_from_url(&encoded), "foo bar.zip");
        let root = Url::parse("https://a.test/").unwrap();
        assert_eq!(filename_from_url(&root), "download");
        let traversal = Url::parse("https://a.test/%2e%2e").unwrap();
        assert_eq!(filename_from_url(&traversal), "download");
    }

    #[test]
    fn sanitize_strips_separators_and_reserved() {
        assert_eq!(sanitize_filename("a/b\\c:d*.pdf"), "c_d_.pdf");
        assert_eq!(sanitize_filename(".."), "download");
        assert_eq!(sanitize_filename(""), "download");
    }

    #[test]
    fn unique_destination_adds_counter() {
        let dir = tempfile::tempdir().unwrap();
        let first = unique_destination(dir.path(), "note.txt");
        std::fs::write(&first, b"one").unwrap();
        let second = unique_destination(dir.path(), "note.txt");
        assert_eq!(second.file_name().unwrap(), "note (1).txt");
        std::fs::write(&second, b"two").unwrap();
        let third = unique_destination(dir.path(), "note.txt");
        assert_eq!(third.file_name().unwrap(), "note (2).txt");
    }

    #[test]
    fn manager_begin_finish_and_clear() {
        let mut mgr = DownloadManager::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let started = mgr.begin("https://a.test/a.bin".into(), path.clone());
        assert_eq!(started.id, 1);
        assert_eq!(started.status, DownloadStatus::Saving);
        assert_eq!(mgr.list().len(), 1);
        let done = mgr
            .finish("https://a.test/a.bin", Some(path.as_path()), true)
            .unwrap();
        assert_eq!(done.status, DownloadStatus::Done);
        mgr.clear_finished();
        assert!(mgr.list().is_empty());
    }

    #[test]
    fn finish_matches_url_when_path_missing() {
        let mut mgr = DownloadManager::new();
        let dir = tempfile::tempdir().unwrap();
        mgr.begin("https://a.test/a.bin".into(), dir.path().join("a.bin"));
        let done = mgr.finish("https://a.test/a.bin", None, false).unwrap();
        assert_eq!(done.status, DownloadStatus::Failed);
    }

    #[test]
    fn newest_saving_item_wins_for_same_url() {
        let mut mgr = DownloadManager::new();
        let dir = tempfile::tempdir().unwrap();
        mgr.begin("https://a.test/a.bin".into(), dir.path().join("a.bin"));
        mgr.begin("https://a.test/a.bin".into(), dir.path().join("a (1).bin"));
        let done = mgr.finish("https://a.test/a.bin", None, true).unwrap();
        assert_eq!(done.id, 2);
        assert_eq!(mgr.get(1).unwrap().status, DownloadStatus::Saving);
        assert_eq!(mgr.get(2).unwrap().status, DownloadStatus::Done);
    }

    #[test]
    fn path_is_under_rejects_outside() {
        let dir = tempfile::tempdir().unwrap();
        let inside = dir.path().join("ok.txt");
        std::fs::write(&inside, b"ok").unwrap();
        let outside = std::env::temp_dir().join("ply-download-outside.txt");
        std::fs::write(&outside, b"no").unwrap();
        assert!(path_is_under(dir.path(), &inside));
        assert!(!path_is_under(dir.path(), &outside));
        let _ = std::fs::remove_file(&outside);
    }
}
// ai-generated end
