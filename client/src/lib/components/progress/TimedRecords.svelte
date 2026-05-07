<script lang="ts">
	import type { TimedRecords } from '$lib/types';
	import { formatDuration } from '$lib/progress/format';

	export let records: TimedRecords;
</script>

<div class="grid grid-cols-1 sm:grid-cols-4 gap-3">
	<div class="card variant-glass-surface p-3 border-l-4 border-success-500/60">
		<div class="text-xs font-semibold opacity-70">Longest timed set</div>
		<div class="text-lg font-bold">{formatDuration(records.longest_set_seconds)}</div>
	</div>
	<div class="card variant-glass-surface p-3 border-l-4 border-warning-500/60">
		<div class="text-xs font-semibold opacity-70">Best timed session</div>
		<div class="text-lg font-bold">{formatDuration(records.best_session_duration_seconds)}</div>
	</div>
	<div class="card variant-glass-surface p-3 border-l-4 border-secondary-500/60">
		<div class="text-xs font-semibold opacity-70">Lifetime timed work</div>
		<div class="text-lg font-bold">{formatDuration(records.lifetime_duration_seconds)}</div>
	</div>
	<div class="card variant-glass-surface p-3 border-l-4 border-tertiary-500/60">
		<div class="text-xs font-semibold opacity-70">Average timed set</div>
		<div class="text-lg font-bold">{formatDuration(records.average_set_duration_seconds)}</div>
	</div>
</div>
