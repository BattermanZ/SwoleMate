# Desktop Shell Foundation — Design

**Date:** 2026-05-27
**Status:** Approved (design phase)
**Slice:** 1 of a multi-slice desktop initiative

## Context

SwoleMate is a mobile-first SvelteKit PWA. The shell today is a sticky `AppBar`,
an optional offline banner, a centered `max-width: 720px` `<main>`, and a fixed
`BottomNav` (Today / Plans / History / Progress / More). The visual language is
defined in `docs/design-system.html` (warm cream surfaces, clay accent, gold/sage,
an always-dark `--surface-deep` for chrome moments).

We are introducing a desktop experience. Desktop is the **sit-down surface** —
reviewing progress, planning templates, and admin/management. **Live logging stays
on mobile** (at the gym) and is explicitly not a desktop priority.

The full desktop rethink is too large for one spec. It is decomposed into:

1. **Desktop shell foundation** ← this spec
2. Dashboard home (new desktop landing)
3. Plans (master-detail template editor)
4. Progress (analysis screen)
5. History + Admin / Settings / Backups

Each subsequent slice gets its own spec → plan → build cycle and plugs into the
foundation defined here. This spec contains **no screen-specific work** — only the
scaffold and reusable primitives.

## Goals

- A persistent desktop shell (sidebar rail + content column) chosen at a single
  breakpoint, leaving the mobile shell untouched.
- A reusable master-detail layout primitive that future desktop screens consume.
- One source of truth for navigation items, shared by mobile and desktop navs.
- No regressions to the existing mobile experience.

## Non-Goals

- Any concrete desktop screen (Dashboard, Plans editor, etc.).
- A tablet-specific intermediate layout (two modes only: mobile / desktop).
- Changes to controllers, stores, data layer, or business logic.
- Desktop-optimized live logging.

## Key Decisions

| Decision | Choice |
|---|---|
| Ambition | Full desktop rethink (delivered incrementally) |
| Shell direction | Sidebar rail + master-detail panes (direction "C") |
| Architecture | Shared logic, split presentation. Desktop view components consume existing controllers/stores. |
| Breakpoint mode | Two modes only: mobile below 1024px, desktop at/above |
| Flash handling | Shell chrome toggled by CSS media query (no flash, SSR-correct); heavy per-screen content tree selected via JS `viewport` store after hydration |
| Sidebar surface | Always-dark (`--surface-deep`), same as the hero card; stays dark in both themes |

## Architecture

### Breakpoint strategy

The breakpoint is expressed in **two cooperating ways**:

1. **Shell chrome — CSS media query.** Both the desktop `SideNav`/content column
   and the mobile `AppBar`/`BottomNav` exist in the layout markup. A `@media
   (min-width: 1024px)` rule shows one and `display:none`s the other. Because this
   is pure CSS, the correct chrome renders on first paint with no JS and no
   hydration flash. This is cheap — chrome is a handful of elements.

2. **Per-screen content tree — JS `viewport` store.** A screen's mobile component
   vs. its desktop (master-detail) component is the expensive part; we render only
   one. Screens branch on `viewport.isDesktop`. The store is `undefined` until
   mount (SSR-safe), and screens treat `undefined`/`false` as "render mobile tree"
   so the safe default is the existing experience, upgrading to desktop on mount.

This keeps chrome flicker-free while never double-rendering heavy content.

### `lib/stores/viewport.ts`

- Exposes a readable store with `isDesktop: boolean | undefined`.
- Implemented with `window.matchMedia('(min-width: 1024px)')`; subscribes to
  changes, unsubscribes on teardown.
- SSR-safe: when `window` is undefined (server) or before mount, value is
  `undefined`. A small helper `isDesktopView(v)` returns `v === true` so callers
  default to mobile when unknown.
- Breakpoint constant (`1024`) is exported and reused by the CSS via a shared
  value where practical (documented in the component, since CSS media queries
  can't read JS — the number is duplicated intentionally with a comment linking
  the two).

### Navigation items — single source

The nav definition (`href`, `label`, `icon` snippet, using the existing
`NavItem` type) currently lives inline in `+layout.svelte`. Extract it so both
`BottomNav` and `SideNav` consume the identical list and active-state logic.

- The `isActive(href, current)` helper (currently private inside `BottomNav`)
  moves to a shared module (e.g. `lib/components/shell/nav.ts`) and is imported
  by both navs. Behavior preserved exactly: `/` matches only `/`; other items
  match exact path or `current.startsWith(\`${href}/\`)`.

Note: the mobile bottom nav has a "More" item; the desktop rail can surface the
same destinations. For the foundation, the desktop rail renders the same item
list. If "More" maps to a sub-sheet on mobile, desktop simply lists its
destination route — no new behavior introduced in this slice.

