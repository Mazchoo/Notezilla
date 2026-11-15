//! Derives GitHub-style anchor ids for rendered headings, so in-document
//! links such as `[Intro](#1-introduction)` have a target.

use std::collections::HashMap;

/// Hands out heading anchor ids, numbering repeats as `slug-1`, `slug-2`, ….
#[derive(Default)]
pub struct HeadingSlugs {
    seen: HashMap<String, usize>,
}

impl HeadingSlugs {
    /// Return the anchor id for a heading whose plain text is `text`.
    pub fn next(&mut self, text: &str) -> String {
        let slug = slugify(text);
        match self.seen.get_mut(&slug) {
            Some(count) => {
                *count += 1;
                format!("{slug}-{count}")
            }
            None => {
                self.seen.insert(slug.clone(), 0);
                slug
            }
        }
    }
}

/// Lowercase `text`, drop punctuation, and turn spaces into hyphens.
fn slugify(text: &str) -> String {
    text.trim()
        .chars()
        .filter_map(|ch| match ch {
            ' ' => Some('-'),
            '-' | '_' => Some(ch),
            _ if ch.is_alphanumeric() => Some(ch),
            _ => None,
        })
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{slugify, HeadingSlugs};

    #[test]
    /// Assert punctuation is dropped and each space becomes a hyphen.
    fn slugify_matches_github_anchors() {
        assert_eq!(
            slugify("3. Flowcharts (flowchart / graph)"),
            "3-flowcharts-flowchart--graph"
        );
        assert_eq!(
            slugify("Styling, CSS Classes, & Link Styling"),
            "styling-css-classes--link-styling"
        );
        assert_eq!(slugify("snake_case Über"), "snake_case-über");
    }

    #[test]
    /// Assert repeated headings get numbered suffixes.
    fn repeated_headings_are_numbered() {
        let mut slugs = HeadingSlugs::default();
        assert_eq!(slugs.next("Example"), "example");
        assert_eq!(slugs.next("Example"), "example-1");
        assert_eq!(slugs.next("Example"), "example-2");
    }
}
