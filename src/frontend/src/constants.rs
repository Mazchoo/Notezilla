pub const DEFAULT_MARKDOWN_PATH: &str = "./example_folder/new_markdown.md";
/// Editor path for a blank template created from the templates tree.
pub const NEW_TEMPLATE_PATH: &str = "./new_template.md";
/// Editor path of the mermaid help manual opened from the help dropdown.
pub const MERMAID_HELP_MANUAL_PATH: &str = "./mermaid_help_manual.md";
pub const MERMAID_HELP_MANUAL: &str = include_str!("../static/mermaid_help_manual.md");
/// Editor path of the graphviz help manual opened from the help dropdown.
pub const GRAPHVIZ_HELP_MANUAL_PATH: &str = "./graphviz_manual.md";
pub const GRAPHVIZ_HELP_MANUAL: &str = include_str!("../static/graphviz_manual.md");
/// Editor path of the LaTeX math help manual opened from the help dropdown.
pub const LATEX_MATH_HELP_MANUAL_PATH: &str = "./latex_math_core_manual.md";
pub const LATEX_MATH_HELP_MANUAL: &str = include_str!("../static/latex_math_core_manual.md");

pub const SIDEBAR_MIN_WIDTH: f64 = 160.0;
pub const SIDEBAR_MAX_WIDTH: f64 = 600.0;

/// Proxied by Trunk to http://127.0.0.1:8020 in development.
pub const MCP_URL: &str = "/mcp";
/// SSE stream of filesystem change pings. Proxied by Trunk to the FastMCP backend.
pub const FILE_CHANGE_EVENTS_URL: &str = "/events";
pub const CONTENT_TYPE_HEADER: &str = "Content-Type";
pub const ACCEPT_HEADER: &str = "Accept";
pub const JSON_CONTENT_TYPE: &str = "application/json";
pub const MCP_ACCEPT: &str = "application/json, text/event-stream";
pub const MCP_SESSION_HEADER: &str = "mcp-session-id";
/// Prefix the MCP server puts on a failed tool's text content.
pub const MCP_TOOL_ERROR_PREFIX: &str = "Error: ";
/// Text content the MCP server returns for a successful tool call.
pub const MCP_TOOL_SUCCESS: &str = "Success";
/// Loopback host of the local Ollama HTTP API. The TCP port is user-configurable.
pub const OLLAMA_HOST: &str = "127.0.0.1";
pub const OLLAMA_TAGS_PATH: &str = "/api/tags";
pub const OLLAMA_GENERATE_PATH: &str = "/api/generate";
pub const OLLAMA_TEMPERATURE_MIN: f64 = 0.0;
pub const OLLAMA_TEMPERATURE_MAX: f64 = 2.0;
pub const OLLAMA_TOP_P_MIN: f64 = 0.0;
pub const OLLAMA_TOP_P_MAX: f64 = 1.0;
/// Gemini Generative Language API origin.
pub const GEMINI_API_ORIGIN: &str = "https://generativelanguage.googleapis.com";
/// Path prefix for Gemini model metadata: `/v1beta/models/{model}`.
pub const GEMINI_MODELS_PATH: &str = "/v1beta/models";
/// Action suffix for POST `{model}:generateContent`.
pub const GEMINI_GENERATE_CONTENT_ACTION: &str = "generateContent";

/// Return the Gemini model metadata URL for `model` and `api_key`.
pub fn gemini_model_url(model: &str, api_key: &str) -> String {
    format!("{GEMINI_API_ORIGIN}{GEMINI_MODELS_PATH}/{model}?key={api_key}")
}

pub const DRAG_MIME: &str = "text/plain";
/// MIME type of downloaded mermaid and graphviz SVG files.
pub const SVG_MIME: &str = "image/svg+xml;charset=utf-8";

