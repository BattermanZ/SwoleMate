<script lang="ts">
	import { Btn } from '$lib/components/ui';
	import type { Snippet } from 'svelte';

	type Tone = 'info' | 'success' | 'warn' | 'error';
	interface Props {
		tone?: Tone;
		message?: string;
		children?: Snippet;
		action?: { label: string; onclick: () => void; disabled?: boolean };
	}
	let { tone = 'info', message, children, action }: Props = $props();
</script>

<div class="notice t-{tone}">
	<div class="top">
		<span class="ico" aria-hidden="true">
			{#if tone === 'error'}
				!
			{:else if tone === 'warn'}
				⚠
			{:else if tone === 'success'}
				✓
			{:else}
				↻
			{/if}
		</span>
		<span class="msg">
			{#if message}{message}{/if}
			{@render children?.()}
		</span>
	</div>
	{#if action}
		<Btn variant="ink" onclick={action.onclick} disabled={action.disabled}>
			{action.label}
		</Btn>
	{/if}
</div>

<style>
	.notice {
		padding: 12px 14px;
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 14px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		box-shadow: 0 4px 12px -8px var(--shadow-card);
	}
	.top {
		display: flex;
		align-items: center;
		gap: 10px;
		font:
			500 13px/1.35 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-2);
	}
	.ico {
		flex: none;
		width: 22px;
		height: 22px;
		border-radius: 7px;
		display: grid;
		place-items: center;
		font:
			800 13px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.t-info .ico,
	.t-success .ico {
		background: color-mix(in oklab, var(--sage) 18%, transparent);
		color: var(--sage);
	}
	.t-warn .ico {
		background: color-mix(in oklab, var(--warn) 20%, transparent);
		color: var(--warn);
	}
	.t-error .ico {
		background: color-mix(in oklab, var(--clay) 18%, transparent);
		color: var(--clay-text);
	}
	.t-error {
		border-color: color-mix(in oklab, var(--clay) 30%, var(--line));
	}
</style>
