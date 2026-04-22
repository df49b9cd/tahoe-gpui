# mdstitch

Streaming markdown preprocessor — auto-completes incomplete syntax during
token-by-token streaming.

## What it does

Partial markdown renders badly mid-stream. `**bold` shows two literal
asterisks, `[link` confuses inline parsers, an unterminated ` ``` ` fence
swallows every subsequent token as code. `mdstitch` runs *before*
[`pulldown-cmark`][pd] on each accumulated chunk and closes unterminated
markers so every intermediate frame is well-formed CommonMark:

- `**bold`   →  `**bold**`
- `` `code ``  →  `` `code` ``
- `[text](http`  →  `[text](stitch:incomplete-link)`
- `$$\frac{a}{b}`  →  `$$\frac{a}{b}$$`

When no changes are needed, `stitch` returns `Cow::Borrowed` — the zero-
allocation fast path — so streaming renderers can invoke it on every delta
without incurring a copy when the text is already closed.

[pd]: https://github.com/pulldown-cmark/pulldown-cmark

## Status

Workspace-private (`publish = false`). Consumed by
[`tahoe-gpui`](../tahoe-gpui) for streaming Markdown rendering.

## Usage

```toml
[dependencies]
mdstitch = { path = "../mdstitch" }
```

```rust
use mdstitch::{stitch, StitchOptions};

let partial = "Hello **wor";
let completed = stitch(partial, &StitchOptions::default());
assert_eq!(completed.as_ref(), "Hello **wor**");
```

Inside `tahoe-gpui`, the incremental parser opts in with
`with_stitch` (see `crates/tahoe-gpui/src/markdown/parser/mod.rs:69`):

```rust
use mdstitch::StitchOptions;
use tahoe_gpui::markdown::IncrementalMarkdownParser;

let mut parser = IncrementalMarkdownParser::with_stitch(StitchOptions::default());
parser.push_delta("# Hello **wor");
let blocks = parser.parse(); // parses "# Hello **wor**"
```

## Built-in handlers

Handlers run in priority order (lower first). Every option defaults to `true`
except `inline_katex`, which is off because a lone `$` is ambiguous with
currency.

| Option                 | Priority | Completes / handles                                                              | Default |
| ---------------------- | -------- | -------------------------------------------------------------------------------- | ------- |
| `single_tilde`         | 0        | Escapes a lone `~` between word characters                                       | on      |
| `comparison_operators` | 5        | Escapes `>` at the start of list items so it doesn't parse as a blockquote       | on      |
| `html_tags`            | 10       | Strips an incomplete trailing HTML tag                                           | on      |
| `setext_headings`      | 15       | Prevents a trailing `===` / `---` line from being misread as a setext underline  | on      |
| `links` / `images`     | 20       | `[text](url` → `[text](stitch:incomplete-link)` (see `LinkMode`)                 | on      |
| `bold_italic`          | 30       | `***x` → `***x***`                                                               | on      |
| `bold`                 | 35       | `**x` → `**x**`                                                                  | on      |
| `italic`               | 40–42    | `__x` / `*x` / `_x` → closed                                                     | on      |
| `inline_code`          | 50       | `` `x `` → `` `x` ``                                                             | on      |
| `strikethrough`        | 60       | `~~x` → `~~x~~`                                                                  | on      |
| `katex`                | 70       | `$$eq` → `$$eq$$`                                                                | on      |
| `inline_katex`         | 75       | `$eq` → `$eq$`                                                                   | **off** |

Priorities are re-exported as constants in [`mdstitch::priority`](src/options.rs)
so custom handlers can slot between the built-ins.

### `LinkMode`

Controls what happens when an incomplete `[text](url…` is detected:

- `LinkMode::Protocol` (default) — rewrite to
  `[text](stitch:incomplete-link)`. Lets the downstream renderer keep the
  link text visible; the sentinel URL can be detected to style it as pending.
- `LinkMode::TextOnly` — drop the link markup entirely and render only the
  text.

## Custom handlers

Implement `StitchHandler` and register with `StitchOptions::handler`:

```rust
use std::borrow::Cow;
use mdstitch::{priority, stitch, StitchHandler, StitchOptions};

struct UpperCaseShouts;

impl StitchHandler for UpperCaseShouts {
    fn handle<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if text.contains("SHOUT") {
            Cow::Owned(text.replace("SHOUT", "shout"))
        } else {
            Cow::Borrowed(text)
        }
    }

    fn name(&self) -> &str { "uppercase-shouts" }

    fn priority(&self) -> i32 { priority::DEFAULT } // = 100, runs after built-ins
}

