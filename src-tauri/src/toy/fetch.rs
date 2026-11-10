// ai-generated
use crate::engine::LoadError;
use std::io::Read;

const MAX_BODY: usize = 1024 * 1024;
const START_HTML: &str = include_str!("../../../fixtures/start.html");

pub fn fetch_document(url: &str) -> Result<String, LoadError> {
    let bytes = fetch_bytes(url)?;
    String::from_utf8(bytes).map_err(|_| LoadError::LoadFailed)
}

pub fn fetch_bytes(url: &str) -> Result<Vec<u8>, LoadError> {
    match scheme(url) {
        Some("about") => {
            if url == "about:blank" {
                Ok(b"<html><body></body></html>".to_vec())
            } else {
                Err(LoadError::Unreachable)
            }
        }
        Some("ply") => fixture_bytes(url),
        Some("http") | Some("https") => http_get(url),
        _ => Err(LoadError::InvalidUrl),
    }
}

fn scheme(url: &str) -> Option<&'static str> {
    let parsed = url::Url::parse(url).ok()?;
    match parsed.scheme() {
        "http" => Some("http"),
        "https" => Some("https"),
        "ply" => Some("ply"),
        "about" => Some("about"),
        _ => None,
    }
}

fn fixture_bytes(url: &str) -> Result<Vec<u8>, LoadError> {
    let parsed = url::Url::parse(url).map_err(|_| LoadError::InvalidUrl)?;
    if parsed.scheme() != "ply" {
        return Err(LoadError::InvalidUrl);
    }
    let name = parsed
        .host_str()
        .unwrap_or("")
        .trim_end_matches(".html")
        .to_ascii_lowercase();
    match name.as_str() {
        "start" => Ok(START_HTML.as_bytes().to_vec()),
        _ => Err(LoadError::Unreachable),
    }
}

fn http_get(url: &str) -> Result<Vec<u8>, LoadError> {
    let parsed = url::Url::parse(url).map_err(|_| LoadError::InvalidUrl)?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(LoadError::InvalidUrl);
    }
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(15))
        .call()
        .map_err(|err| match err {
            ureq::Error::Status(_, _) => LoadError::LoadFailed,
            ureq::Error::Transport(_) => LoadError::Unreachable,
        })?;
    let mut buf = Vec::new();
    response
        .into_reader()
        .take(MAX_BODY as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|_| LoadError::LoadFailed)?;
    cap_body(buf)
}

fn cap_body(bytes: Vec<u8>) -> Result<Vec<u8>, LoadError> {
    if bytes.len() > MAX_BODY {
        Err(LoadError::LoadFailed)
    } else {
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_blank_is_empty_document() {
        let html = fetch_document("about:blank").unwrap();
        assert!(html.to_ascii_lowercase().contains("body"));
    }

    #[test]
    fn ply_start_loads_fixture() {
        let html = fetch_document("ply://start").unwrap();
        assert!(html.contains("Ply toy engine"));
    }

    #[test]
    fn missing_fixture_is_unreachable() {
        assert_eq!(
            fetch_document("ply://missing").unwrap_err(),
            LoadError::Unreachable
        );
    }

    #[test]
    fn javascript_and_file_are_invalid() {
        assert_eq!(
            fetch_document("javascript:alert(1)").unwrap_err(),
            LoadError::InvalidUrl
        );
        assert_eq!(
            fetch_document("file:///etc/passwd").unwrap_err(),
            LoadError::InvalidUrl
        );
    }
}
// ai-generated end