pub const EXPORT_TEMPLATE: &str = include_str!("../static/export.html");
pub const EXPORT_PDF_TEMPLATE: &str = include_str!("../static/export-pdf.html");
pub const NIGHT_CSS: &str = include_str!("../night.css");
pub const DAY_CSS: &str = include_str!("../day.css");
/// Style sheet math-core's MathML needs: `<menclose>`, table padding, wide accents.
pub const MATH_CSS: &str = include_str!("../math.css");
pub const PROMPT_TEMPLATE: &str = include_str!("../static/prompt.md");

/// Placeholder shown for markdown images until real image serving exists.
pub const IMAGE_MISSING_SVG: &str = include_str!("../static/image-missing.svg");
/// Checkmark drawn on a checked settings checkbox. Served as a static file.
#[allow(dead_code)]
pub const CHECKBOX_CHECK_SVG_HREF: &str = "/checkbox-check.svg";
/// Checkmark for the day theme, painted against the light-theme accent fill.
#[allow(dead_code)]
pub const CHECKBOX_CHECK_DAY_SVG_HREF: &str = "/checkbox-check-day.svg";

/// Night palette. Hex tokens match `night.css`.
pub const BG_0: &str = "#11111b";
pub const BG_1: &str = "#181825";
pub const BG_2: &str = "#1e1e2e";
pub const BG_3: &str = "#313244";
pub const BORDER: &str = "#45475a";
pub const TEXT: &str = "#cdd6f4";
pub const TEXT_MUTED: &str = "#6c7086";
/// Mermaid arrow fallback when a marker URL has no hex suffix (`--subtext0`).
pub const TEXT_SUBTLE: &str = "#a6adc8";
pub const ACCENT: &str = "#cba6f7";
pub const CODE: &str = "#f38ba8";

/// Day palette. Hex tokens match `day.css`. Dark text on a light page.
pub const DAY_BG_0: &str = "#dce0e8";
pub const DAY_BG_1: &str = "#e6e9ef";
pub const DAY_BG_2: &str = "#eff1f5";
pub const DAY_BG_3: &str = "#ccd0da";
pub const DAY_BORDER: &str = "#bcc0cc";
pub const DAY_TEXT: &str = "#4c4f69";
pub const DAY_TEXT_MUTED: &str = "#9ca0b0";
pub const DAY_TEXT_SUBTLE: &str = "#6c6f85";
pub const DAY_ACCENT: &str = "#5b21b6";
pub const DAY_CODE: &str = "#d20f39";

/// [`TEXT`] as PDF DeviceRGB fill. Ironpress math letters inherit this and
/// never emit `rg`; fraction rules emit hardcoded `0 0 0 rg`.
pub const TEXT_FILL: &[u8] = &crate::rendering::pdf_rgb_operator(TEXT, true);
pub const TEXT_STROKE: &[u8] = &crate::rendering::pdf_rgb_operator(TEXT, false);
pub const DAY_TEXT_FILL: &[u8] = &crate::rendering::pdf_rgb_operator(DAY_TEXT, true);
pub const DAY_TEXT_STROKE: &[u8] = &crate::rendering::pdf_rgb_operator(DAY_TEXT, false);

pub const DEFAULT_FONT_SIZE: usize = 14;
/// Alphabetic baseline below the node center so Latin caps sit in the box.
pub const BASELINE_FROM_CENTER: f64 = 0.35;