### `components/shell/SideNav.svelte`

- Desktop-only left rail, ~220px wide, full viewport height, `position: sticky`/
  fixed to the viewport, `--surface-deep` background, `--on-deep` text.
- Top: `Logo` + "SwoleMate" wordmark.
- Middle: nav items. Active item uses the clay pill treatment
  (`background: var(--clay)`, white text, clay glow) consistent with `BottomNav`'s
  active state. Inactive items use `--on-deep-soft`.
- Bottom (pinned via `margin-top: auto`): theme toggle and logout, reusing the
  exact toggle logic already in `AppBar` (extract the `toggleTheme` function to a
  shared util so `AppBar` and `SideNav` share it rather than duplicating).
- Props: `items: NavItem[]`, `current?: string`, `onLogout?: () => void`.

### `components/shell/DesktopTopBar.svelte`

- Contextual header inside the content column (not the rail).
- Props: `title` (string), optional `subtitle`, and an `actions` snippet the
  screen fills (e.g. "+ New plan").
- Styling per design system: Onest 800 ~20px title with an optional
  Instrument-Serif italic em subtitle; hairline bottom border; lightly frosted
  over the page background.
- The offline banner renders inside the desktop content column (top of the
  scroll area) so offline state is visible in desktop mode too.

### `components/ui/MasterDetail.svelte`

- Reusable two-pane layout primitive. **Layout only — owns no selection state.**
- Props (snippets): `list` (left pane), `detail` (right pane), `empty` (shown in
  the detail region when the screen passes no detail / nothing selected).
- Behavior:
  - Left pane fixed width (~320px), scrolls independently.
  - Right pane fills remaining width, scrolls independently.
  - When the screen renders nothing into `detail`, the `empty` snippet shows.
- Desktop-only helper. Mobile screens keep their existing single-column flow and
  do not use this component.

### `+layout.svelte` changes

- Import the `viewport` store and the shared nav items.
- Render both shells; CSS media query shows the correct one:
  - **Mobile (`< 1024px`):** existing `AppBar` + offline banner + `<main>` +
    `BottomNav`. Unchanged.
  - **Desktop (`>= 1024px`):** `SideNav` (fixed left) + content column containing
    `DesktopTopBar` slot region, offline banner, and `<main>` (no 720px clamp;
    desktop content uses the full column width with its own max-width as screens
    dictate later).
- The login route (`isLogin`) bypasses both shells, as today.

## Component / File Summary

| Path | Action |
|---|---|
| `lib/stores/viewport.ts` | new — `isDesktop` store + `isDesktopView` helper + breakpoint constant |
| `lib/components/shell/nav.ts` | new — shared nav items + `isActive` helper |
| `lib/components/shell/theme.ts` | new — extracted `toggleTheme` util |
| `lib/components/shell/SideNav.svelte` | new — desktop rail |
| `lib/components/shell/DesktopTopBar.svelte` | new — desktop content header |
| `lib/components/ui/MasterDetail.svelte` | new — two-pane primitive |
| `lib/components/ui/index.ts` | edit — export `MasterDetail` |
| `lib/components/ui/BottomNav.svelte` | edit — import shared `isActive` |
| `lib/components/shell/AppBar.svelte` | edit — import shared `toggleTheme` |
| `routes/+layout.svelte` | edit — dual shell + CSS breakpoint |

## Testing

Existing Vitest setup; no new infrastructure.

- **`viewport` store** — mock `matchMedia`; assert `isDesktop` flips at the 1024px
  boundary, and that the value is `undefined` (safe default → mobile) before
  mount / on server.
- **`isActive` (shared nav)** — `/` matches only `/`; `/workouts/123` activates
  History; non-matching routes inactive. Both navs inherit this.
- **`MasterDetail`** — render test: `list` and `detail` snippets land in the
  correct panes; `empty` snippet renders when no `detail` is provided.
- **Manual/visual** — verify live app at desktop and mobile widths and across the
  1024px transition: chrome swaps cleanly, no console errors, no double-rendered
  content, offline banner visible in both modes, theme toggle works from both
  navs.

## Risks & Mitigations

- **CSS/JS breakpoint duplication (1024px in two places).** Documented with a
  linking comment; acceptable since CSS media queries cannot read JS values.
- **`toggleTheme` / `isActive` extraction.** Pure refactors with tests pinning
  current behavior before the move to guard against regressions.
- **Desktop content width.** The foundation removes the 720px clamp on desktop
  but does not impose a new max-width globally; each screen slice sets its own.
  The foundation ships with a sensible default content padding so an
  un-migrated screen viewed at desktop width is readable, not full-bleed.
