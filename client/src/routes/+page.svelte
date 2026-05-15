<script lang="ts">
	import {
		Btn,
		Card,
		Chip,
		Badge,
		Chk,
		Spill,
		SetPillList,
		StepperPill,
		PageHero,
		SegmentedTabs,
		BottomNav,
		type NavItem
	} from '$lib/components/ui';

	let reps = $state(8);
	let weight = $state(105);
	let trackReps = $state(true);
	let trackTime = $state(false);
	let trackWeight = $state(true);
	let perSide = $state(false);

	type Tab = 'overview' | 'exercise' | 'trends';
	let tab = $state<Tab>('overview');

	// Sample sets demonstrating spill grouping + per-side modes
	const benchSets = [
		{ id: 1, reps: 10, weight: 100 },
		{ id: 2, reps: 9, weight: 102.5 },
		{ id: 3, reps: 8, weight: 105 }
	];
	const grouped = [
		{ id: 1, reps: 10, weight: 100 },
		{ id: 2, reps: 10, weight: 100 },
		{ id: 3, reps: 10, weight: 100 },
		{ id: 4, reps: 8, weight: 105 }
	];

	const navItems: NavItem[] = [
		{
			href: '/',
			label: 'Today',
			icon: heart
		},
		{
			href: '/plans',
			label: 'Plans',
			icon: plans
		},
		{
			href: '/history',
			label: 'History',
			icon: clock
		},
		{
			href: '/progress',
			label: 'Progress',
			icon: chart
		},
		{
			href: '/more',
			label: 'More',
			icon: more
		}
	];

	function toggleTheme() {
		const root = document.documentElement;
		const isDark = root.getAttribute('data-theme') === 'dark';
		const next = isDark ? 'light' : 'dark';
		root.setAttribute('data-theme', next);
		root.classList.toggle('dark', next === 'dark');
		try {
			localStorage.setItem('theme', next);
		} catch {
			/* ignore */
		}
	}
</script>