/// rusty-mermaid measures labels as Intel One Mono (~0.6em). Ironpress maps
/// that family to Helvetica, so PDF uses Courier (standard 14 monospace).
pub const PDF_FONT_FAMILY: &str = "Courier, monospace";
/// Advance width of one [`PDF_FONT_FAMILY`] glyph as a fraction of font size.
pub const MONOSPACE_ADVANCE_RATIO: f64 = 0.6;
/// Matches rusty_mermaid_core::constants::BASELINE_ASCENT_RATIO.
pub const BASELINE_ASCENT_RATIO: f64 = 0.3;
pub const ARROW_SIZE: f64 = 8.0;
/// Font family of flattened graphviz node labels.
pub const GRAPHVIZ_LABEL_FONT_FAMILY: &str = "Helvetica, Arial, sans-serif";
/// Gap between a graphviz edge and its upright label, as a fraction of font size.
pub const GRAPHVIZ_EDGE_LABEL_GAP: f64 = 0.25;
/// Extra mermaid padding so a stroke centered on the viewBox edge is not clipped.
pub const MERMAID_STROKE_SLOP: f64 = 1.0;
/// Optional mermaid statement terminator rusty-mermaid rejects at line end.
pub const MERMAID_STATEMENT_TERMINATOR: char = ';';
/// Named entities decoded in mermaid labels, as `&name;` or mermaid's `#name;`.
pub const MERMAID_NAMED_ENTITIES: &[(&str, char)] = &[
    ("quot", '"'),
    ("amp", '&'),
    ("lt", '<'),
    ("gt", '>'),
    ("apos", '\''),
    ("nbsp", '\u{a0}'),
    ("lbrack", '['),
    ("rbrack", ']'),
    ("lpar", '('),
    ("rpar", ')'),
    ("lbrace", '{'),
    ("rbrace", '}'),
    ("semi", ';'),
    ("colon", ':'),
    ("num", '#'),
    ("vert", '|'),
    ("copy", '©'),
    ("reg", '®'),
    ("deg", '°'),
    ("hellip", '…'),
    ("ndash", '–'),
    ("mdash", '—'),
    ("larr", '←'),
    ("rarr", '→'),
    ("uarr", '↑'),
    ("darr", '↓'),
    ("hearts", '♥'),
];
/// Decoded entity characters that would change how rusty-mermaid parses or
/// measures a label, so they pass through layout as stand-ins.
pub const MERMAID_PROTECTED_CHARS: &[char] = &[
    '"', '<', '>', '&', '[', ']', '(', ')', '{', '}', '|', ';', '*',
];
/// First stand-in for [`MERMAID_PROTECTED_CHARS`]. U+20C2..=U+20CF are
/// unassigned, and rusty-mermaid measures them at the full monospace advance.
pub const MERMAID_STAND_IN_BASE: u32 = 0x20C2;
/// Line prefixes of mermaid statements whose `#…;` text is a colour, not an entity.
pub const MERMAID_STYLE_PREFIXES: &[&str] = &["style ", "classDef ", "linkStyle ", "%%"];
/// Delimiters of a mermaid markdown string label, `"`…`"`.
pub const MERMAID_MARKDOWN_STRING_OPEN: &str = "\"`";
pub const MERMAID_MARKDOWN_STRING_CLOSE: &str = "`\"";
/// Lowest WCAG contrast ratio a mermaid label may have against its fill
/// before it is repainted black or white (the WCAG large-text minimum).
pub const MERMAID_MIN_TEXT_CONTRAST: f64 = 3.0;
/// Segments a circular path arc is split into when testing what a label sits on.
pub const MERMAID_ARC_SAMPLES: usize = 16;
/// Width of a rusty-mermaid journey task box.
pub const MERMAID_JOURNEY_TASK_WIDTH: f64 = 120.0;
/// Height of a rusty-mermaid journey task box.
pub const MERMAID_JOURNEY_TASK_HEIGHT: f64 = 32.0;
/// Horizontal space rusty-mermaid keeps between a journey task label and its box edges.
pub const MERMAID_JOURNEY_LABEL_PADDING: f64 = 8.0;
/// Width of a rusty-mermaid architecture service box.
pub const MERMAID_ARCHITECTURE_SERVICE_WIDTH: f64 = 100.0;
/// Height of a rusty-mermaid architecture service box.
pub const MERMAID_ARCHITECTURE_SERVICE_HEIGHT: f64 = 80.0;
/// Scene offset of an architecture service's icon circle from its box.
pub const MERMAID_ARCHITECTURE_ICON_OFFSET: usize = 1;
/// Scene offset of an architecture service's label from its box (after the icon text).
pub const MERMAID_ARCHITECTURE_LABEL_OFFSET: usize = 3;
/// Space kept between a restored mermaid label and the edges of its box.
pub const MERMAID_LABEL_PADDING: f64 = 8.0;
/// Smallest font size a restored mermaid label is shrunk to while fitting its box.
pub const MERMAID_MIN_LABEL_FONT_SIZE: f64 = 8.0;
/// Step a restored mermaid label's font size shrinks by while fitting its box.
pub const MERMAID_LABEL_FONT_STEP: f64 = 1.0;
/// Stroke width rusty-mermaid gives treemap leaf boxes (sections use 1.5).
pub const MERMAID_TREEMAP_LEAF_STROKE: f64 = 1.0;
/// Share of a treemap leaf's height rusty-mermaid uses as its label font size.
pub const MERMAID_TREEMAP_FONT_RATIO: f64 = 0.25;
/// Size of a treemap leaf's value relative to its name.
pub const MERMAID_TREEMAP_VALUE_SCALE: f64 = 0.85;
/// Objects a mermaid `theme` key may sit in: `%%{init: ...}%%` or `%%{initialize: ...}%%`.
pub const MERMAID_INIT_KEYS: &[&str] = &["init", "initialize"];
/// Directive key that names a mermaid theme.
pub const MERMAID_THEME_KEY: &str = "theme";
/// `themeVariables` key holding the diagram font size, e.g. `'14px'`.
pub const MERMAID_FONT_SIZE_KEY: &str = "fontSize";
/// Unit suffix allowed on a mermaid `fontSize` value.
pub const MERMAID_FONT_SIZE_UNIT: &str = "px";
/// Mermaid theme names.
pub const MERMAID_THEME_CLASSIC: &str = "classic";
pub const MERMAID_THEME_FOREST: &str = "forest";
pub const MERMAID_THEME_DARK: &str = "dark";
pub const MERMAID_THEME_NEUTRAL: &str = "neutral";
/// Background of the light mermaid themes. Near-white rather than white
/// because the rusty-mermaid SVG renderer skips the background rect for
/// pure white, which would leave light diagrams on the night editor surface.
pub const MERMAID_LIGHT_THEME_BACKGROUND: &str = "#fefefe";
/// `themeVariables` of the lavender `classic` theme, applied over `Theme::light()`.
pub const MERMAID_CLASSIC_THEME_VARIABLES: &[(&str, &str)] = &[
    ("background", MERMAID_LIGHT_THEME_BACKGROUND),
    ("primaryColor", "#ececff"),
    ("primaryTextColor", "#333333"),
    ("primaryBorderColor", "#9370db"),
    ("lineColor", "#333333"),
    ("secondaryColor", "#ffffde"),
    ("tertiaryColor", "#ffffde"),
    ("tertiaryBorderColor", "#aaaa33"),
    ("tertiaryTextColor", "#333333"),
    ("textColor", "#333333"),
    ("edgeLabelBackground", "#e8e8e8"),
    ("noteBkgColor", "#fff5ad"),
    ("noteBorderColor", "#aaaa33"),
    ("noteTextColor", "#333333"),
    ("actorBorder", "#ccccff"),
    ("actorLineColor", "#808080"),
    ("signalColor", "#333333"),
    ("activationBkgColor", "#f4f4f4"),
    ("activationBorderColor", "#666666"),
];
/// `themeVariables` of mermaid's `forest` theme, applied over `Theme::light()`.
pub const MERMAID_FOREST_THEME_VARIABLES: &[(&str, &str)] = &[
    ("background", MERMAID_LIGHT_THEME_BACKGROUND),
    ("primaryColor", "#cde498"),
    ("primaryTextColor", "#000000"),
    ("primaryBorderColor", "#13540c"),
    ("lineColor", "#008000"),
    ("secondaryColor", "#cdffb2"),
    ("tertiaryColor", "#cdffb2"),
    ("tertiaryBorderColor", "#6eaa49"),
    ("tertiaryTextColor", "#333333"),
    ("textColor", "#000000"),
    ("edgeLabelBackground", "#e8e8e8"),
    ("noteBkgColor", "#fff5ad"),
    ("noteBorderColor", "#6eaa49"),
    ("noteTextColor", "#000000"),
    ("actorLineColor", "#808080"),
    ("signalColor", "#333333"),
    ("labelBoxBorderColor", "#326932"),
    ("activationBkgColor", "#dddddd"),
    ("activationBorderColor", "#13540c"),
    ("specialStateColor", "#13540c"),
    ("compositeBorder", "#13540c"),
];
/// `themeVariables` of mermaid's `neutral` theme, applied over `Theme::light()`.
pub const MERMAID_NEUTRAL_THEME_VARIABLES: &[(&str, &str)] = &[
    ("background", MERMAID_LIGHT_THEME_BACKGROUND),
    ("primaryColor", "#eeeeee"),
    ("primaryTextColor", "#111111"),
    ("primaryBorderColor", "#999999"),
    ("lineColor", "#666666"),
    ("secondaryColor", "#f4f4f4"),
    ("tertiaryColor", "#f4f4f4"),
    ("tertiaryBorderColor", "#707070"),
    ("tertiaryTextColor", "#333333"),
    ("textColor", "#333333"),
    ("edgeLabelBackground", "#ffffff"),
    ("noteBkgColor", "#666666"),
    ("noteBorderColor", "#999999"),
    ("noteTextColor", "#ffffff"),
    ("actorBorder", "#707070"),
    ("actorLineColor", "#666666"),
    ("signalColor", "#333333"),
    ("activationBkgColor", "#f4f4f4"),
    ("activationBorderColor", "#666666"),
    ("specialStateColor", "#222222"),
    ("compositeBorder", "#707070"),
];
/// `themeVariables` of mermaid's `dark` theme, applied over `Theme::dark()`.
pub const MERMAID_DARK_THEME_VARIABLES: &[(&str, &str)] = &[
    ("background", "#333333"),
    ("primaryColor", "#1f2020"),
    ("primaryTextColor", "#e0dfdf"),
    ("primaryBorderColor", "#81b1db"),
    ("lineColor", "#cccccc"),
    ("secondaryColor", "#474949"),
    ("tertiaryColor", "#474949"),
    ("tertiaryBorderColor", "#888888"),
    ("tertiaryTextColor", "#f9fffe"),
    ("textColor", "#cccccc"),
    ("edgeLabelBackground", "#585858"),
    ("noteBkgColor", "#fff5ad"),
    ("noteBorderColor", "#aaaa33"),
    ("noteTextColor", "#333333"),
    ("actorLineColor", "#cccccc"),
    ("signalColor", "#cccccc"),
    ("activationBkgColor", "#474949"),
    ("activationBorderColor", "#81b1db"),
    ("specialStateColor", "#cccccc"),
    ("compositeBorder", "#81b1db"),
];
/// Offset WCAG adds to both luminances in a contrast ratio.
pub const WCAG_CONTRAST_FLARE: f64 = 0.05;

