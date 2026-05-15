<script lang="ts">
	import { SetPillList } from '$lib/components/ui';
	import { formatDateShort } from '$lib/utils/date';
	import type { SetLike } from '$lib/today/setPills';

	interface Props {
		startedAt: string;
		notes?: string;
		sets: SetLike[];
		perSideWeight?: boolean;
		splitWeight?: boolean;
	}
	let { startedAt, notes, sets, perSideWeight = false, splitWeight = false }: Props = $props();
</script>

<div class="last-time">
	<div class="head">
		<b>{sets.length} sets</b>
		<span>logged</span>
		<span class="date">— {formatDateShort(startedAt)}</span>
	</div>
	<div class="pills">
		<SetPillList {sets} {perSideWeight} {splitWeight} size="xs" />
	</div>
	{#if notes}<p class="notes">Notes: {notes}</p>{/if}
</div>

<style>
	.last-time {
		padding: 12px;
		border-radius: 14px;
		background: var(--card-3);
		border: 1px solid var(--line);
	}
	.head {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
		font: 500 12px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-2);
	}
	.head b {
		font-weight: 800;
	}
	.head .date {
		color: var(--ink-soft);
		font-style: italic;
		font-family: 'Instrument Serif';
		font-size: 13px;
	}
	.pills {
		margin-top: 8px;
	}
	.notes {
		margin: 6px 0 0;
		font: italic 400 13px/1.4 'Instrument Serif';
		color: var(--ink-2);
	}
</style>
