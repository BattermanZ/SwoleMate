<script lang="ts">
	/**
	 * Segmented set pill: [count?] · [reps?] · [duration?] · [weight?]
	 *
	 * Renders one row of set data as a horizontally-segmented capsule. The
	 * weight cell's tint scales with `intensity` (0–1) so heavier sets pop
	 * within a group of pills. Use {@link SetPillList} to render a list of
	 * sets with automatic grouping + intensity.
	 *
	 * For PRs, set `pr` — the pill gets a gold border + glow and a ★ suffix
	 * is appended to the weight cell.
	 */

	interface Props {
		count?: number;
		reps?: number;
		duration?: string;
		weight?: string;
		intensity?: number;
		bodyweight?: boolean;
		pr?: boolean;
		size?: 'sm' | 'xs';
	}

	let {
		count,
		reps,
		duration,
		weight,
		intensity = 0.6,
		bodyweight = false,
		pr = false,
		size = 'sm'
	}: Props = $props();

	let pct = $derived(`${Math.round(intensity * 100)}%`);
</script>

<span class="spill s-{size}" class:pr style="--w: {pct};">
	{#if count !== undefined && count > 1}<span class="count">{count}×</span>{/if}
	{#if reps !== undefined && reps > 0}<span class="reps">{reps}×</span>{/if}
	{#if duration}<span class="duration">{duration}</span>{/if}
	{#if weight}<span class="weight" class:bw={bodyweight}>{weight}</span>{/if}
</span>

<style>
	.spill {
		display: inline-flex;
		align-items: stretch;
		overflow: hidden;
		border-radius: 999px;
		border: 1px solid color-mix(in oklab, var(--ink) 10%, transparent);
		box-shadow: 0 4px 10px -4px var(--shadow-soft);
		font:
			800 12px/1 'Onest',
			system-ui,
			sans-serif;
		font-variant-numeric: tabular-nums;
		vertical-align: middle;
	}
	.spill > :global(span) {
		padding: 7px 10px;
		display: inline-flex;
		align-items: center;
		border-left: 1px solid color-mix(in oklab, var(--ink) 10%, transparent);
		white-space: nowrap;
	}
	.spill > :global(span:first-child) {
		border-left: 0;
	}

	.count {
		background: var(--gold);
		color: white;
		padding: 7px 11px !important;
	}
	.reps {
		background: var(--bg-2);
		color: var(--ink-2);
	}
	.duration {
		background: color-mix(in oklab, var(--clay) 22%, var(--card));
		color: var(--clay-text);
	}
	.weight {
		background: color-mix(in oklab, var(--clay) var(--w, 60%), var(--bg-2));
		color: var(--ink);
		font-weight: 800;
	}
	.weight.bw {
		background: color-mix(in oklab, var(--ink) 14%, var(--bg-2));
	}

	.spill.pr {
		border-color: var(--gold);
		box-shadow: 0 6px 14px -4px color-mix(in oklab, var(--gold) 55%, transparent);
	}
	.spill.pr .weight::after {
		content: '★';
		display: inline-grid;
		place-items: center;
		width: 14px;
		height: 14px;
		border-radius: 999px;
		background: var(--surface-deep);
		color: var(--gold);
		font-size: 9px;
		line-height: 1;
		margin-left: 5px;
		box-shadow: 0 0 0 1px color-mix(in oklab, var(--gold) 38%, transparent);
	}

	.s-xs {
		font-size: 10.5px;
		box-shadow: 0 2px 6px -2px var(--shadow-soft);
	}
	.s-xs > :global(span) {
		padding: 5px 8px;
	}
	.s-xs .count {
		padding: 5px 9px !important;
	}
</style>
