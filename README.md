# FlashBee

A calm, native macOS Git viewer for reviewing changes across multiple local repositories.

FlashBee reads repositories directly from disk, never uploads them, and keeps the whole
review flow — workspaces, repository tree, branches, and diffs — in one desktop window.

This branch turns the original web app into a **Tauri 2 desktop app**. The browser version
is preserved in [`demo/`](demo/README.md) as a runnable reference.

## Highlights

- Native macOS window with overlay title bar, integrated traffic lights, system menu bar,
  and window state restored in logical points across Retina displays
- Local Git engine written in Rust: branches, working tree status, unified diffs,
  staged/unstaged groups, untracked files, and rename detection
- Multi-repository workspaces, created and persisted entirely on this Mac
- Linked `git worktree` checkouts appear nested under their repository with their
  own branch, ahead/behind state, and changes; review, diff, push, and merge them
  without adding each checkout by hand
- Add a parent folder to discover every Git repository inside it and add them in
  one step; they stay together in a collapsible folder group you can rescan for
  new repositories. Plain folders can optionally be initialized with `git init`
- Native folder picker, reveal in Finder, open in editor/default app, and native right-click menus
- Side-by-side and unified diff views with syntax highlighting for TypeScript, TSX, Rust, CSS,
  JSON, Markdown, Python, Go, WGSL, and more
- Hover any diff line to see its author, relative commit time, hash, and commit summary
- Branch comparison is read-only: pick a local or remote-tracking branch to diff the
  working tree against it
- Pull from the repository toolbar; pushing opens a commit review dialog with per-commit
  diffs, selective commits, tag pushes, and `--force-with-lease`, with upstream tracking
  and ahead/behind counts; configure or update remote URLs without leaving the app
- Merge conflict resolver with side-by-side and unified views: resolve each conflict
  left or right, take an entire side, or quick-merge every conflict where only one side
  changed. Resolved files are staged, and the merge can be completed or aborted
- Auto-refresh when the window regains focus
- English and Simplified Chinese UI, with a matching localized application menu
- Daylight, Midnight, and follow-system themes; system UI/code fonts by default with
  adjustable font choices and sizes
- Desktop interaction details: hidden scrollbars, compact single-line file rows,
  native text selection, no browser context menus, and keyboard-first navigation

## Requirements

- macOS 11 or newer
- Node.js 20+
- Rust stable (`rustup`) with the macOS target
- Xcode Command Line Tools
- Git installed and available on `PATH`

## Run the app

```bash
npm install
npm run app
```

`npm run app` starts Vite and launches the Tauri shell with HMR.

### Build a `.app`

```bash
npm run app:build
```

The bundle is written to `src-tauri/target/release/bundle/macos/FlashBee.app`
(and a `.dmg` next to it when the default bundle targets are used).

For a faster local check without release optimizations:

```bash
npm run tauri build -- --debug --bundles app
```

## Run the archived web demo

```bash
cd demo
npm install
npm run dev
```

Open [http://127.0.0.1:5173](http://127.0.0.1:5173). The demo keeps the original
Vite middleware Git API and browser-first styling; it is intentionally independent
of the native app.

## Keyboard shortcuts

| Action | Shortcut |
| --- | --- |
| Open repository | `⌘ O` |
| New workspace | `⇧ ⌘ N` |
| Command palette | `⌘ K` |
| Toggle sidebar | `⌘ \` |
| Split / unified diff | `⌘ 1` / `⌘ 2` |
| Previous / next file | `⌘ [` / `⌘ ]` |
| Refresh | `⌘ R` |
| Settings | `⌘ ,` |
| Next / previous file | `J` / `K` |
| Toggle diff layout | `S` |
| Close overlay | `Esc` |

## Project layout

```text
.
├── src/                    React frontend for the desktop app
│   ├── components/         Sidebar, diff viewer, palette, settings, dialogs
│   ├── lib/                Native bridge, Git client, i18n, settings, workspaces
│   └── data/projects.ts    Optional sample workspace data
├── src-tauri/
│   ├── src/git.rs          Git inspection and diff parsing
│   ├── src/menu.rs         Localized macOS application menu
│   ├── src/storage.rs      JSON persistence in the app data directory
│   ├── src/window_state.rs Logical-point window state restore
│   ├── src/lib.rs          Tauri commands and app setup
│   └── tauri.conf.json     Window, bundle, CSP, and plugin configuration
├── demo/                   Archived browser version (Vite + web Git API)
└── public/                 Favicon
```

## How data is stored

- Workspaces and preferences are mirrored to JSON files under
  `~/Library/Application Support/com.flashbee.app/`.
- Window geometry is stored in `window-state.json` in the same directory and restored
  only when it still intersects a connected display.
- Local repository diffs and transient merge state are **not** persisted; they are
  re-read from disk on launch. Only repository paths, names, folder groupings, and the
  selected comparison branch are kept. Remote URLs live in each repository's Git config.
- Repositories are never uploaded except for the explicit pull and push commands you
  trigger. All other Git work runs locally.

## Notes

- The app compares the working tree against the selected branch (or `HEAD` when no
  branch is selected). Selecting a branch never checks it out.
- Pull and push are the only network operations. Pull uses a merge and stops on
  conflicts; the resolver then writes the chosen result for each conflicted file and
  stages it with `git add`. Completing the merge creates the merge commit. Aborting
  returns the work tree to its pre-merge state.
- The optional `git init` when adding a plain folder is the only other repository
  mutation FlashBee performs.
- The sample workspace is optional and can be opened from the empty state; remove it like
  any other workspace when finished.
- If a repository is moved or deleted on disk, FlashBee reports the load failure and leaves
  the rest of the workspace untouched.

## License

MIT © 2026 wangzhuo. See [LICENSE](LICENSE).