let opts = StitchOptions::default().handler(Box::new(UpperCaseShouts));
let _ = stitch("SHOUTing into the void", &opts);
```

Handler authors can reuse `mdstitch`'s own scanning helpers so they honour the
same code-block and link boundaries as the built-ins:

- `is_inside_code_block(text, pos)`
- `is_within_link_or_image_url(text, pos)`
- `is_within_math_block(text, pos)`
- `is_word_char(ch)`

For repeated queries on the same input, share a [`CodeBlockRanges`](src/ranges.rs)
instead — it scans once in O(n) and answers subsequent checks in O(log n).

## Secondary utilities

`mdstitch` also exposes helpers that tahoe-gpui uses outside the auto-completion
pipeline:

- `has_incomplete_code_fence(&str) -> bool` — walks lines per CommonMark §4.5
  to detect an unclosed fence. Used by `IncrementalMarkdownParser` to gate
  code-block styling mid-stream.
- `has_table(&str) -> bool` — detects a GFM table delimiter row (`| --- |`).
- `detect_text_direction(&str) -> TextDirection` — first-strong-character
  Unicode heuristic, returns `Ltr` or `Rtl`. Skips common markdown syntax
  (headings, emphasis, inline code, links) before sampling.
- `preprocess_custom_tags(markdown, &[tag])` — replaces `\n\n` inside a named
  HTML tag with an `<!---->` spacer so blank lines don't split the CommonMark
  block.
- `preprocess_literal_tag_content(markdown, &[tag])` — escapes markdown
  metacharacters inside chosen tags so their body renders as literal text.
- `normalize_html_indentation(&str) -> Cow<'_, str>` — dedents leading
  whitespace that would otherwise make a tag look like an indented code block.

## Module layout

```
src/
├── lib.rs                    # entry point, pipeline orchestration, re-exports
├── options.rs                # StitchOptions, StitchHandler, LinkMode, priority::*
├── ranges.rs                 # CodeBlockRanges — shared range index
├── fence.rs                  # CommonMark §4.5 fence/inline-code scanner
├── bracket.rs                # balanced [ / ] matcher (respects code spans)
├── utils.rs                  # shared predicates (is_word_char, is_escaped, …)
│
│   # One handler per marker class:
├── emphasis.rs               # ** *** __  and the three italic variants
├── inline_code.rs            # `…`
├── strikethrough.rs          # ~~…~~
├── single_tilde.rs           # lone ~ escaping
├── link_image.rs             # [text](url, ![alt](url
├── katex.rs                  # $$…$$ and $…$
├── html_tags.rs              # incomplete trailing tag stripping
├── setext_heading.rs         # dangling === / --- underlines
├── comparison_operators.rs   # > at list-item start
│
│   # Secondary (not part of the stitch() pipeline):
├── detect_direction.rs       # RTL/LTR detection
├── incomplete_code.rs        # has_incomplete_code_fence, has_table
├── preprocess.rs             # custom / literal HTML tag handling
│
└── tests.rs                  # unit tests + proptest fuzzers
```

## Testing

```bash
cargo nextest run -p mdstitch
```

Use `cargo nextest`, not `cargo test` — the workspace `.config/nextest.toml`
tunes parallelism and retries.

The test suite exercises every built-in handler in isolation plus a
`proptest!` block that fuzzes:

- Arbitrary UTF-8 never panics.
- Every streaming prefix of arbitrary UTF-8 never panics (each cut on a char
  boundary is `stitch`'d).
- Idempotency across every option combination: `stitch(stitch(x)) == stitch(x)`.
- Custom-handler order matches the priority sort.

Proptest regressions are committed under `proptest-regressions/tests.txt`.

## License

Apache-2.0. See [LICENSE](../../LICENSE) at the workspace root.
