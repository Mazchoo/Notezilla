# Vendored crates

## math-core 0.9.0 and math-core-renderer-internal 0.9.0

Source: <https://github.com/tmke8/math-core>, commit
`fc95cb704a8159467b69f1ebc93460de7c195ce1`, MIT licensed. `Cargo.toml` patches
crates.io to these copies. Only `src/`, the manifest, the README and the
license are kept; upstream tests and examples are not.

Every change is marked with a `Notezilla:` comment in the source.

| Addition | Where |
| :--- | :--- |
| `alignedat`, `alignat`, `alignat*` environments: `aligned` / `align` with no space between column pairs | `environments.rs`, `parser.rs`, renderer `table.rs` (`Alignment::AlternatingTight`) |
| `\cancelto{value}{expr}`: `<msup>` over `<menclose notation="updiagonalarrow">` | `commands.rs`, `token.rs`, `parser.rs`, renderer `attribute.rs`, `ast.rs` |
| `#rrggbb` color literals: `\color{#2563eb}` | `lexer.rs` (`Token::HexColor`), `parser.rs` |
| Math accents inside `\text`: `\text{Amp\grave{e}re}` is typeset like `` \`{e} `` | `text_parser.rs` |
| `class="math-core-wide-tilde"` / `math-core-wide-check` on `\widetilde` / `\widecheck` | renderer `ast.rs` |
| `#![allow(deprecated)]` for upstream's use of `kstring::KString` | both `lib.rs` |

The markup these produce is styled by `../math.css`, which also carries
upstream's required `css/mathmlfixes.css`.

To move to a newer math-core, copy the new sources over these directories and
reapply the `Notezilla:` changes; the tests in
`src/rendering/renders/math.rs` and `latex_manual_has_no_math_errors` in
`src/rendering/math_substitution.rs` cover each one.
