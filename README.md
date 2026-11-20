# FlashBee

A calm, multi-repository Git viewer for reviewing changes across your projects.

## Run locally

Requires Node.js 20+ and Git.

```bash
npm install
npm run dev
```

Open [http://127.0.0.1:5173](http://127.0.0.1:5173).

The local Git API runs inside Vite. On macOS, **Add repository → Choose folder**
opens a native folder picker. A local path can be entered manually on any
platform. Repositories are read directly from disk and never uploaded.

## What's inside

- Multi-repository sidebar with branches, ahead/behind state, and staged/unstaged file groups
- Create and delete workspaces, plus add and remove repositories with local persistence
- Add repositories from a local folder, with Git branches read automatically and the current branch selected by default
- Real working-tree changes and unified diffs are loaded for local repositories
- Per-file additions/deletions, status badges, and quick filtering
- Side-by-side and unified diff modes with syntax highlighting for TSX, TypeScript, CSS, JSON, Markdown, Python, Go, Rust, WGSL, and more
- English and Simplified Chinese UI, with auto system-language detection
- Daylight, Midnight, and follow-system themes with a smooth crossfade
- 16px default interface and code sizes; adjustable interface/code fonts and sizes
- Settings panel persisted locally, including motion-friendly button states
- Command palette (`⌘ K`), file navigation (`J` / `K`), view switch (`S`), sidebar toggle (`⌘ \`), and settings (`⌘ ,`)
- Clean working tree state and responsive layout

The four seeded repositories use realistic demo data. Repositories added from
your machine are read live through a small local Git API served by Vite:
`server/repositoryApi.ts` reads branches and working-tree diffs, while
`src/lib/repositoryApi.ts` is the browser client. The native folder picker is
available on macOS; a local path can be entered as a fallback on other systems
or when the picker is unavailable.

Theme, language, and typography preferences live in `src/lib/settings.ts`.
The `System` theme follows `prefers-color-scheme` live and uses the browser View
Transitions API for a full-surface crossfade, with a color-transition fallback.
Interface strings live in `src/lib/i18n.ts`. Workspace and repository state is
managed in `src/lib/workspaces.ts` and persisted to `localStorage`. Button motion
uses shared spring/easing tokens and respects `prefers-reduced-motion`.

The app can inspect its own checkout: run it locally, then add this repository as
a local repository to review FlashBee's own uncommitted changes.
