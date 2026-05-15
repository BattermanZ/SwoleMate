<script lang="ts">
	type Tone = 'gold' | 'sage';

	interface Props {
		exerciseName: string;
		prTagLabel: string; // "All-time PR" or "Recent best"
		prType: string; // "Max weight" / "Estimated 1RM" / etc.
		newValue: string;
		previousValue?: string;
		dateLabel: string;
		details?: string;
		tone?: Tone;
		onclick?: () => void;
	}

	let {
		exerciseName,
		prTagLabel,
		prType,
		newValue,
		previousValue,
		dateLabel,
		details,
		tone = 'gold',
		onclick
	}: Props = $props();
</script>

<svelte:element
	this={onclick ? 'button' : 'article'}
	class="row tone-{tone}"
	type={onclick ? 'button' : undefined}
	onclick={onclick}
	role={onclick ? 'button' : undefined}
	tabindex={onclick ? 0 : undefined}
>
	<div class="top">
		<span class="ex">{exerciseName}</span>
		<span class="date">{dateLabel}</span>
	</div>
	<div class="body">
		<span class="tag">{prTagLabel}</span>
		<span class="ptype">{prType}</span>
		<span class="new">{newValue}</span>
		{#if previousValue}<span class="from">from <b>{previousValue}</b></span>{/if}
	</div>
	{#if details}<div class="details">{details}</div>{/if}
</svelte:element>

<style>
	.row {
		position: relative;
		overflow: hidden;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 14px;
		padding: 12px 14px 12px 18px;
		display: block;
		width: 100%;
		text-align: left;
		font: inherit;
		color: inherit;
		cursor: default;
	}
	button.row {
		cursor: pointer;
	}
	.row::before {
		content: '';
		position: absolute;
		top: 10px;
		bottom: 10px;
		left: 0;
		width: 3px;
		border-radius: 0 3px 3px 0;
	}
	.tone-gold::before {
		background: var(--gold);
	}
	.tone-sage::before {
		background: var(--sage);
	}

	.top {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 10px;
	}
	.ex {
		font: 800 14px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.01em;
	}
	.date {
		flex: none;
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
		border: 1px solid var(--line);
		background: var(--card);
		padding: 4px 9px;
		border-radius: 999px;
	}

	.body {
		margin-top: 6px;
		font: 500 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--ink-2);
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px 6px;
	}
	.tag {
		font: 700 9px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		padding: 4px 8px;
		border-radius: 999px;
		white-space: nowrap;
	}
	.tone-gold .tag {
		background: color-mix(in oklab, var(--gold) 24%, var(--card));
		color: var(--clay-text);
		border: 1px solid color-mix(in oklab, var(--gold) 40%, var(--line));
	}
	.tone-sage .tag {
		background: color-mix(in oklab, var(--sage) 22%, var(--card));
		color: var(--sage);
		border: 1px solid color-mix(in oklab, var(--sage) 40%, var(--line));
	}
	.ptype {
		font-weight: 700;
	}
	.new {
		font: 800 16px/1 'Onest', system-ui, sans-serif;
		color: var(--ink);
		letter-spacing: -0.015em;
		font-variant-numeric: tabular-nums;
	}
	.from {
		color: var(--ink-soft);
		font-style: italic;
		font-family: 'Instrument Serif';
		font-size: 13px;
	}
	.from b {
		font-style: normal;
		font-family: 'Onest', system-ui, sans-serif;
		font-weight: 700;
	}

	.details {
		margin-top: 6px;
		font: 500 11px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
		font-variant-numeric: tabular-nums;
	}
</style>