/// Syntect theme applied to fenced code blocks in the night theme.
pub const CODE_THEME: &str = "base16-ocean.dark";
/// Syntect theme applied to fenced code blocks in the day theme.
pub const DAY_CODE_THEME: &str = "base16-ocean.light";
/// Item marker of an unordered list rewritten for PDF export.
pub const LIST_BULLET: &str = "•";

/// Attribute holding the markdown source byte offset a rendered element came from.
pub const SOURCE_OFFSET_ATTR: &str = "data-src";
/// Attribute naming how a tagged element's text maps to the source.
pub const SOURCE_KIND_ATTR: &str = "data-src-kind";
/// [`SOURCE_KIND_ATTR`] value: each text line is the tail of one source line.
pub const SOURCE_KIND_LINES: &str = "lines";
/// [`SOURCE_KIND_ATTR`] value: the element maps to its offset as a whole.
pub const SOURCE_KIND_ATOM: &str = "atom";
/// Selector matching every element tagged with [`SOURCE_OFFSET_ATTR`].
pub const SOURCE_MAPPED_SELECTOR: &str = "[data-src]";
/// Selector of the rendered markdown inside an editor block.
pub const BLOCK_RENDER_SELECTOR: &str = ".block-render";
/// Selector of a rendered markdown link.
pub const LINK_SELECTOR: &str = "a[href]";
/// Attribute holding a link's destination.
pub const LINK_HREF_ATTR: &str = "href";
/// Selector of every rendered heading carrying an anchor id.
pub const ANCHORED_HEADING_SELECTOR: &str = "h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]";
/// Document methods returning the caret under a viewport point, each with the
/// result properties holding the caret's node and its offset in that node.
pub const CARET_FROM_POINT_METHODS: &[(&str, &str, &str)] = &[
    ("caretPositionFromPoint", "offsetNode", "offset"),
    ("caretRangeFromPoint", "startContainer", "startOffset"),
];
/// Inline styles of the hidden copy of a textarea used to measure where a
/// caret sits. The copy also takes the textarea's class, so with these it
/// wraps text exactly as the textarea does without affecting layout.
pub const TEXTAREA_MIRROR_STYLE: &[(&str, &str)] = &[
    ("position", "absolute"),
    ("top", "0"),
    ("left", "0"),
    ("height", "auto"),
    ("visibility", "hidden"),
    ("pointer-events", "none"),
    ("white-space", "pre-wrap"),
    ("overflow-wrap", "break-word"),
];
/// Zero-width text that marks the caret position inside the textarea mirror.
pub const CARET_MARKER_TEXT: &str = "\u{200b}";

