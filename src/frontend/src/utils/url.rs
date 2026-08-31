/// Return a URL with a trailing FQDN dot stripped from `hostname`, if present.
fn url_without_trailing_dot_host(
    protocol: &str,
    hostname: &str,
    port: &str,
    rest: &str,
) -> Option<String> {
    let hostname = hostname.strip_suffix('.')?;
    if hostname.is_empty() {
        return None;
    }
    let host = if port.is_empty() {
        hostname.to_string()
    } else {
        format!("{hostname}:{port}")
    };
    Some(format!("{protocol}//{host}{rest}"))
}

/// Reload when the hostname ends with a DNS-root trailing dot.
pub(crate) fn redirect_trailing_dot_hostname() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let location = window.location();
    let (Ok(protocol), Ok(hostname), Ok(port), Ok(pathname), Ok(search), Ok(hash)) = (
        location.protocol(),
        location.hostname(),
        location.port(),
        location.pathname(),
        location.search(),
        location.hash(),
    ) else {
        return false;
    };
    let Some(url) = url_without_trailing_dot_host(
        &protocol,
        &hostname,
        &port,
        &format!("{pathname}{search}{hash}"),
    ) else {
        return false;
    };
    let _ = location.replace(&url);
    true
}

#[cfg(test)]
mod tests {
    use super::url_without_trailing_dot_host;

    #[test]
    /// Assert a trailing FQDN dot is stripped from the hostname and left unchanged otherwise.
    fn url_without_trailing_dot_host_strips_dns_root_dot() {
        assert_eq!(
            url_without_trailing_dot_host("http:", "localhost.", "8080", "/"),
            Some("http://localhost:8080/".into())
        );
        assert_eq!(
            url_without_trailing_dot_host("http:", "localhost", "8080", "/"),
            None
        );
    }
}
