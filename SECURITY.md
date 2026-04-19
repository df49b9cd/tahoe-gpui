# Security Policy

## Supported versions

`tahoe-gpui` is pre-1.0. Only the latest `main` receives security fixes.

| Version | Supported |
| --- | --- |
| `main` | ✅ |
| `< 0.1` | ❌ |

## Reporting a vulnerability

Please **do not** open public issues for security problems.

Use GitHub's private vulnerability reporting:

1. Open https://github.com/df49b9cd/tahoe-gpui/security/advisories/new
2. Describe the issue, affected component (`foundations`, `markdown`, `workflow`, …), and a minimal reproduction if possible.

You can expect an acknowledgement within **7 days** and a fix or mitigation
plan within **30 days** for valid reports. Anonymous reports are welcome.

## Scope

In scope:

- Memory-safety issues (unsoundness in `unsafe` blocks).
- Input-handling bugs in the streaming markdown / code / remend parsers that
  could cause panics or unbounded resource use on attacker-controlled input.
- Any vulnerability in the dependency tree that `tahoe-gpui` re-exposes.

Out of scope:

- Bugs in [GPUI](https://github.com/zed-industries/zed) itself — report upstream.
- Issues that require an attacker to already have code execution on the host.