/// Classes of the fallback block a failing render emits. Styled by `index.html`.
pub const GRAPHVIZ_ERROR_CLASS: &str = "graphviz-error";
pub const MERMAID_ERROR_CLASS: &str = "mermaid-error";

/// Error shown in the graphviz fallback block when the graph has no nodes.
pub const GRAPHVIZ_EMPTY_GRAPH_ERROR: &str = "Graph has no nodes";

/// XML declaration layout-rs prefixes to its SVG output.
pub const SVG_XML_DECLARATION: &str =
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>";

/// Header of every PDF ironpress produces.
pub const PDF_HEADER: &[u8] = b"%PDF-1.4\n";
/// Black DeviceRGB operators ironpress hardcodes for math, replaced by
/// [`TEXT_FILL`] and [`TEXT_STROKE`].
pub const PDF_BLACK_FILL: &[u8] = b"0 0 0 rg";
pub const PDF_BLACK_STROKE: &[u8] = b"0 0 0 RG";

pub const MATH_FONTS: [&[u8]; 3] = [b"/Helvetica-Oblique ", b"/Symbol ", b"/Helvetica "];

/// Liberation Sans faces embedded in exported PDFs. Names are ironpress font
/// variant keys: the family, then `__bold`, `__italic` or `__bold_italic`.
/// The first entry's name is the family `export-pdf.html` sets on the body.
pub const PDF_EMBEDDED_FONTS: [(&str, &[u8]); 4] = [
    (
        "Liberation Sans",
        include_bytes!("assets/LiberationSans-Regular.ttf"),
    ),
    (
        "Liberation Sans__bold",
        include_bytes!("assets/LiberationSans-Bold.ttf"),
    ),
    (
        "Liberation Sans__italic",
        include_bytes!("assets/LiberationSans-Italic.ttf"),
    ),
    (
        "Liberation Sans__bold_italic",
        include_bytes!("assets/LiberationSans-BoldItalic.ttf"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assert the night and day stylesheets match the editor palette tokens.
    fn theme_stylesheets_match_palette() {
        for (css, tokens) in [
            (
                NIGHT_CSS,
                [
                    ("--bg-0", BG_0),
                    ("--bg-1", BG_1),
                    ("--bg-2", BG_2),
                    ("--bg-3", BG_3),
                    ("--border", BORDER),
                    ("--text", TEXT),
                    ("--text-muted", TEXT_MUTED),
                    ("--accent", ACCENT),
                    ("--code", CODE),
                ],
            ),
            (
                DAY_CSS,
                [
                    ("--bg-0", DAY_BG_0),
                    ("--bg-1", DAY_BG_1),
                    ("--bg-2", DAY_BG_2),
                    ("--bg-3", DAY_BG_3),
                    ("--border", DAY_BORDER),
                    ("--text", DAY_TEXT),
                    ("--text-muted", DAY_TEXT_MUTED),
                    ("--accent", DAY_ACCENT),
                    ("--code", DAY_CODE),
                ],
            ),
        ] {
            for (token, hex) in tokens {
                let decl = format!("{token}: {hex}");
                assert!(css.contains(&decl), "missing {decl} in {css}");
            }
        }
        assert!(
            EXPORT_PDF_TEMPLATE.contains("{{THEME_CSS}}"),
            "PDF export must inject the active theme stylesheet: {EXPORT_PDF_TEMPLATE}"
        );
        assert!(
            EXPORT_PDF_TEMPLATE.contains("{{PAGE_BG}}"),
            "PDF @page background must use the active page token: {EXPORT_PDF_TEMPLATE}"
        );
        assert!(
            EXPORT_TEMPLATE.contains("{{THEME_CSS}}"),
            "HTML export must inject the active theme stylesheet: {EXPORT_TEMPLATE}"
        );
        assert!(
            EXPORT_TEMPLATE.contains("{{MATH_CSS}}"),
            "HTML export must inject the MathML stylesheet: {EXPORT_TEMPLATE}"
        );
    }

    #[test]
    /// Assert day selection and mark highlights paint with the accent token.
    fn day_highlights_use_accent() {
        for selector in ["::selection", "mark"] {
            assert!(
                DAY_CSS.contains(selector),
                "day highlights must style {selector}: {DAY_CSS}"
            );
        }
        assert!(
            DAY_CSS.contains("background: var(--accent)"),
            "day highlights must fill with --accent: {DAY_CSS}"
        );
        assert_eq!(DAY_ACCENT, "#5b21b6");
    }

    #[test]
    /// Assert night file-tree folder rows use the page background, matching day.
    fn night_folder_rows_use_page_background() {
        assert!(
            NIGHT_CSS
                .contains("html[data-theme=\"dark\"] .sidebar-panel .file-tree-root .menu-list a,"),
            "night must style file-tree folder-name rows: {NIGHT_CSS}"
        );
        assert!(
            NIGHT_CSS.contains(&format!("background-color: {BG_2};")),
            "night folder-name rows must use the page background {BG_2}: {NIGHT_CSS}"
        );
    }

    #[test]
    /// Assert inline code color uses the CODE token alias, not a raw hex.
    fn export_pdf_stylesheet_uses_code_alias() {
        assert!(
            EXPORT_PDF_TEMPLATE.contains("color: var(--code)"),
            "inline code must use var(--code): {EXPORT_PDF_TEMPLATE}"
        );
        assert!(
            !EXPORT_PDF_TEMPLATE.contains(&format!("color: {CODE}")),
            "inline code must not hardcode CODE hex: {EXPORT_PDF_TEMPLATE}"
        );
    }

    #[test]
    /// Assert the checkbox checkmark is a file URL, not an embedded SVG string.
    fn index_css_loads_checkbox_check_from_file() {
        const CSS: &str = include_str!("../index.css");
        assert!(
            !CSS.contains("data:image/svg+xml"),
            "CSS must not embed SVG as a data URI: {CSS}"
        );
        let url = format!("url(\"{CHECKBOX_CHECK_SVG_HREF}\")");
        assert!(
            CSS.contains(&url),
            "checked checkbox must load {CHECKBOX_CHECK_SVG_HREF}"
        );
        let day_url = format!("url(\"{CHECKBOX_CHECK_DAY_SVG_HREF}\")");
        assert!(
            DAY_CSS.contains(&day_url),
            "day checked checkbox must load {CHECKBOX_CHECK_DAY_SVG_HREF}"
        );
        let night = include_str!("../static/checkbox-check.svg");
        let day = include_str!("../static/checkbox-check-day.svg");
        for svg in [night, day] {
            assert!(svg.contains("<svg"), "{svg}");
            assert!(svg.contains("</svg>"), "{svg}");
        }
    }
}
