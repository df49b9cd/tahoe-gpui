## Summary

<!-- What does this PR change, and why? 1–3 bullet points. -->

## Scope

<!-- Scope label from CLAUDE.md: foundations / components / markdown / code / theme / materials / voice / workflow / remend -->

## Breaking changes

<!-- Call out any public-API break explicitly. Example:
     BREAKING: TextField::with_prompt → TextField::placeholder -->

None.

## Test plan

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo nextest run --workspace --all-features`
- [ ] `cargo doc --workspace --no-deps --all-features`
- [ ] Relevant gallery example still runs: `cargo run -p tahoe-gpui --example <name>`

## HIG / issue reference

<!-- Link to the HIG section or GitHub issue that motivates the change. -->
