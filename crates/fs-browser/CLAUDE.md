# CLAUDE.md – fs-browser

## What is this?

FreeSynergy Browser — web browser with WebEngine abstraction, bookmarks, and history.
Runs inside the FreeSynergy Desktop shell or standalone.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Quality Gates (before every commit)

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo test
```

## Architecture

MVC pattern — all components stay renderer-agnostic:

- `BrowserModel`      — observable state (URL, title, loading, history, bookmarks)
- `BrowserController` — navigation + bookmark logic (knows only `WebEngine` + `BookmarkStore` traits)
- `BrowserView`       — `FsView` impl in `view.rs` (the ONLY file that imports `fs-render`)

Supporting:
- `NavigationHistory` — composite back/forward stack (from `fs-web-engine`)
- `BookmarkStore`     — repository trait, `InMemoryBookmarkStore` for testing
- `keys.rs`           — FTL key constants (all user-visible strings translated via `fs-i18n`)
- `cli.rs`            — clap CLI: `open`, `history`, `bookmarks list|add|remove`
- `grpc.rs`           — tonic gRPC server: Open/Navigate/History/Bookmarks/Health
- `rest.rs`           — axum REST + OpenAPI via utoipa

## FTL

Keys are in `fs-i18n/locales/{lang}/browser.ftl`.
