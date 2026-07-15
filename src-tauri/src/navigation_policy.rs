use serde::Serialize;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NavigationDecision {
    Internal,
    External,
    Reject,
}

pub fn decide_navigation(raw_url: &str) -> NavigationDecision {
    let Ok(url) = Url::parse(raw_url) else {
        return NavigationDecision::Reject;
    };

    match url.scheme() {
        "mailto" | "tel" => return NavigationDecision::External,
        "https" | "http" => {}
        _ => return NavigationDecision::Reject,
    }

    let Some(host) = url.host_str().map(str::to_ascii_lowercase) else {
        return NavigationDecision::Reject;
    };

    if host == "libecity.com" || host.ends_with(".libecity.com") {
        NavigationDecision::Internal
    } else {
        NavigationDecision::External
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_libe_city_hosts() {
        assert_eq!(
            decide_navigation("https://libecity.com/"),
            NavigationDecision::Internal
        );
        assert_eq!(
            decide_navigation("https://library.libecity.com/a"),
            NavigationDecision::Internal
        );
    }

    #[test]
    fn rejects_deceptive_or_unsafe_urls() {
        assert_eq!(
            decide_navigation("https://libecity.com.example.org"),
            NavigationDecision::External
        );
        assert_eq!(
            decide_navigation("javascript:alert(1)"),
            NavigationDecision::Reject
        );
        assert_eq!(decide_navigation("not a url"), NavigationDecision::Reject);
    }
}
