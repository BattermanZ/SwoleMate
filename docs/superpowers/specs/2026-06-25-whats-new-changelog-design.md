# What's New — changelog on update

**Date:** 2026-06-25
**Status:** Approved design, ready for implementation plan
**Target version:** 3.2.0

## Goal

Show users a short, plain-language "What's New" panel the first time they open
the app after it updates to a new version. Keep it friendly for non-technical
people. Let them re-open it any time.

## Decisions (made during brainstorming)

- **Seen-state storage:** per-device, in `localStorage`. No backend changes.
- **Changelog source:** a curated, typed TypeScript file edited by hand at
  release time.
- **Presentation:** an automatic modal on first load after an update, reusing
  the existing global-modal pattern; also re-openable from the menu.
- **Versioning:** this release is `3.2.0` (new features → minor bump from the
  last shipped release `3.1.2`).

## Architecture

Five small, independently understandable units:

### 1. Version exposure — `src/lib/version.ts`
The semver version is not available to the running frontend today. Inject
`package.json`'s `version` at build time via a Vite `define` (`__APP_VERSION__`)
in `vite.config.ts`, and re-export it as `APP_VERSION` from `src/lib/version.ts`.
Declare `__APP_VERSION__` in `src/app.d.ts` for type-safety. Works in the static
SPA build; no runtime fetch.

### 2. Changelog content — `src/lib/changelog.ts`
A typed, newest-first array, hand-edited at release time:

```ts
export interface ChangelogEntry {
  version: string; // semver, e.g. "3.2.0"
  date: string;    // ISO date, e.g. "2026-06-25"
  title: string;   // short headline
  items: string[]; // plain-language bullet points
}

export const CHANGELOG: ChangelogEntry[] = [ /* newest first */ ];
```

Seeded with the `3.2.0` entry (see "Initial content" below).

### 3. Detection — pure, TDD-first core — `src/lib/whatsNew.ts`
The decision logic is a pure function with no DOM or storage dependencies, so it
is built test-first:

```ts
// Compare two semver strings numerically. Returns >0 if a>b, 0 if equal, <0 if a<b.
export function compareVersions(a: string, b: string): number;

// Given the last-seen version (or null) and the changelog, return the entries
// the user has not seen yet (those strictly newer than lastSeen), newest first.
export function entriesToShow(
  lastSeen: string | null,
  changelog: ChangelogEntry[]
): ChangelogEntry[];
```

Behavioural rules (each is a test case):
- `lastSeen === null` (first-ever visit) → returns `[]`. New users are not shown
  historical notes; the caller seeds `lastSeen` to the current version.
- `lastSeen` older than the newest entry → returns every entry strictly newer
  than `lastSeen` (skipping multiple versions stacks them, newest first).
- `lastSeen` equal to the newest entry → returns `[]`.
- A release whose version has no changelog entry → returns `[]` (no nagging on
  notes-less patch releases).
- Malformed/short semver parts are treated as `0` (defensive; tested).

### 4. Trigger + store + modal
- `src/lib/stores/whatsNew.ts` — a writable store holding the entries to display
  (`ChangelogEntry[]`) or `null` when closed. Exposes:
  - `openWhatsNew(entries: ChangelogEntry[])` — sets the store.
  - `closeWhatsNew()` — clears the store.
- `src/lib/components/ui/WhatsNewHost.svelte` — mounted once in
  `routes/+layout.svelte`, beside `ConfirmHost`. Renders a modal when the store
  is non-null: a fixed `What's New` modal title, then each entry rendered as a
  block with its own `v{version} · {date}` sub-label, `title`, and bulleted
  `items` (so stacked, multi-version updates read clearly newest-first), and a
  single `Got it` button. Mirrors `ConfirmHost`'s
  backdrop click-to-close, `Escape` to close, `role="dialog"`/`aria-modal`, and
  respects `prefers-reduced-motion`.
- **Auto-trigger** in `routes/+layout.svelte`, run once on mount and gated on
  `$authState.status === 'authenticated'` (so it never covers the login screen):
  1. Read `localStorage['swolemate.lastSeenVersion']`.
  2. `const unseen = entriesToShow(lastSeen, CHANGELOG)`.
  3. If `lastSeen === null` → seed it to `APP_VERSION`, show nothing.
  4. Else if `unseen.length > 0` → `openWhatsNew(unseen)`.
  5. On modal close → write `APP_VERSION` to `localStorage`.

### 5. Re-open entry point
- A `What's New` row on the **Settings** page (`routes/settings/+page.svelte`)
  and the mobile **More** page (`routes/more/+page.svelte`) that calls
  `openWhatsNew(CHANGELOG)` to show the full history on demand.
- Display the current version as a small `v{APP_VERSION}` label alongside it.
- Re-opening does **not** change `lastSeenVersion` (it is not a "first sight").

## Data flow

```
build: package.json.version --(vite define)--> __APP_VERSION__ --> APP_VERSION
load (authenticated): localStorage.lastSeenVersion + CHANGELOG
        --> entriesToShow() --> [unseen] --> openWhatsNew() --> WhatsNewHost modal
close: APP_VERSION --> localStorage.lastSeenVersion
menu "What's New": openWhatsNew(CHANGELOG)  (no storage write)
```

## Error / edge handling
- `localStorage` unavailable/throwing (private mode): wrap reads/writes in
  try/catch; on failure, behave as "already seen" (show nothing) rather than
  crashing the app.
- Empty `CHANGELOG`: trigger shows nothing.
- First run seeds silently; users upgrading from a pre-feature build have no
  `lastSeenVersion`, so they are seeded to `APP_VERSION` and not shown old
  notes — the feature starts cleanly from this release forward.

## Testing
TDD the pure core in `src/lib/whatsNew.ts` via vitest (`client/src/test/whats-new.test.ts`):
- `compareVersions`: greater / equal / less, differing part counts, non-numeric
  parts.
- `entriesToShow`: first-run (null), single update, multiple skipped versions,
  equal-to-newest, version with no entry, empty changelog.

Glue (store, modal, trigger) stays thin; a light component smoke test for
`WhatsNewHost` (renders entries, `Got it` closes) is optional and lower priority.

## Initial content (3.2.0)

```ts
{
  version: '3.2.0',
  date: '2026-06-25',
  title: 'Your training calendar, now on mobile',
  items: [
    'See your whole year on your phone — the training calendar is now in the Progress tab on mobile. Swipe across it to look back through earlier months.',
    'The rest-timer chime no longer interrupts your music or podcasts.'
  ]
}
```

Derived from commits since `v3.1.2`: mobile heatmap + scroll affordance + card
header fix (one feature line); timer-chime fixes (one fix line). The card-header
fix is internal polish on an unreleased version, so it is not a separate user
line. The "What's New" feature itself is not listed — it delivers the notes
rather than being news.

## Out of scope (YAGNI)
- Per-account seen-state / cross-device sync (chose per-device).
- Markdown or git-derived changelog generation (chose curated TS).
- A non-modal banner surface (chose auto modal).
- Backend version endpoint.
