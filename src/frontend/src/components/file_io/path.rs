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

pub(crate) fn path_to_pdf_filename(path: &str) -> String {
    format!("{}.pdf", strip_md_ext(basename(path)))
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

/// Basename after rename, matching backend `rename_basename` rules.
/// For files, if `new_name` has no extension and `src_path`'s basename has one,
/// the source extension is appended (e.g. `note.md` + `"renamed"` → `"renamed.md"`).
/// Folder names are returned unchanged.
pub(crate) fn resolved_rename_basename(src_path: &str, new_name: &str, is_file: bool) -> String {
    if !is_file {
        return new_name.to_string();
    }
    let src_base = basename(src_path);
    let src_suffix = match src_base.rfind('.') {
        Some(i) if i > 0 => &src_base[i..],
        _ => "",
    };
    let new_has_suffix = matches!(new_name.rfind('.'), Some(i) if i > 0);
    if !src_suffix.is_empty() && !new_has_suffix {
        format!("{new_name}{src_suffix}")
    } else {
        new_name.to_string()
    }
}

/// New path of `path` after renaming `src` to basename `new_basename`, if affected.
pub(crate) fn rewrite_path_after_rename(
    path: &str,
    src: &str,
    new_basename: &str,
) -> Option<String> {
    let new_root = join_path(parent_path(src), new_basename);
    if path == src {
        return Some(new_root);
    }
    let prefix = format!("{src}/");
    path.strip_prefix(&prefix)
        .map(|rest| format!("{new_root}/{rest}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_pdf_filename_strips_md_and_uses_basename() {
        assert_eq!(path_to_pdf_filename("notes/hello.md"), "hello.pdf");
        assert_eq!(path_to_pdf_filename("notes/hello.markdown"), "hello.pdf");
        assert_eq!(path_to_pdf_filename("notes\\windows.md"), "windows.pdf");
        assert_eq!(path_to_pdf_filename("no-ext"), "no-ext.pdf");
    }
}
