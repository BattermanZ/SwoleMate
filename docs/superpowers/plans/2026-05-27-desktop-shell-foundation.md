# Desktop Shell Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a desktop shell (always-dark sidebar rail + content column) chosen at a 1024px breakpoint, plus a reusable MasterDetail layout primitive, without touching the mobile experience or any business logic.

**Architecture:** Shared logic, split presentation. A `viewport` store drives per-screen content-tree selection after hydration; a CSS media query toggles the shell chrome (no flash, SSR-correct). Two refactors (`isActive`, `toggleTheme`) extract shared logic so mobile and desktop navs/headers stay DRY. New desktop components (`SideNav`, `DesktopTopBar`, `MasterDetail`) consume existing controllers/stores untouched.

**Tech Stack:** SvelteKit, Svelte 5 (runes), TypeScript, Vitest + jsdom + @testing-library/svelte v5. Design tokens from `docs/design-system.html`.

---

## File Structure

| Path | Responsibility |
|---|---|
| `client/src/lib/components/shell/nav.ts` | NEW — `isActive(href, current)` shared active-route logic |
| `client/src/lib/stores/viewport.ts` | NEW — `isDesktop` readable store + `isDesktopView` helper + `DESKTOP_MIN_WIDTH` |
| `client/src/lib/components/shell/theme.ts` | NEW — `toggleTheme()` util shared by AppBar + SideNav |
| `client/src/lib/components/shell/SideNav.svelte` | NEW — desktop left rail |
| `client/src/lib/components/shell/DesktopTopBar.svelte` | NEW — desktop content header w/ actions snippet |
| `client/src/lib/components/ui/MasterDetail.svelte` | NEW — two-pane layout primitive |
| `client/src/lib/components/ui/index.ts` | MODIFY — export MasterDetail |
| `client/src/lib/components/ui/BottomNav.svelte` | MODIFY — use shared `isActive` |
| `client/src/lib/components/shell/AppBar.svelte` | MODIFY — use shared `toggleTheme` |
| `client/src/routes/+layout.svelte` | MODIFY — dual shell + CSS breakpoint |
| `client/src/test/*.test.ts` + harness `.svelte` | NEW — unit/component tests |

All commands run from `client/`. Test command: `npm run test:unit -- <file>`.

---

## Task 1: Extract `isActive` into shared nav module

**Files:**
- Create: `client/src/lib/components/shell/nav.ts`
- Test: `client/src/test/nav-active.test.ts`
- Modify: `client/src/lib/components/ui/BottomNav.svelte`

- [ ] **Step 1: Write the failing test**

Create `client/src/test/nav-active.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { isActive } from '$lib/components/shell/nav';

describe('isActive', () => {
	it('returns false when current is undefined', () => {
		expect(isActive('/', undefined)).toBe(false);
	});

	it('matches the root only on exact root path', () => {
		expect(isActive('/', '/')).toBe(true);
		expect(isActive('/', '/workouts')).toBe(false);
	});

	it('matches an exact non-root path', () => {
		expect(isActive('/progress', '/progress')).toBe(true);
	});

	it('matches nested child routes via prefix', () => {
		expect(isActive('/workouts', '/workouts/123')).toBe(true);
	});

	it('does not match unrelated routes', () => {
		expect(isActive('/workouts', '/progress')).toBe(false);
	});

	it('does not treat a path as a prefix of a longer sibling segment', () => {
		expect(isActive('/work', '/workouts')).toBe(false);
	});
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test:unit -- src/test/nav-active.test.ts`
Expected: FAIL — cannot resolve `$lib/components/shell/nav`.

- [ ] **Step 3: Write minimal implementation**

Create `client/src/lib/components/shell/nav.ts`:

```ts
/** Shared active-route logic for BottomNav (mobile) and SideNav (desktop). */
export function isActive(href: string, current?: string): boolean {
	if (!current) return false;
	if (href === '/') return current === '/';
	return current === href || current.startsWith(`${href}/`);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test:unit -- src/test/nav-active.test.ts`
Expected: PASS (6 tests).

- [ ] **Step 5: Refactor BottomNav to use the shared helper**

