<script lang="ts">
	interface Props {
		exerciseName: string;
		summary: string;
		delta?: string;
		onclick?: () => void;
	}
	let { exerciseName, summary, delta, onclick }: Props = $props();
</script>

<button class="pr-flash" type="button" {onclick}>
	<span class="icn" aria-hidden="true">⚡</span>
	<span class="body">
		<span class="k">New personal record</span>
		<span class="v"
			>{exerciseName} <b>{summary}</b>{#if delta}
				— {delta}{/if}</span
		>
	</span>
	<span class="arr" aria-hidden="true">→</span>
</button>

<style>
	.pr-flash {
		padding: 14px 16px;
		border-radius: 18px;
		border: 0;
		background: linear-gradient(95deg, var(--clay) 0%, var(--clay-2) 60%, var(--gold) 100%);
		color: #fff;
		display: grid;
		grid-template-columns: 46px 1fr auto;
		align-items: center;
		gap: 14px;
		box-shadow: 0 14px 32px -10px rgba(255, 94, 31, 0.5);
		position: relative;
		overflow: hidden;
		cursor: pointer;
		width: 100%;
		text-align: left;
		font: inherit;
	}
	.pr-flash::after {
		content: '';
		position: absolute;
		inset: -50% -20% auto auto;
		width: 220px;
		height: 220px;
		background: radial-gradient(circle, rgba(255, 255, 255, 0.35), transparent 60%);
		pointer-events: none;
	}
	.icn {
		width: 46px;
		height: 46px;
		border-radius: 14px;
		background: rgba(255, 255, 255, 0.18);
		display: grid;
		place-items: center;
		font:
			800 22px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.body {
		display: block;
		min-width: 0;
	}
	.k {
		display: block;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.85);
	}
	.v {
		display: block;
		font: italic 400 18px/1.2 'Instrument Serif';
		margin-top: 4px;
	}
	.v b {
		font-style: normal;
		font-family: 'Onest', system-ui, sans-serif;
		font-weight: 800;
	}
	.arr {
		font:
			800 22px/1 'Onest',
			system-ui,
			sans-serif;
	}
</style>
