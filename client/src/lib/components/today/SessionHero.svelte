<script lang="ts">
	import { PageHero } from '$lib/components/ui';

	interface Props {
		elapsedLabel: string;
		exerciseCount: number;
		exercisesPlanned?: number;
		setCount: number;
		volumeKg: number;
		durationSeconds: number;
		prCount?: number;
		startedAtLabel?: string;
		onCancel?: () => void;
		onEnd?: () => void;
		disabled?: boolean;
	}
	let {
		elapsedLabel,
		exerciseCount,
		exercisesPlanned,
		setCount,
		volumeKg,
		durationSeconds,
		prCount = 0,
		startedAtLabel,
		onCancel,
		onEnd,
		disabled = false
	}: Props = $props();

	function formatTime(seconds: number): string {
		const total = Math.max(0, Math.round(seconds));
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${String(s).padStart(2, '0')}`;
	}

	// progress ring circumference for r=68: 2*pi*68 ≈ 427
	const RING_CIRC = 427;
	const elapsedMin = $derived(() => {
		const [m, s] = elapsedLabel.split(':').map(Number);
		return (m ?? 0) + (s ?? 0) / 60;
	});
	// Use 60min as a soft visual "session length"; ring fills as session goes
	let ringFill = $derived(() => Math.min(1, Math.max(0.08, elapsedMin() / 60)));
	let dashOffset = $derived(() => RING_CIRC * (1 - ringFill()));
</script>

<PageHero kicker={`► Session live · ${elapsedLabel} in`}>
	{#snippet title()}Push day, <em>going off.</em>{/snippet}
	{#snippet sub()}
		{#if startedAtLabel}Started {startedAtLabel}{/if}
	{/snippet}

	<div class="timer-block">
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
					stroke="url(#hg)"
					stroke-width="10"
					fill="none"
					stroke-dasharray={RING_CIRC}
					stroke-dashoffset={dashOffset()}
					stroke-linecap="round"
					transform="rotate(-90 75 75)"
				/>
				<defs>
					<linearGradient id="hg" x1="0" y1="0" x2="1" y2="1">
						<stop offset="0%" stop-color="#ff7a2a" />
						<stop offset="100%" stop-color="#ff5e1f" />
					</linearGradient>
				</defs>
			</svg>
			<div class="center"><span class="big">{elapsedLabel}</span></div>
		</div>

		<div class="stats-wrap">
			<div class="stat-grid">
				<div class="cell">
					<div class="k">Exercises</div>
					<div class="v">
						{exerciseCount}{#if exercisesPlanned}<small>/ {exercisesPlanned}</small>{/if}
					</div>
				</div>
				<div class="cell">
					<div class="k">Sets done</div>
					<div class="v">{setCount}</div>
				</div>
				<div class="cell">
					<div class="k">{volumeKg > 0 ? 'Volume' : 'Time'}</div>
					<div class="v">
						{#if volumeKg > 0}
							{Math.round(volumeKg).toLocaleString()}<small>kg</small>
						{:else}
							{formatTime(durationSeconds)}
						{/if}
					</div>
				</div>
				<div class="cell">
					<div class="k">Records</div>
					<div class="v" style="color: var(--clay-2)">
						{prCount}{#if prCount > 0}<small>★</small>{/if}
					</div>
				</div>
			</div>
		</div>
	</div>

	{#snippet actions()}
		<button class="cancel" {disabled} onclick={onCancel}>Cancel</button>
		<button class="end" {disabled} onclick={onEnd}>End session →</button>
	{/snippet}
</PageHero>

<style>
	.timer-block {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		position: relative;
	}
	.stats-wrap {
		width: 100%;
		min-width: 0;
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
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.big {
		font: 800 32px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.04em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
		transform: translateY(-1px);
	}
	.stat-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 8px;
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
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--on-deep-soft);
	}
	.v {
		font: 800 22px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.025em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
		display: flex;
		align-items: baseline;
		gap: 4px;
	}
	.v small {
		font: 500 11px/1 'Onest', system-ui, sans-serif;
		color: var(--on-deep-soft);
		font-weight: 600;
	}

	.cancel,
	.end {
		flex: 1;
		border: 0;
		padding: 14px;
		border-radius: 14px;
		font: 700 13px/1 'Onest', system-ui, sans-serif;
		cursor: pointer;
	}
	.cancel {
		border: 1px solid var(--on-deep-line);
		background: color-mix(in oklab, var(--on-deep) 6%, transparent);
		color: var(--on-deep);
		flex: 0 1 35%;
	}
	.end {
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		color: white;
		font-weight: 800;
		font-size: 14px;
		box-shadow:
			0 10px 22px -8px rgba(255, 94, 31, 0.55),
			inset 0 -3px 0 var(--clay-deep);
		flex: 1 1 60%;
	}
	.cancel:disabled,
	.end:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