{#snippet heart()}
	<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4"
		><path d="M12 21s-7-4.5-7-11a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 6.5-7 11-7 11z" /></svg
	>
{/snippet}
{#snippet plans()}
	<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
		><rect x="4" y="4" width="16" height="16" rx="3" /><path d="M4 9h16" /></svg
	>
{/snippet}
{#snippet clock()}
	<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
		><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg
	>
{/snippet}
{#snippet chart()}
	<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
		><path d="M4 19h16" /><path d="M6 16l4-5 4 3 5-7" /></svg
	>
{/snippet}
{#snippet more()}
	<span style="font-weight: 800;">⋯</span>
{/snippet}

<main class="page">
	<header class="page-head">
		<div class="brand">
			<span class="logo">SM</span>
			<span class="name">SwoleMate</span>
			<span class="ver">design-system v0.1</span>
		</div>
		<Btn variant="ghost" size="sm" onclick={toggleTheme} aria-label="Toggle theme">
			Toggle theme
		</Btn>
	</header>

	<PageHero kicker="► Component showcase">
		{#snippet title()}Step 2 done. <em>Components live.</em>{/snippet}
		{#snippet sub()}
			Every primitive from the spec is implemented as a Svelte 5 component. Step 3 (porting Today)
			starts next.
		{/snippet}
		<div class="hero-stats">
			<div class="cell"><div class="k">Components</div><div class="v">11</div></div>
			<div class="cell"><div class="k">Tokens</div><div class="v">26</div></div>
			<div class="cell"><div class="k">Tests</div><div class="v">69 ✓</div></div>
			<div class="cell"><div class="k">Skeleton</div><div class="v">0</div></div>
		</div>
	</PageHero>

	<!-- ─── Buttons ───────────────────────────────────────────── -->
	<Card>
		{#snippet title()}Buttons <em>— 6 variants</em>{/snippet}
		{#snippet lede()}One primary, three secondaries, one celebratory, one icon-only.{/snippet}
		<div class="row">
			<Btn variant="primary">▶ Log set 4 of 5</Btn>
			<Btn variant="success">✓ Mark done</Btn>
			<Btn variant="ink">Sync now</Btn>
			<Btn variant="soft">Collapse</Btn>
			<Btn variant="ghost">Use template</Btn>
			<Btn variant="icon" aria-label="Delete">✕</Btn>
		</div>
	</Card>

	<!-- ─── Spill pills ───────────────────────────────────────── -->
	<Card>
		{#snippet title()}Spill pills <em>— segmented set rendering</em>{/snippet}
		{#snippet lede()}
			Single sets, count-grouped sets, timed sets, bodyweight, and a PR marker. Heatmap intensity
			scales with the relative weight inside the group.
		{/snippet}

		<h4 class="sub">Current sets (heatmap)</h4>
		<SetPillList sets={benchSets} prGroupIndex={2} />

		<h4 class="sub">Grouped sets (count prefix)</h4>
		<SetPillList sets={grouped} />

		<h4 class="sub">Single pills</h4>
		<div class="row">
			<Spill count={2} reps={10} weight="BW" bodyweight intensity={0.5} />
			<Spill reps={8} weight="22.5kg/side" intensity={0.7} />
			<Spill reps={6} weight="27.5/22.5kg" intensity={0.85} />
			<Spill count={3} duration="0:30" />
			<Spill reps={8} weight="105kg" intensity={0.85} pr />
		</div>
	</Card>

	<!-- ─── Stepper pill ──────────────────────────────────────── -->
	<Card>
		{#snippet title()}Stepper pill <em>— numeric input</em>{/snippet}
		{#snippet lede()}
			Two-handed steppers with a centred value and optional unit suffix.
		{/snippet}
		<div class="row">
			<StepperPill bind:value={reps} label="Reps" min={0} max={50} />
			<StepperPill bind:value={weight} label="Weight" step={2.5} min={0} unit="kg" />
		</div>
		<p class="aside">Bound state: <code>reps={reps}</code> · <code>weight={weight}</code></p>
	</Card>

	<!-- ─── Chk / Chip / Badge ───────────────────────────────── -->
	<Card>
		{#snippet title()}Toggles, chips &amp; badges{/snippet}
		<h4 class="sub">Tracking toggle (Chk)</h4>
		<div class="row">
			<Chk bind:checked={trackReps} label="Reps" />
			<Chk bind:checked={trackTime} label="Time" />
			<Chk bind:checked={trackWeight} label="Weight" />
			<Chk bind:checked={perSide} label="Per-side" />
		</div>

		<h4 class="sub">Chips</h4>
		<div class="row">
			<Chip>grip: pronated</Chip>
			<Chip>rom: full</Chip>
			<Chip size="xs">+2</Chip>
		</div>

		<h4 class="sub">Badges</h4>
		<div class="row">
			<Badge tone="done">Done</Badge>
			<Badge tone="live">In progress</Badge>
			<Badge tone="soft">Edit</Badge>
			<Badge tone="warn">Offline</Badge>
			<Badge tone="pr">All-time PR</Badge>
		</div>
	</Card>

	<!-- ─── Segmented tabs ───────────────────────────────────── -->
	<Card>
		{#snippet title()}Segmented tabs <em>— 2-3 positions</em>{/snippet}
		<SegmentedTabs
			items={[
				{ id: 'overview', label: 'Overview' },
				{ id: 'exercise', label: 'Exercise' },
				{ id: 'trends', label: 'Trends' }
			]}
			bind:selected={tab}
			aria-label="Progress sections"
		/>
		<p class="aside">Selected: <code>{tab}</code></p>
	</Card>

	<div style="height: 100px"></div>
</main>

<BottomNav items={navItems} current="/" />

<style>
	.page {
		max-width: 720px;
		margin: 0 auto;
		padding: 24px 18px 120px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.page-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: 6px;
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.logo {
		width: 32px;
		height: 32px;
		border-radius: 9px;
		background: linear-gradient(135deg, var(--clay-2), var(--clay));
		color: white;
		display: grid;
		place-items: center;
		font: 800 13px/1 'Onest', system-ui, sans-serif;
		box-shadow: 0 4px 10px -3px rgba(255, 94, 31, 0.55);
	}
	.name {
		font: 800 17px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.01em;
	}
	.ver {
		font: 600 11px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}

	.hero-stats {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 8px;
	}
	.cell {
		background: color-mix(in oklab, var(--on-deep) 7%, transparent);
		border: 1px solid color-mix(in oklab, var(--on-deep) 12%, transparent);
		border-radius: 12px;
		padding: 12px 14px;
	}
	.k {
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--on-deep-soft);
	}
	.v {
		margin-top: 6px;
		font: 800 22px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.025em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
	}

	.sub {
		margin: 16px 0 8px;
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.sub:first-of-type {
		margin-top: 8px;
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		align-items: center;
	}

	.aside {
		margin: 10px 0 0;
		font: italic 400 12px/1.4 'Instrument Serif';
		color: var(--ink-soft);
	}
	.aside :global(code) {
		font: 700 11px/1 'JetBrains Mono', monospace;
		padding: 2px 6px;
		border-radius: 5px;
		background: var(--card-3);
		color: var(--clay-text);
		border: 1px solid var(--line);
	}

	@media (min-width: 768px) {
		.hero-stats {
			grid-template-columns: repeat(4, minmax(0, 1fr));
		}
	}
</style>
