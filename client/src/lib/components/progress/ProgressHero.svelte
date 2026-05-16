<script lang="ts">
	import { PageHero } from '$lib/components/ui';

	interface Props {
		consistencyDone: number;
		consistencyWindow: number;
		totalWorkouts: number;
		perWeek: number;
		perWeekDelta?: number;
		avgDurationMin: number;
		avgDurationDelta?: number;
		focusExercise?: string;
		loading?: boolean;
		error?: string | null;
		onRefresh?: () => void;
	}
	let {
		consistencyDone,
		consistencyWindow,
		totalWorkouts,
		perWeek,
		perWeekDelta,
		avgDurationMin,
		avgDurationDelta,
		focusExercise,
		loading = false,
		error = null,
		onRefresh
	}: Props = $props();

	const RING_CIRC = 427;
	let pct = $derived(consistencyWindow > 0 ? Math.min(1, consistencyDone / consistencyWindow) : 0);
	let dashOffset = $derived(RING_CIRC * (1 - Math.max(0.06, pct)));

	function fmtSigned(n: number | undefined, suffix = ''): string | undefined {
		if (n === undefined || n === 0) return undefined;
		const sign = n > 0 ? '+' : '−';
		return `${sign} ${Math.abs(n)}${suffix}`;
	}
</script>

<PageHero kicker="► Progress · last 30 days">
	{#snippet title()}Going strong, <em>by the numbers.</em>{/snippet}
	{#snippet sub()}Consistency ring is days trained ÷ days in window.{/snippet}

	<div class="block">
		<div class="ring">
			<svg width="150" height="150" viewBox="0 0 150 150" aria-hidden="true">
				<circle
					cx="75"
					cy="75"
					r="68"
					stroke="rgba(243,236,225,0.1)"
					stroke-width="10"
					fill="none"
				/>
				<circle
					cx="75"
					cy="75"
					r="68"
					stroke="url(#progress-ring)"
					stroke-width="10"
					fill="none"
					stroke-dasharray={RING_CIRC}
					stroke-dashoffset={dashOffset}
					stroke-linecap="round"
					transform="rotate(-90 75 75)"
				/>
				<defs>
					<linearGradient id="progress-ring" x1="0" y1="0" x2="1" y2="1">
						<stop offset="0%" stop-color="#ff7a2a" />
						<stop offset="100%" stop-color="#ff5e1f" />
					</linearGradient>
				</defs>
			</svg>
			<div class="center">
				<span class="big">{consistencyDone}<small>/{consistencyWindow}</small></span>
			</div>
		</div>

		<div class="stats">
			<div class="cell">
				<div class="k">Total</div>
				<div class="v">{totalWorkouts}</div>
			</div>
			<div class="cell">
				<div class="k">Per week</div>
				<div class="v">{perWeek.toFixed(1)}</div>
				{#if fmtSigned(perWeekDelta)}
					<div class="d" class:up={(perWeekDelta ?? 0) > 0} class:down={(perWeekDelta ?? 0) < 0}>
						{fmtSigned(perWeekDelta)} last 4w
					</div>
				{/if}
			</div>
			<div class="cell">
				<div class="k">Avg duration</div>
				<div class="v">{avgDurationMin}<small>m</small></div>
				{#if fmtSigned(avgDurationDelta, 'm')}
					<div
						class="d"
						class:up={(avgDurationDelta ?? 0) > 0}
						class:down={(avgDurationDelta ?? 0) < 0}
					>
						{fmtSigned(avgDurationDelta, 'm')} last 4w
					</div>
				{/if}
			</div>
			<div class="cell focus">
				<div class="k">Focus</div>
				<div class="v fx">{focusExercise || '—'}</div>
			</div>
		</div>
	</div>

	{#snippet actions()}
		<div class="foot">
			{#if error}<div class="err">{error}</div>{/if}
			<button class="refresh" type="button" onclick={onRefresh} disabled={loading}>
				↻ Refresh
			</button>
		</div>
	{/snippet}
</PageHero>

<style>
	.block {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		position: relative;
	}
	.ring {
		width: 150px;
		height: 150px;
		position: relative;
		flex: none;
		animation: ring-breathe 5s ease-in-out infinite;
	}
	@keyframes ring-breathe {
		50% {
			transform: scale(1.03);
		}
	}
	.center {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
	}
	.big {
		font:
			800 32px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.04em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
		transform: translateY(-1px);
		display: flex;
		align-items: baseline;
	}
	.big small {
		font:
			500 13px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
		margin-left: 2px;
	}

	.stats {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 8px;
		width: 100%;
	}
	.cell {
		background: color-mix(in oklab, var(--on-deep) 7%, transparent);
		border: 1px solid color-mix(in oklab, var(--on-deep) 12%, transparent);
		border-radius: 12px;
		padding: 12px 14px;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.k {
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--on-deep-soft);
	}
	.v {
		font:
			800 22px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.025em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
		display: flex;
		align-items: baseline;
		gap: 4px;
	}
	.v small {
		font:
			500 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
		font-weight: 600;
	}
	.v.fx {
		font-size: 15px;
		letter-spacing: -0.01em;
		color: var(--clay-2);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.d {
		font:
			600 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
	}
	.d.up {
		color: var(--sage);
	}
	.d.down {
		color: var(--clay-2);
	}

	.foot {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		gap: 8px;
	}
	.refresh {
		padding: 10px 14px;
		border-radius: 999px;
		border: 1px solid var(--on-deep-line);
		background: color-mix(in oklab, var(--on-deep) 6%, transparent);
		color: var(--on-deep);
		font:
			700 12px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.refresh:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}
	.err {
		font:
			600 12px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--clay-2);
		flex: 1;
	}
</style>