In `client/src/lib/components/ui/BottomNav.svelte`, delete the local `isActive` function and import the shared one. The `<script lang="ts">` block becomes:

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { isActive } from '$lib/components/shell/nav';

	export type NavItem = {
		href: string;
		label: string;
		icon: Snippet;
	};

	interface Props {
		items: NavItem[];
		current?: string;
		'aria-label'?: string;
	}

	let { items, current, 'aria-label': ariaLabel = 'Primary navigation' }: Props = $props();
</script>
```

The markup using `{@const active = isActive(item.href)}` must now pass `current`. Change that line to:

```svelte
		{@const active = isActive(item.href, current)}
```

Leave everything else (markup, styles) unchanged.

- [ ] **Step 6: Verify nothing regressed**

Run: `npm run test:unit -- src/test/nav-active.test.ts && npm run check`
Expected: tests PASS; `svelte-check` reports no new errors.

- [ ] **Step 7: Commit**

```bash
git add client/src/lib/components/shell/nav.ts client/src/test/nav-active.test.ts client/src/lib/components/ui/BottomNav.svelte
git commit -m "refactor(nav): extract shared isActive helper"
```

---

## Task 2: viewport store

**Files:**
- Create: `client/src/lib/stores/viewport.ts`
- Test: `client/src/test/viewport.test.ts`

- [ ] **Step 1: Write the failing test**

Create `client/src/test/viewport.test.ts`:

```ts
import { get } from 'svelte/store';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { DESKTOP_MIN_WIDTH, isDesktop, isDesktopView } from '$lib/stores/viewport';

type Listener = (e: { matches: boolean }) => void;

function installMatchMedia(initialMatches: boolean) {
	let listeners: Listener[] = [];
	const mql = {
		matches: initialMatches,
		media: '',
		addEventListener: (_type: string, cb: Listener) => {
			listeners.push(cb);
		},
		removeEventListener: (_type: string, cb: Listener) => {
			listeners = listeners.filter((l) => l !== cb);
		}
	};
	Object.defineProperty(window, 'matchMedia', {
		configurable: true,
		writable: true,
		value: vi.fn(() => mql)
	});
	return {
		emit(matches: boolean) {
			mql.matches = matches;
			listeners.forEach((l) => l({ matches }));
		}
	};
}

afterEach(() => {
	vi.restoreAllMocks();
});

describe('isDesktopView', () => {
	it('treats undefined and false as not-desktop (mobile-safe default)', () => {
		expect(isDesktopView(undefined)).toBe(false);
		expect(isDesktopView(false)).toBe(false);
	});

	it('treats true as desktop', () => {
		expect(isDesktopView(true)).toBe(true);
	});
});

