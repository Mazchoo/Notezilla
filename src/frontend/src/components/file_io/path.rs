/// Return the last path segment of `path`.
pub(crate) fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
}

/// Strip a trailing `.md` or `.markdown` suffix from `name`.
pub(crate) fn strip_md_ext(name: &str) -> &str {
    name.strip_suffix(".markdown")
        .or_else(|| name.strip_suffix(".md"))
        .unwrap_or(name)
}

/// Return a markdown download filename for `path`.
pub(crate) fn path_to_markdown_filename(path: &str) -> String {
    let name = basename(path);
    if name.ends_with(".md") || name.ends_with(".markdown") {
        name.to_string()
    } else {
        format!("{name}.md")
    }
}

/// Return an HTML download filename for `path`.
pub(crate) fn path_to_html_filename(path: &str) -> String {
    format!("{}.html", strip_md_ext(basename(path)))
}

/// Return a PDF download filename for `path`.
pub(crate) fn path_to_pdf_filename(path: &str) -> String {
    format!("{}.pdf", strip_md_ext(basename(path)))
}

/// Return the HTML page title derived from `path`.
pub(crate) fn html_page_title(path: &str) -> String {
    strip_md_ext(basename(path)).to_string()
}

/// Join a directory path and an entry name with `/`.
pub(crate) fn join_path(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

/// Return the parent directory of `path`, or `""` for a top-level entry.
pub(crate) fn parent_path(path: &str) -> &str {
    path.rsplit_once('/').map(|(p, _)| p).unwrap_or("")
}

/// Return whether moving `src` into directory `dst` is a no-op or invalid.
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

/// Return the new path of `path` after moving `src` into directory `dst`.
pub(crate) fn rewrite_path_after_move(path: &str, src: &str, dst: &str) -> Option<String> {
    let new_root = join_path(dst, basename(src));
    if path == src {
        return Some(new_root);
    }
    let prefix = format!("{src}/");
    path.strip_prefix(&prefix)
        .map(|rest| format!("{new_root}/{rest}"))
}

/// Resolve the basename after rename, matching backend `rename_basename` rules.
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

/// Return the new path of `path` after renaming `src` to `new_basename`.
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
    /// Assert basename returns the last slash or backslash segment.
    fn basename_returns_last_segment() {
        assert_eq!(basename("notes/hello.md"), "hello.md");
        assert_eq!(basename("notes\\hello.md"), "hello.md");
        assert_eq!(basename("hello.md"), "hello.md");
        assert_eq!(basename("notes/"), "notes/");
        assert_eq!(basename(""), "");
    }

    #[test]
    /// Assert markdown suffixes are stripped and other names are unchanged.
    fn strip_md_ext_drops_markdown_suffixes() {
        assert_eq!(strip_md_ext("hello.md"), "hello");
        assert_eq!(strip_md_ext("hello.markdown"), "hello");
        assert_eq!(strip_md_ext("hello.txt"), "hello.txt");
        assert_eq!(strip_md_ext("hello.md.bak"), "hello.md.bak");
    }

    #[test]
    /// Assert markdown download names keep an existing suffix or add `.md`.
    fn path_to_markdown_filename_keeps_or_adds_md() {
        assert_eq!(path_to_markdown_filename("notes/hello.md"), "hello.md");
        assert_eq!(
            path_to_markdown_filename("notes/hello.markdown"),
            "hello.markdown"
        );
        assert_eq!(path_to_markdown_filename("notes/hello"), "hello.md");
    }

    #[test]
    /// Assert HTML filenames use the basename and drop markdown extensions.
    fn path_to_html_filename_strips_md_and_uses_basename() {
        assert_eq!(path_to_html_filename("notes/hello.md"), "hello.html");
        assert_eq!(path_to_html_filename("notes/hello.markdown"), "hello.html");
        assert_eq!(path_to_html_filename("no-ext"), "no-ext.html");
    }

    #[test]
    /// Assert PDF filenames use the basename and drop markdown extensions.
    fn path_to_pdf_filename_strips_md_and_uses_basename() {
        assert_eq!(path_to_pdf_filename("notes/hello.md"), "hello.pdf");
        assert_eq!(path_to_pdf_filename("notes/hello.markdown"), "hello.pdf");
        assert_eq!(path_to_pdf_filename("notes\\windows.md"), "windows.pdf");
        assert_eq!(path_to_pdf_filename("no-ext"), "no-ext.pdf");
    }

    #[test]
    /// Assert HTML page titles use the basename without markdown extensions.
    fn html_page_title_strips_md_and_uses_basename() {
        assert_eq!(html_page_title("notes/hello.md"), "hello");
        assert_eq!(html_page_title("notes/hello.markdown"), "hello");
        assert_eq!(html_page_title("no-ext"), "no-ext");
    }

    #[test]
    /// Assert join_path inserts a slash except for an empty directory.
    fn join_path_joins_or_returns_name() {
        assert_eq!(join_path("notes", "a.md"), "notes/a.md");
        assert_eq!(join_path("", "a.md"), "a.md");
    }

    #[test]
    /// Assert parent_path returns the directory, or empty for a top-level entry.
    fn parent_path_returns_directory_or_empty() {
        assert_eq!(parent_path("notes/sub/a.md"), "notes/sub");
        assert_eq!(parent_path("a.md"), "");
    }

    #[test]
    /// Assert moves into the same parent, onto self, or into a descendant are invalid.
    fn is_invalid_move_rejects_noop_and_into_self() {
        assert!(is_invalid_move("", "notes"));
        assert!(is_invalid_move("notes/a.md", "notes"));
        assert!(is_invalid_move("notes", "notes"));
        assert!(is_invalid_move("notes", "notes/sub"));
        assert!(!is_invalid_move("notes/a.md", "other"));
        assert!(!is_invalid_move("notes-old", "notes"));
    }

    #[test]
    /// Assert rewrite_path_after_move remaps the moved path and its descendants.
    fn rewrite_path_after_move_updates_src_and_children() {
        assert_eq!(
            rewrite_path_after_move("notes/a.md", "notes/a.md", "other"),
            Some("other/a.md".into())
        );
        assert_eq!(
            rewrite_path_after_move("notes/sub/b.md", "notes", "archive"),
            Some("archive/notes/sub/b.md".into())
        );
        assert_eq!(
            rewrite_path_after_move("other/x.md", "notes", "archive"),
            None
        );
    }

    #[test]
    /// Assert file rename keeps the source suffix when the new name has none.
    fn resolved_rename_basename_preserves_file_suffix() {
        assert_eq!(resolved_rename_basename("a.md", "b", true), "b.md");
        assert_eq!(resolved_rename_basename("a.md", "b.txt", true), "b.txt");
        assert_eq!(resolved_rename_basename("folder", "renamed", false), "renamed");
        assert_eq!(resolved_rename_basename(".hidden", "x", true), "x");
        assert_eq!(resolved_rename_basename("noext", "x", true), "x");
    }

    #[test]
    /// Assert rewrite_path_after_rename remaps the renamed path and its descendants.
    fn rewrite_path_after_rename_updates_src_and_children() {
        assert_eq!(
            rewrite_path_after_rename("notes/a.md", "notes/a.md", "b.md"),
            Some("notes/b.md".into())
        );
        assert_eq!(
            rewrite_path_after_rename("notes/sub/x.md", "notes", "archive"),
            Some("archive/sub/x.md".into())
        );
        assert_eq!(
            rewrite_path_after_rename("other/x.md", "notes", "archive"),
            None
        );
    }
}
