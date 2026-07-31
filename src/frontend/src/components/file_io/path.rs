pub(crate) fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
}

pub(crate) fn strip_md_ext(name: &str) -> &str {
    name.strip_suffix(".markdown")
        .or_else(|| name.strip_suffix(".md"))
        .unwrap_or(name)
}

pub(crate) fn path_to_markdown_filename(path: &str) -> String {
    let name = basename(path);
    if name.ends_with(".md") || name.ends_with(".markdown") {
        name.to_string()
    } else {
        format!("{name}.md")
    }
}

pub(crate) fn path_to_html_filename(path: &str) -> String {
    format!("{}.html", strip_md_ext(basename(path)))
}

pub(crate) fn html_page_title(path: &str) -> String {
    strip_md_ext(basename(path)).to_string()
}
