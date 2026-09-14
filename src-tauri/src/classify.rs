// ai-generated
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassifyResult {
    NoOp,
    Url(String),
}

pub fn classify_input(input: &str) -> ClassifyResult {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ClassifyResult::NoOp;
    }
    if trimmed.chars().any(|c| c.is_whitespace()) {
        return ClassifyResult::Url(search_url(trimmed));
    }
    if let Ok(parsed) = url::Url::parse(trimmed) {
        match parsed.scheme() {
            "http" | "https" | "about" | "ply" => {
                return ClassifyResult::Url(parsed.into());
            }
            _ => {}
        }
    }
    if looks_like_host(trimmed) {
        let with_scheme = format!("https://{trimmed}");
        if let Ok(parsed) = url::Url::parse(&with_scheme) {
            return ClassifyResult::Url(parsed.into());
        }
    }
    ClassifyResult::Url(search_url(trimmed))
}

fn search_url(text: &str) -> String {
    let q: String = url::form_urlencoded::byte_serialize(text.as_bytes()).collect();
    format!("https://duckduckgo.com/?q={q}")
}

fn looks_like_host(s: &str) -> bool {
    let host = s.split('/').next().unwrap_or(s);
    let name = host.split(':').next().unwrap_or(host);
    name.eq_ignore_ascii_case("localhost") || name.contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_whitespace_are_noop() {
        assert_eq!(classify_input(""), ClassifyResult::NoOp);
        assert_eq!(classify_input("   "), ClassifyResult::NoOp);
        assert_eq!(classify_input("\t\n"), ClassifyResult::NoOp);
    }

    #[test]
    fn host_gets_https() {
        assert_eq!(
            classify_input("example.com"),
            ClassifyResult::Url("https://example.com/".to_string())
        );
    }

    #[test]
    fn absolute_http_s_unchanged() {
        assert_eq!(
            classify_input("https://a.test/x"),
            ClassifyResult::Url("https://a.test/x".to_string())
        );
        assert_eq!(
            classify_input("http://a.test/x"),
            ClassifyResult::Url("http://a.test/x".to_string())
        );
    }

    #[test]
    fn about_blank_unchanged() {
        assert_eq!(
            classify_input("about:blank"),
            ClassifyResult::Url("about:blank".to_string())
        );
    }

    #[test]
    fn ply_scheme_unchanged() {
        assert_eq!(
            classify_input("ply://start"),
            ClassifyResult::Url("ply://start".to_string())
        );
    }

    #[test]
    fn localhost_is_host() {
        assert_eq!(
            classify_input("localhost:8000"),
            ClassifyResult::Url("https://localhost:8000/".to_string())
        );
    }

    #[test]
    fn words_become_duckduckgo() {
        match classify_input("foo bar") {
            ClassifyResult::Url(u) => {
                assert!(u.starts_with("https://duckduckgo.com/?q="));
                assert!(u.contains("foo"));
                assert!(u.contains("bar"));
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn single_word_without_dot_is_search() {
        match classify_input(" Banff ") {
            ClassifyResult::Url(u) => {
                assert!(u.starts_with("https://duckduckgo.com/?q="));
                assert!(u.contains("Banff"));
            }
            other => panic!("{:?}", other),
        }
    }
}
// ai-generated end
