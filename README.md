# Cadence

A local-first desktop app for tracking your 1:1 conversations and personal tasks.
Built with **Tauri 2** (Rust) + **Svelte 5** (Vite). All your data lives as plain
**Markdown files** in a folder you control — no database, no cloud, fully diffable
and editable in any text editor.

---

## Why this exists

- **People** — one running history per teammate, with cadence reminders so nobody
  falls through the cracks.
- **Tasks** — a personal kanban (To do / In progress / Done) where each task can be
  linked to the person it belongs to.
- **Today** — what needs your attention right now: tasks due today + people overdue
  for a check-in.

The visual design is "Direction A — Quiet": warm paper palette, Newsreader serif
headings, conversation history as a vertical timeline. See `DESIGN_REFERENCE/` for
the original HTML prototypes this UI was translated from.

---

## Run it

You need **Node 18+**, **Rust** (stable), and the Tauri prerequisites for your OS
(see https://tauri.app/start/prerequisites/).

```bash
cd cadence-app
npm install

# Desktop app (Rust + Svelte together):
npm run tauri dev

# Frontend only, in a browser, with sample data (no Rust needed):
npm run dev
```

## App updates

Cadence now supports **in-app updates with Tauri’s updater plugin**.

### 1. Generate a signing key once

```bash
npm run tauri signer generate -- -w .tauri/updater.key
```

This creates:

- `.tauri/updater.key` — keep this private
- `.tauri/updater.key.pub` — safe to share, used by the app to verify updates

### 2. Build a release with updater config embedded

Set these **before** you run the Tauri build:

```bash
export CADENCE_UPDATER_ENDPOINT="https://downloads.example.com/cadence/{{target}}/{{arch}}/{{current_version}}.json"
export CADENCE_UPDATER_PUBKEY="$(cat .tauri/updater.key.pub)"
export TAURI_SIGNING_PRIVATE_KEY="$(cat .tauri/updater.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
```

Then build:

```bash
npm run tauri:build
```

`src-tauri/tauri.conf.json` is configured with `"createUpdaterArtifacts": true`, so
Tauri will emit the signed updater bundle plus its `.sig` file during release builds.

### 3. Generate a static update manifest

After a signed release build, generate the JSON manifest the updater will fetch:

```bash
UPDATE_BASE_URL="https://downloads.example.com/cadence" \
UPDATE_TARGET="darwin" \
UPDATE_ARCH="aarch64" \
npm run tauri:update-manifest
```

That writes:

```text
release-manifests/<target>/<arch>/<version>.json
```

Host that JSON at the URL pattern you embedded in `CADENCE_UPDATER_ENDPOINT`, and host
the updater artifact itself at `UPDATE_BASE_URL/<artifact filename>`.

### 4. What users see

The sidebar now includes an **Updates** panel:

- `Check for updates` fetches the manifest and compares versions
- `Install update` downloads the signed bundle and restarts the app
- if the build was made without updater env vars, the panel explains what is missing

> **Icons:** the bundler expects icons in `src-tauri/icons/`. Generate them once with
> `npm run tauri icon path/to/logo.png`, or remove the `bundle.icon` list in
> `src-tauri/tauri.conf.json` while developing.

On first launch the app creates a vault at `~/Documents/CadenceVault/`. To start from
the included sample data, copy `sample-vault/` there, or point the app at any folder
via the `set_vault_path` / `pick_vault` commands (wire a folder picker into Settings).

---

## The Markdown data model

Everything is stored under the **vault** folder:

```
CadenceVault/
├── people/
│   ├── marcus-koh.md
│   ├── ana-reyes.md
│   └── …
└── tasks.md
```

### Person file — `people/<slug>.md`

YAML frontmatter holds the person's metadata; each `## ` section is one logged 1:1,
**newest first**. Lines starting with `- [ ]` / `- [x]` inside a section are action
items.

```markdown
---
name: Marcus Koh
role: Frontend Engineer
joined: 2024-03
cadence_weeks: 2          # how often you want to meet (weeks)
color: "#6b7d9c"          # avatar color
next_1on1: 2026-06-30     # optional scheduled date
---

## 2026-06-16 — Roadmap & career growth

Walked through Q3 roadmap ownership…

- [ ] Share migration RFC by Fri
- [x] Set up intro with platform team lead
```

- **Heading format:** `## YYYY-MM-DD — Title` (em dash, en dash, or hyphen all parse).
- **`last_met`** is derived from the most recent section's date.
- **Cadence status** is derived: `over` if days-since-last-met > `cadence_weeks × 7`,
  `due` if within 3 days of that, else `ok`. (See `compute_status` in `vault.rs` and
  the mirror in `stores.js`.)

### Tasks file — `tasks.md`

Three `## ` columns; each `- [ ]` item carries inline `@key:value` metadata.
`@person` uses underscores for spaces so it stays one token.

```markdown
# Tasks

## To do
- [ ] Share migration RFC @person:Marcus_Koh @due:Fri @priority:high

## In progress
- [ ] Rebalance on-call rotation @person:Marcus_Koh @due:Jun_20 @priority:med

## Done
- [x] Approve Sofia PTO request @person:Sofia_Petrov @due:Today @priority:low
```

`@priority` is `high | med | low`. A task links to a person by matching `@person`
to that person's `name`.

---

## Architecture

```
┌─────────────────────────── Svelte (src/) ───────────────────────────┐
│  App.svelte → Sidebar + {PeopleView|PersonDetail|TasksView|TodayView}│
│  lib/stores.js   screen/selection state + status & format helpers    │
│  lib/api.js      invoke() wrappers, with browser sample-data fallback │
└───────────────────────────────┬─────────────────────────────────────┘
                                 │  Tauri invoke()
┌────────────────────────────── Rust (src-tauri/) ─────────────────────┐
│  lib.rs     commands: list_people, get_person, add_conversation,     │
│             list_tasks, save_tasks, get/set/pick vault                │
│  vault.rs   markdown parse + render (frontmatter, sections, tasks)   │
│  models.rs  Person / Conversation / ActionItem / Task (serde)        │
└──────────────────────────────────────────────────────────────────────┘
                                 │  std::fs
                          CadenceVault/*.md
```

`src/lib/api.js` detects whether it is running inside Tauri. In a plain browser it
serves data from `src/lib/sample.js`, so the whole UI is browsable with `npm run dev`
while you iterate on the frontend.

### Tauri commands (in `src-tauri/src/lib.rs`)

| Command             | Args                    | Returns        |
| ------------------- | ----------------------- | -------------- |
| `list_people`       | —                       | `Person[]`     |
| `get_person`        | `slug`                  | `Person`       |
| `add_conversation`  | `slug`, `conv`          | `Person`       |
| `list_tasks`        | —                       | `Task[]`       |
| `save_tasks`        | `tasks`                 | —              |
| `get_vault_path`    | —                       | `string`       |
| `set_vault_path`    | `path`                  | —              |
| `pick_vault`        | —                       | `string\|null` |

---

## What's implemented vs. next

**Working:** reading the vault, rendering all four views, logging a new 1:1
(`add_conversation` writes back to the markdown file), toggling a task done
(`save_tasks` rewrites `tasks.md`).

**Good next steps:**
- Settings screen with a folder picker (call `pick_vault` → `set_vault_path` → refresh).
- Optional release-channel support (`stable` / `beta`) by swapping the updater endpoint.
- File-watcher (`notify` crate) so edits made in your text editor live-update the app.

---

## File map

```
cadence-app/
├── package.json, vite.config.js, svelte.config.js, index.html
├── scripts/
│   └── generate-update-manifest.mjs
├── src/
│   ├── main.js, app.css, App.svelte
│   └── lib/
│       ├── api.js          Tauri command wrappers + browser fallback
│       ├── stores.js       state + helpers (status, dates, initials)
│       ├── sample.js       sample data for browser dev
│       └── components/     Sidebar, PeopleView, PersonCard,
│                           PersonDetail, TasksView, TodayView, UpdaterPanel
├── src-tauri/
│   ├── Cargo.toml, build.rs, tauri.conf.json
│   ├── capabilities/default.json
│   └── src/                main.rs, lib.rs, vault.rs, models.rs
├── sample-vault/           ready-to-use example markdown vault
└── DESIGN_REFERENCE/       original HTML design prototypes
```
