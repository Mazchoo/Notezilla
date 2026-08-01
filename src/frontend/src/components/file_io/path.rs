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

/// Join a directory path and an entry name with `/`.
/// An empty `dir` yields `name` alone (note-folder root).
pub(crate) fn join_path(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

/// Parent directory of `path`, or `""` for a top-level entry.
pub(crate) fn parent_path(path: &str) -> &str {
    path.rsplit_once('/').map(|(p, _)| p).unwrap_or("")
}

/// Whether moving `src` into directory `dst` is a no-op or invalid.
pub(crate) fn is_invalid_move(src: &str, dst: &str) -> bool {
    if src.is_empty() {
        return true;
    }
    if parent_path(src) == dst {
        return true;
    }
    if src == dst {
        return true;
    }
    dst.starts_with(src) && dst.as_bytes().get(src.len()) == Some(&b'/')
}

/// New path of `path` after moving `src` into directory `dst`, if affected.
pub(crate) fn rewrite_path_after_move(path: &str, src: &str, dst: &str) -> Option<String> {
    let new_root = join_path(dst, basename(src));
    if path == src {
        return Some(new_root);
    }
    let prefix = format!("{src}/");
    path.strip_prefix(&prefix)
        .map(|rest| format!("{new_root}/{rest}"))
}