describe('isDesktop store', () => {
	it('exposes the 1024px breakpoint constant', () => {
		expect(DESKTOP_MIN_WIDTH).toBe(1024);
	});

	it('reflects the initial matchMedia state on subscribe', () => {
		installMatchMedia(true);
		let value: boolean | undefined;
		const unsub = isDesktop.subscribe((v) => (value = v));
		expect(value).toBe(true);
		unsub();
	});

	it('updates when the media query changes', () => {
		const mm = installMatchMedia(false);
		let value: boolean | undefined;
		const unsub = isDesktop.subscribe((v) => (value = v));
		expect(value).toBe(false);
		mm.emit(true);
		expect(value).toBe(true);
		unsub();
	});
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test:unit -- src/test/viewport.test.ts`
Expected: FAIL — cannot resolve `$lib/stores/viewport`.

- [ ] **Step 3: Write minimal implementation**

Create `client/src/lib/stores/viewport.ts`:

```ts
import { readable } from 'svelte/store';

/**
 * Desktop breakpoint in px. DUPLICATED in the CSS media query inside
 * routes/+layout.svelte (`@media (min-width: 1024px)`) because CSS cannot read
 * JS values. Keep the two in sync.
 */
export const DESKTOP_MIN_WIDTH = 1024;

const QUERY = `(min-width: ${DESKTOP_MIN_WIDTH}px)`;

/**
 * `true` on desktop, `false` on mobile, `undefined` on the server / before mount.
 * Drives per-screen content-tree selection. Treat `undefined` as mobile via
 * `isDesktopView`.
 */
export const isDesktop = readable<boolean | undefined>(undefined, (set) => {
	if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
		return;
	}
	const mql = window.matchMedia(QUERY);
	set(mql.matches);
	const handler = (e: MediaQueryListEvent) => set(e.matches);
	mql.addEventListener('change', handler);
	return () => mql.removeEventListener('change', handler);
});

/** Safe accessor: unknown/`undefined` resolves to mobile. */
export function isDesktopView(value: boolean | undefined): boolean {
	return value === true;
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test:unit -- src/test/viewport.test.ts`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add client/src/lib/stores/viewport.ts client/src/test/viewport.test.ts
git commit -m "feat(desktop): add viewport breakpoint store"
```

---

## Task 3: Extract `toggleTheme` util

**Files:**
- Create: `client/src/lib/components/shell/theme.ts`
- Test: `client/src/test/theme-toggle.test.ts`
- Modify: `client/src/lib/components/shell/AppBar.svelte`

- [ ] **Step 1: Write the failing test**

Create `client/src/test/theme-toggle.test.ts`:

```ts
import { afterEach, describe, expect, it } from 'vitest';
import { toggleTheme } from '$lib/components/shell/theme';

afterEach(() => {
	document.documentElement.classList.remove('dark');
	document.documentElement.removeAttribute('data-theme');
	localStorage.clear();
});

describe('toggleTheme', () => {
	it('switches from light to dark, setting attribute, class and storage', () => {
		toggleTheme();
		const root = document.documentElement;
		expect(root.getAttribute('data-theme')).toBe('dark');
		expect(root.classList.contains('dark')).toBe(true);
		expect(localStorage.getItem('theme')).toBe('dark');
	});

	it('switches from dark back to light, clearing attribute and class', () => {
		const root = document.documentElement;
		root.setAttribute('data-theme', 'dark');
		root.classList.add('dark');
		toggleTheme();
		expect(root.getAttribute('data-theme')).toBe(null);
		expect(root.classList.contains('dark')).toBe(false);
		expect(localStorage.getItem('theme')).toBe('light');
	});
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test:unit -- src/test/theme-toggle.test.ts`
Expected: FAIL — cannot resolve `$lib/components/shell/theme`.

- [ ] **Step 3: Write minimal implementation**

Create `client/src/lib/components/shell/theme.ts`:

```ts
/** Toggle light/dark theme on <html>, persisting the choice to localStorage. */
export function toggleTheme(): void {
	const root = document.documentElement;
	const isDark = root.getAttribute('data-theme') === 'dark';
	const next = isDark ? 'light' : 'dark';
	if (next === 'dark') {
		root.setAttribute('data-theme', 'dark');
		root.classList.add('dark');
	} else {
		root.removeAttribute('data-theme');
		root.classList.remove('dark');
	}
	try {
		localStorage.setItem('theme', next);
	} catch {
		/* ignore */
	}
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test:unit -- src/test/theme-toggle.test.ts`
Expected: PASS (2 tests).

- [ ] **Step 5: Refactor AppBar to use the shared util**

In `client/src/lib/components/shell/AppBar.svelte`, delete the inline `toggleTheme` function and import the shared one. The `<script lang="ts">` block becomes:

```svelte
<script lang="ts">
	import { Logo } from '$lib/components/ui';
	import { toggleTheme } from '$lib/components/shell/theme';
	interface Props {
		onLogout?: () => void;
	}
	let { onLogout }: Props = $props();
</script>
```

Leave the markup (which already calls `onclick={toggleTheme}`) and styles unchanged.

- [ ] **Step 6: Verify nothing regressed**

Run: `npm run test:unit -- src/test/theme-toggle.test.ts && npm run check`
Expected: tests PASS; no new `svelte-check` errors.

- [ ] **Step 7: Commit**

```bash
git add client/src/lib/components/shell/theme.ts client/src/test/theme-toggle.test.ts client/src/lib/components/shell/AppBar.svelte
git commit -m "refactor(shell): extract shared toggleTheme util"
```

---

## Task 4: MasterDetail primitive

**Files:**
- Create: `client/src/lib/components/ui/MasterDetail.svelte`
- Create: `client/src/test/fixtures/MasterDetailFull.svelte` (harness, both panes)
- Create: `client/src/test/fixtures/MasterDetailNoDetail.svelte` (harness, empty case)
- Test: `client/src/test/master-detail.test.ts`
- Modify: `client/src/lib/components/ui/index.ts`

- [ ] **Step 1: Write the harness fixtures**

Create `client/src/test/fixtures/MasterDetailFull.svelte`:

```svelte
<script lang="ts">
	import MasterDetail from '$lib/components/ui/MasterDetail.svelte';
</script>

<MasterDetail>
	{#snippet list()}
		<div data-testid="list-content">LIST</div>
	{/snippet}
	{#snippet detail()}
		<div data-testid="detail-content">DETAIL</div>
	{/snippet}
	{#snippet empty()}
		<div data-testid="empty-content">Nothing selected</div>
	{/snippet}
</MasterDetail>
```

Create `client/src/test/fixtures/MasterDetailNoDetail.svelte`:

```svelte
<script lang="ts">
	import MasterDetail from '$lib/components/ui/MasterDetail.svelte';
</script>

<MasterDetail>
	{#snippet list()}
		<div data-testid="list-content">LIST</div>
	{/snippet}
	{#snippet empty()}
		<div data-testid="empty-content">Nothing selected</div>
	{/snippet}
</MasterDetail>
```

- [ ] **Step 2: Write the failing test**

Create `client/src/test/master-detail.test.ts`:

```ts
import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import MasterDetailFull from './fixtures/MasterDetailFull.svelte';
import MasterDetailNoDetail from './fixtures/MasterDetailNoDetail.svelte';

describe('MasterDetail', () => {
	it('renders the list snippet in the list pane', () => {
		const { getByTestId } = render(MasterDetailFull);
		const list = getByTestId('list-content');
		expect(list).toBeInTheDocument();
		expect(list.closest('.list')).not.toBeNull();
	});

	it('renders the detail snippet in the detail pane when provided', () => {
		const { getByTestId, queryByTestId } = render(MasterDetailFull);
		const detail = getByTestId('detail-content');
		expect(detail).toBeInTheDocument();
		expect(detail.closest('.detail')).not.toBeNull();
		expect(queryByTestId('empty-content')).toBeNull();
	});

	it('renders the empty snippet when no detail snippet is provided', () => {
		const { getByTestId, queryByTestId } = render(MasterDetailNoDetail);
		expect(getByTestId('empty-content')).toBeInTheDocument();
		expect(queryByTestId('detail-content')).toBeNull();
	});
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `npm run test:unit -- src/test/master-detail.test.ts`
Expected: FAIL — cannot resolve `$lib/components/ui/MasterDetail.svelte`.

- [ ] **Step 4: Write minimal implementation**

Create `client/src/lib/components/ui/MasterDetail.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Left pane — the list of items. */
		list: Snippet;
		/** Right pane — the detail of the selected item. Omit to show `empty`. */
		detail?: Snippet;
		/** Shown in the detail pane when `detail` is not provided. */
		empty?: Snippet;
	}

	let { list, detail, empty }: Props = $props();
</script>

<div class="master-detail">
	<aside class="list">{@render list()}</aside>
	<section class="detail">
		{#if detail}
			{@render detail()}
		{:else if empty}
			{@render empty()}
		{/if}
	</section>
</div>

<style>
	.master-detail {
		display: flex;
		gap: 16px;
		min-height: 0;
		height: 100%;
	}
	.list {
		width: 320px;
		flex: none;
		overflow-y: auto;
		min-height: 0;
	}
	.detail {
		flex: 1;
		min-width: 0;
		overflow-y: auto;
		min-height: 0;
	}
</style>
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npm run test:unit -- src/test/master-detail.test.ts`
Expected: PASS (3 tests).

- [ ] **Step 6: Export MasterDetail**

In `client/src/lib/components/ui/index.ts`, add after the `BottomNav` exports:

```ts
export { default as MasterDetail } from './MasterDetail.svelte';
```

- [ ] **Step 7: Verify check passes**

Run: `npm run check`
Expected: no new `svelte-check` errors.

- [ ] **Step 8: Commit**

```bash
git add client/src/lib/components/ui/MasterDetail.svelte client/src/lib/components/ui/index.ts client/src/test/master-detail.test.ts client/src/test/fixtures/MasterDetailFull.svelte client/src/test/fixtures/MasterDetailNoDetail.svelte
git commit -m "feat(desktop): add MasterDetail layout primitive"
```

---

## Task 5: SideNav component

**Files:**
- Create: `client/src/lib/components/shell/SideNav.svelte`
- Test: `client/src/test/fixtures/SideNavHarness.svelte` (provides icon snippets + items)
- Test: `client/src/test/side-nav.test.ts`

- [ ] **Step 1: Write the harness fixture**

Create `client/src/test/fixtures/SideNavHarness.svelte`:

```svelte
<script lang="ts">
	import SideNav from '$lib/components/shell/SideNav.svelte';
	import type { NavItem } from '$lib/components/ui';

	interface Props {
		current?: string;
		onLogout?: () => void;
	}
	let { current, onLogout }: Props = $props();
</script>

{#snippet icon()}
	<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /></svg>
{/snippet}

{@const items = [
	{ href: '/', label: 'Today', icon },
	{ href: '/templates', label: 'Plans', icon },
	{ href: '/progress', label: 'Progress', icon }
] satisfies NavItem[]}

<SideNav {items} {current} {onLogout} />
```

- [ ] **Step 2: Write the failing test**

Create `client/src/test/side-nav.test.ts`:

```ts
import { render } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import SideNavHarness from './fixtures/SideNavHarness.svelte';

describe('SideNav', () => {
	it('renders one link per nav item with its label', () => {
		const { getByRole } = render(SideNavHarness, { current: '/' });
		expect(getByRole('link', { name: /Today/ })).toBeInTheDocument();
		expect(getByRole('link', { name: /Plans/ })).toBeInTheDocument();
		expect(getByRole('link', { name: /Progress/ })).toBeInTheDocument();
	});

	it('marks the current route active via aria-current', () => {
		const { getByRole } = render(SideNavHarness, { current: '/progress' });
		const active = getByRole('link', { name: /Progress/ });
		expect(active).toHaveAttribute('aria-current', 'page');
		expect(getByRole('link', { name: /Today/ })).not.toHaveAttribute('aria-current');
	});

	it('renders a logout button that fires the callback when onLogout is provided', async () => {
		const onLogout = vi.fn();
		const { getByRole } = render(SideNavHarness, { current: '/', onLogout });
		getByRole('button', { name: /Log out/i }).click();
		expect(onLogout).toHaveBeenCalledOnce();
	});

	it('omits the logout button when onLogout is not provided', () => {
		const { queryByRole } = render(SideNavHarness, { current: '/' });
		expect(queryByRole('button', { name: /Log out/i })).toBeNull();
	});
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `npm run test:unit -- src/test/side-nav.test.ts`
Expected: FAIL — cannot resolve `$lib/components/shell/SideNav.svelte`.

- [ ] **Step 4: Write minimal implementation**

Create `client/src/lib/components/shell/SideNav.svelte`:

```svelte
<script lang="ts">
	import { Logo, type NavItem } from '$lib/components/ui';
	import { isActive } from '$lib/components/shell/nav';
	import { toggleTheme } from '$lib/components/shell/theme';

	interface Props {
		items: NavItem[];
		current?: string;
		onLogout?: () => void;
	}
	let { items, current, onLogout }: Props = $props();
</script>

<nav class="sidenav" aria-label="Primary navigation">
	<a class="brand" href="/" aria-label="SwoleMate home">
		<Logo size={30} />
		<span class="name">SwoleMate</span>
	</a>

	<div class="items">
		{#each items as item (item.href)}
			{@const active = isActive(item.href, current)}
			<a href={item.href} class:active aria-current={active ? 'page' : undefined}>
				<span class="ico">{@render item.icon()}</span>
				<span class="lbl">{item.label}</span>
			</a>
		{/each}
	</div>

	<div class="foot">
		<button type="button" class="foot-btn theme-toggle" aria-label="Toggle dark mode" onclick={toggleTheme}>
			<svg class="ico-moon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
			</svg>
			<svg class="ico-sun" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="4" />
				<path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
			</svg>
			<span class="lbl">Theme</span>
		</button>
		{#if onLogout}
			<button type="button" class="foot-btn" aria-label="Log out" onclick={onLogout}>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
					<polyline points="16 17 21 12 16 7" />
					<line x1="21" y1="12" x2="9" y2="12" />
				</svg>
				<span class="lbl">Log out</span>
			</button>
		{/if}
	</div>
</nav>

<style>
	.sidenav {
		display: flex;
		flex-direction: column;
		gap: 4px;
		width: 220px;
		height: 100dvh;
		padding: 18px 14px;
		background: var(--surface-deep);
		color: var(--on-deep);
		border-right: 1px solid var(--on-deep-line);
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 9px;
		margin-bottom: 18px;
		text-decoration: none;
		color: inherit;
	}
	.name {
		font: 800 16px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.01em;
	}
	.items {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.items a {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 11px;
		border-radius: 10px;
		font: 700 13px/1 'Onest', system-ui, sans-serif;
		color: var(--on-deep-soft);
		text-decoration: none;
		transition:
			background-color 160ms ease,
			color 160ms ease;
	}
	.items a:hover {
		color: var(--on-deep);
	}
	.items a.active {
		background: var(--clay);
		color: #fff;
		box-shadow: 0 6px 16px -8px var(--clay);
	}
	.ico {
		width: 18px;
		height: 18px;
		display: grid;
		place-items: center;
		flex: none;
	}
	.ico :global(svg) {
		width: 18px;
		height: 18px;
	}
	.foot {
		margin-top: auto;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding-top: 10px;
		border-top: 1px solid var(--on-deep-line);
	}
	.foot-btn {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 11px;
		border: 0;
		border-radius: 10px;
		background: transparent;
		color: var(--on-deep-soft);
		font: 700 12px/1 'Onest', system-ui, sans-serif;
		cursor: pointer;
		text-align: left;
		transition: color 160ms ease;
	}
	.foot-btn:hover {
		color: var(--on-deep);
	}
	.theme-toggle .ico-sun {
		display: none;
	}
	.theme-toggle .ico-moon {
		display: inline-block;
	}
	:global([data-theme='dark']) .theme-toggle .ico-sun {
		display: inline-block;
	}
	:global([data-theme='dark']) .theme-toggle .ico-moon {
		display: none;
	}
</style>
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npm run test:unit -- src/test/side-nav.test.ts`
Expected: PASS (4 tests).

- [ ] **Step 6: Verify check passes**

Run: `npm run check`
Expected: no new `svelte-check` errors.

- [ ] **Step 7: Commit**

```bash
git add client/src/lib/components/shell/SideNav.svelte client/src/test/side-nav.test.ts client/src/test/fixtures/SideNavHarness.svelte
git commit -m "feat(desktop): add SideNav rail component"
```

---

## Task 6: DesktopTopBar component

**Files:**
- Create: `client/src/lib/components/shell/DesktopTopBar.svelte`
- Test: `client/src/test/fixtures/DesktopTopBarHarness.svelte`
- Test: `client/src/test/desktop-topbar.test.ts`

- [ ] **Step 1: Write the harness fixture**

Create `client/src/test/fixtures/DesktopTopBarHarness.svelte`:

```svelte
<script lang="ts">
	import DesktopTopBar from '$lib/components/shell/DesktopTopBar.svelte';

	interface Props {
		title: string;
		subtitle?: string;
		withActions?: boolean;
	}
	let { title, subtitle, withActions = false }: Props = $props();
</script>

{#if withActions}
	<DesktopTopBar {title} {subtitle}>
		{#snippet actions()}
			<button data-testid="action-btn">New</button>
		{/snippet}
	</DesktopTopBar>
{:else}
	<DesktopTopBar {title} {subtitle} />
{/if}
```

- [ ] **Step 2: Write the failing test**

Create `client/src/test/desktop-topbar.test.ts`:

```ts
import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import DesktopTopBarHarness from './fixtures/DesktopTopBarHarness.svelte';

describe('DesktopTopBar', () => {
	it('renders the title', () => {
		const { getByText } = render(DesktopTopBarHarness, { title: 'Plans' });
		expect(getByText('Plans')).toBeInTheDocument();
	});

	it('renders the subtitle when provided', () => {
		const { getByText } = render(DesktopTopBarHarness, { title: 'Plans', subtitle: 'your templates' });
		expect(getByText('your templates')).toBeInTheDocument();
	});

	it('renders the actions snippet when provided', () => {
		const { getByTestId } = render(DesktopTopBarHarness, { title: 'Plans', withActions: true });
		expect(getByTestId('action-btn')).toBeInTheDocument();
	});

	it('omits the actions region when no actions snippet is provided', () => {
		const { container } = render(DesktopTopBarHarness, { title: 'Plans' });
		expect(container.querySelector('.actions')).toBeNull();
	});
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `npm run test:unit -- src/test/desktop-topbar.test.ts`
Expected: FAIL — cannot resolve `$lib/components/shell/DesktopTopBar.svelte`.

- [ ] **Step 4: Write minimal implementation**

Create `client/src/lib/components/shell/DesktopTopBar.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		subtitle?: string;
		/** Right-aligned per-screen actions (buttons, etc.). */
		actions?: Snippet;
	}
	let { title, subtitle, actions }: Props = $props();
</script>

<header class="desktop-topbar">
	<h1 class="title">
		{title}{#if subtitle}<em>— {subtitle}</em>{/if}
	</h1>
	{#if actions}
		<div class="actions">{@render actions()}</div>
	{/if}
</header>

<style>
	.desktop-topbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 16px 24px;
		border-bottom: 1px solid var(--line);
		background: color-mix(in oklab, var(--bg) 86%, transparent);
		backdrop-filter: blur(12px);
	}
	.title {
		margin: 0;
		font: 800 20px/1.05 'Onest', system-ui, sans-serif;
		letter-spacing: -0.02em;
		color: var(--ink);
	}
	.title em {
		font: italic 400 16px/1 'Instrument Serif', serif;
		color: var(--ink-soft);
		margin-left: 6px;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: none;
	}
</style>
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npm run test:unit -- src/test/desktop-topbar.test.ts`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit**

```bash
git add client/src/lib/components/shell/DesktopTopBar.svelte client/src/test/desktop-topbar.test.ts client/src/test/fixtures/DesktopTopBarHarness.svelte
git commit -m "feat(desktop): add DesktopTopBar header component"
```

---

## Task 7: Wire the dual shell into the layout

**Files:**
- Modify: `client/src/routes/+layout.svelte`

This task has no unit test (layout wiring is verified manually in Task 8). The change renders both shells and uses a CSS media query to show exactly one, so chrome never flashes.

- [ ] **Step 1: Add the SideNav import**

In `client/src/routes/+layout.svelte`, add to the import block (after the `AppBar` import on line 9):

```svelte
	import SideNav from '$lib/components/shell/SideNav.svelte';
```

- [ ] **Step 2: Replace the authenticated shell markup**

Replace the entire `{:else}` branch (currently the `<div class="shell">…</div>` block, lines 130–156) with a dual-shell version. The `navItems` array is built once and passed to both navs:

```svelte
{:else}
	{@const navItems = [
		{ href: '/', label: 'Today', icon: iconToday },
		{ href: '/templates', label: 'Plans', icon: iconPlans },
		{ href: '/workouts', label: 'History', icon: iconHistory },
		{ href: '/progress', label: 'Progress', icon: iconProgress },
		{ href: '/more', label: 'More', icon: iconMore }
	] satisfies NavItem[]}

	{#snippet offlineBanner()}
		{#if $authState.offline}
			<div class="offline-wrap">
				<div class="offline">
					<span class="dot"></span>
					Offline mode — showing cached data. Some actions are disabled.
				</div>
			</div>
		{/if}
	{/snippet}

	<!-- Mobile shell (shown < 1024px via CSS) -->
	<div class="shell shell-mobile">
		<AppBar onLogout={$authState.status === 'authenticated' ? logout : undefined} />
		{@render offlineBanner()}
		<main>
			{@render children?.()}
		</main>
		<BottomNav items={navItems} current={currentPath} />
	</div>

	<!-- Desktop shell (shown >= 1024px via CSS) -->
	<div class="shell shell-desktop">
		<SideNav
			items={navItems}
			current={currentPath}
			onLogout={$authState.status === 'authenticated' ? logout : undefined}
		/>
		<div class="desktop-content">
			{@render offlineBanner()}
			<main class="desktop-main">
				{@render children?.()}
			</main>
		</div>
	</div>
{/if}
```

> Note: `DesktopTopBar` is intentionally NOT rendered here — it is a per-screen header each future screen slice renders inside its own content, with its own title/actions. The foundation only provides the component and the content column.

- [ ] **Step 3: Replace the `<style>` block**

Replace the existing `<style>` block (lines 158–196) with:

```svelte
<style>
	/* Default (mobile-first): show mobile shell, hide desktop shell. */
	.shell-mobile {
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
	}
	.shell-desktop {
		display: none;
	}
	main {
		flex: 1;
		padding: 14px 18px calc(96px + env(safe-area-inset-bottom));
		max-width: 720px;
		width: 100%;
		margin: 0 auto;
	}
	.offline-wrap {
		display: flex;
		justify-content: center;
		padding: 8px 18px 0;
	}
	.offline {
		padding: 8px 14px;
		font:
			600 12px/1.3 'Onest',
			system-ui,
			sans-serif;
		color: var(--warn);
		background: color-mix(in oklab, var(--warn) 14%, var(--card));
		border: 1px solid color-mix(in oklab, var(--warn) 30%, var(--line));
		border-radius: 999px;
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}
	.dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--warn);
	}

	/* Desktop: swap shells. Breakpoint MUST match DESKTOP_MIN_WIDTH (1024) in
	   lib/stores/viewport.ts. */
	@media (min-width: 1024px) {
		.shell-mobile {
			display: none;
		}
		.shell-desktop {
			display: flex;
			min-height: 100dvh;
		}
		.desktop-content {
			flex: 1;
			min-width: 0;
			display: flex;
			flex-direction: column;
		}
		.desktop-main {
			flex: 1;
			padding: 24px;
			max-width: none;
			margin: 0;
		}
	}
</style>
```

- [ ] **Step 4: Verify check passes**

Run: `npm run check`
Expected: no new `svelte-check` errors.

- [ ] **Step 5: Run the full unit suite (no regressions)**

Run: `npm run test:unit`
Expected: all tests PASS.

- [ ] **Step 6: Commit**

```bash
git add client/src/routes/+layout.svelte
git commit -m "feat(desktop): wire dual mobile/desktop shell into layout"
```

---

## Task 8: Manual verification

**Files:** none (verification only).

- [ ] **Step 1: Start the dev server**

Run (from `client/`): `npm run dev`
Open the printed URL (default `http://localhost:2470`). Log in if prompted.

- [ ] **Step 2: Verify desktop shell at wide width**

Widen the browser to ≥1024px. Confirm:
- The dark `SideNav` rail is on the left with logo, the five nav items, and theme + logout pinned at the bottom.
- The bottom nav and mobile AppBar are NOT visible.
- Clicking nav items navigates and the active item shows the clay pill.
- Theme toggle in the rail flips light/dark; logout works.
- Content fills the column (no 720px clamp).

- [ ] **Step 3: Verify mobile shell at narrow width**

Resize the browser below 1024px (or use devtools device mode). Confirm:
- The AppBar (top) and BottomNav (bottom) are back, identical to before.
- The SideNav is NOT visible.
- Today/Plans/History/Progress/More all work as before.

- [ ] **Step 4: Verify the transition + offline banner**

- Drag the window across the 1024px boundary repeatedly: chrome swaps cleanly with no flash of the wrong shell, no console errors, content not duplicated.
- If you can simulate offline (devtools → Network → Offline, then reload a cached route), confirm the offline banner appears in BOTH mobile and desktop shells.

- [ ] **Step 5: Final full check + commit any fixes**

Run: `npm run check && npm run test:unit`
Expected: all green. If manual verification surfaced issues, fix them, re-run, and commit with a descriptive message.

---

## Self-Review Notes

- **Spec coverage:** viewport store (Task 2), CSS-chrome/JS-content breakpoint (Tasks 2 + 7), shared nav items + `isActive` (Tasks 1 + 7), `SideNav` always-dark (Task 5), `DesktopTopBar` (Task 6), `MasterDetail` (Task 4), `+layout` dual shell + offline banner in both modes (Task 7), testing approach (Tasks 1–6 unit, Task 8 manual). All spec sections mapped.
- **Spec deviation (intentional):** the spec listed nav *items* living in `nav.ts`; because items carry `icon: Snippet` (snippets cannot live in `.ts`), items stay defined once in `+layout.svelte` and are passed to both navs, while only `isActive` moves to `nav.ts`. Single-source-of-truth goal preserved.
- **Type consistency:** `NavItem` imported from `$lib/components/ui` everywhere; `isActive(href, current)`, `toggleTheme()`, `isDesktop`/`isDesktopView`/`DESKTOP_MIN_WIDTH`, and `MasterDetail` props (`list`/`detail`/`empty`) are used identically across tasks.
- **Note:** `isDesktop`/`isDesktopView` are delivered as foundation primitives for screen slices to branch their content tree; the foundation shell itself uses the CSS breakpoint, so no screen consumes the store until slice 2+.
