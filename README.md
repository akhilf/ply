<!-- ai-generated -->
# Ply

A personal desktop web browser. **Ply** means a layer — chrome (tabs, address bar) is one ply, the page engine is the other.

Owner: [akhilf](https://github.com/akhilf)

## What it does (Stage 1)

- Tabs, address bar, back / forward / reload
- Bookmarks and history (SQLite in the app data directory)
- Isolated chrome: untrusted pages never share JavaScript with the toolbar
- Search: non-URL input goes to DuckDuckGo
- macOS, Windows, and Linux (Tauri 2 + OS webview)

Stage 2 (not in this tree yet) will add a small HTML/CSS engine in Rust. Same chrome, different page ply.

## Run

Needs Node 20+, Rust (rustup), and the Tauri CLI (from `npm`).

```bash
npm install
npm run tauri dev
```

Tests:

```bash
npm test
cd src-tauri && cargo test --lib
```

## Layout

| Path | Role |
| --- | --- |
| `src/` | TypeScript chrome UI |
| `src-tauri/` | Rust session, tabs, store, native webviews |
| `index.html` | Chrome shell only — pages load in separate webviews |

## Name

Ply is a working product name, not an acronym. Rename is a label change only.
<!-- ai-generated end -->
