use super::super::path::diagram_svg_filename;
use super::{download_text_file, log_export_error, yield_for_paint};
use crate::components::toast::show_error_toast;
use crate::constants::SVG_MIME;
use crate::info_messages::{export_failed_toast, export_progress_label};
use crate::models::block::EditorEntry;
use crate::rendering::{diagram_fences, DiagramKind};
use crate::theme::current_theme;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Prompt the browser to save every mermaid and graphviz diagram as an SVG file.
pub fn export_entries_as_svg_diagrams(
    entries: Vec<EditorEntry>,
    progress: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
) {
    if progress.get_untracked().is_some() || entries.is_empty() {
        return;
    }
    let jobs = diagram_export_jobs(&entries);
    if jobs.is_empty() {
        return;
    }
    let total = jobs.len();
    progress.set(Some(export_progress_label(&jobs[0].0, 0, total)));

    spawn_local(async move {
        let mut errors = Vec::new();
        for (i, (filename, kind, source)) in jobs.into_iter().enumerate() {
            if i > 0 {
                progress.set(Some(export_progress_label(&filename, i, total)));
            }
            yield_for_paint().await;
            match kind.render_svg(&source) {
                Ok(svg) => {
                    let svg = with_page_background(&svg);
                    if let Err(err) = download_text_file(&filename, &svg, SVG_MIME) {
                        errors.push(log_export_error(export_failed_toast(
                            &filename,
                            &format!("{err:?}"),
                        )));
                    }
                }
                Err(e) => {
                    errors.push(log_export_error(export_failed_toast(&filename, &e)));
                }
            }
        }
        progress.set(None);
        if !errors.is_empty() {
            show_error_toast(error_toast, errors.join("\n"));
        }
    });
}

/// Return download names, kinds, and sources for every mermaid and graphviz fence.
fn diagram_export_jobs(entries: &[EditorEntry]) -> Vec<(String, DiagramKind, String)> {
    let mut jobs = Vec::new();
    for entry in entries {
        let path = entry.title.path.get_untracked();
        let markdown = entry.content.text.get_untracked();
        for (index, (kind, source)) in diagram_fences(&markdown).into_iter().enumerate() {
            jobs.push((
                diagram_svg_filename(&path, kind.as_str(), index),
                kind,
                source,
            ));
        }
    }
    jobs
}

/// Return `svg` with a page-fill rect so the editor background is kept on export.
fn with_page_background(svg: &str) -> String {
    let Some(start) = svg.find("<svg") else {
        return svg.to_string();
    };
    let Some(rel_end) = svg[start..].find('>') else {
        return svg.to_string();
    };
    let open_end = start + rel_end;
    if svg.as_bytes().get(open_end.saturating_sub(1)) == Some(&b'/') {
        return svg.to_string();
    }
    let insert_at = open_end + 1;
    let fill = current_theme().palette().bg_2;
    let rect = format!(r#"<rect width="100%" height="100%" fill="{fill}"/>"#);
    let mut out = String::with_capacity(svg.len() + rect.len());
    out.push_str(&svg[..insert_at]);
    out.push_str(&rect);
    out.push_str(&svg[insert_at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::BG_2;

    #[test]
    /// Assert diagram exports are named `{stem}_{type}_{index}.svg` in document order.
    fn diagram_export_jobs_names_each_diagram() {
        use crate::models::block::EditorEntry;
        use leptos::prelude::Owner;

        let owner = Owner::new();
        owner.with(|| {
            let markdown = "```mermaid\ngraph LR\n    A --> B\n```\n\n\
```graphviz\ndigraph { A -> B }\n```\n";
            let entry = EditorEntry::new("./folder/new-markdown.md", markdown);
            let jobs = diagram_export_jobs(&[entry]);
            assert_eq!(jobs.len(), 2);
            assert_eq!(jobs[0].0, "new-markdown_mermaid_0.svg");
            assert_eq!(jobs[1].0, "new-markdown_graphviz_1.svg");
            assert_eq!(jobs[0].1, DiagramKind::Mermaid);
            assert_eq!(jobs[1].1, DiagramKind::Graphviz);
            assert!(jobs[0].2.contains("graph LR"), "{}", jobs[0].2);
            assert!(jobs[1].2.contains("digraph"), "{}", jobs[1].2);
        });
    }

    #[test]
    /// Assert exported SVG paints the editor page behind the diagram.
    fn with_page_background_inserts_a_fill_rect() {
        let out = with_page_background(r#"<svg viewBox="0 0 10 10"><g/></svg>"#);
        let rect = format!(r#"<rect width="100%" height="100%" fill="{BG_2}"/>"#);
        assert_eq!(out, format!(r#"<svg viewBox="0 0 10 10">{rect}<g/></svg>"#));
        assert_eq!(with_page_background("<g/>"), "<g/>");

        let _guard = crate::theme::ThemeGuard::set(crate::theme::ColorTheme::Day);
        let day = with_page_background(r#"<svg viewBox="0 0 10 10"><g/></svg>"#);
        let day_rect = format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            crate::constants::DAY_BG_2
        );
        assert_eq!(
            day,
            format!(r#"<svg viewBox="0 0 10 10">{day_rect}<g/></svg>"#)
        );
    }
}
