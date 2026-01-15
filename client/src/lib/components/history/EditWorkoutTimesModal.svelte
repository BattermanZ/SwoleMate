<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { FeedbackEmoji, Workout } from '$lib/types';
	import { formatDateShort, formatTime } from '$lib/utils/date';

	export let open = false;
	export let workout: Workout | null = null;
	export let disabled = false;
	export let error: string | null = null;

	const dispatch = createEventDispatcher<{
		cancel: undefined;
		submit: {
			start_time: string;
			end_time: string;
			notes: string | null;
			feedback: FeedbackEmoji | null;
		};
	}>();

	let startLocal = '';
	let endLocal = '';
	let notes = '';
	let feedback: FeedbackEmoji | '' = '';
	let localError: string | null = null;

	function toDateTimeLocal(iso: string): string {
		const d = new Date(iso);
		if (!Number.isFinite(d.getTime())) return '';
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
	}

	function localToIso(value: string): string | null {
		const d = new Date(value);
		if (!Number.isFinite(d.getTime())) return null;
		return d.toISOString();
	}

	$: if (open && workout) {
		startLocal = toDateTimeLocal(workout.start_time);
		endLocal = toDateTimeLocal(workout.end_time);
		notes = workout.notes ?? '';
		feedback = workout.feedback ?? '';
		localError = null;
	}

	$: {
		if (!open) {
			localError = null;
		} else {
			const start = startLocal ? new Date(startLocal).getTime() : NaN;
			const end = endLocal ? new Date(endLocal).getTime() : NaN;
			if (!Number.isFinite(start) || !Number.isFinite(end)) {
				localError = 'Start and end time are required.';
			} else if (end < start) {
				localError = 'End time must be after start time.';
			} else {
				localError = null;
			}
		}
	}

	function submit() {
		if (!workout) return;
		if (localError) return;
		const startIso = localToIso(startLocal);
		const endIso = localToIso(endLocal);
		if (!startIso || !endIso) {
			localError = 'Invalid date/time.';
			return;
		}
		dispatch('submit', {
			start_time: startIso,
			end_time: endIso,
			notes: notes.trim() ? notes : null,
			feedback: feedback ? feedback : null
		});
	}
</script>

{#if open && workout}
	<div class="modal-backdrop fixed inset-0 z-50 bg-black/50 flex items-center justify-center">
		<div class="card variant-filled-surface p-4 w-full max-w-lg mx-4 space-y-4">
			<header class="text-center space-y-1">
				<h3 class="h3">Edit session times</h3>
				<p class="text-sm opacity-70">
					{formatDateShort(workout.start_time)} • {formatTime(workout.start_time)}–{formatTime(
						workout.end_time
					)}
				</p>
			</header>

			<div class="grid gap-3 sm:grid-cols-2">
				<label class="block">
					<span class="text-sm font-semibold opacity-80">Start</span>
					<input
						type="datetime-local"
						class="input mt-1 w-full"
						bind:value={startLocal}
						{disabled}
					/>
				</label>
				<label class="block">
					<span class="text-sm font-semibold opacity-80">End</span>
					<input type="datetime-local" class="input mt-1 w-full" bind:value={endLocal} {disabled} />
				</label>
			</div>

			<div class="grid gap-3 sm:grid-cols-2">
				<label class="block">
					<span class="text-sm font-semibold opacity-80">Mood</span>
					<select class="select mt-1 w-full" bind:value={feedback} {disabled}>
						<option value="">—</option>
						<option value="😊">😊 Good</option>
						<option value="😐">😐 Neutral</option>
						<option value="😞">😞 Bad</option>
					</select>
				</label>

				<label class="block sm:col-span-2">
					<span class="text-sm font-semibold opacity-80">Notes</span>
					<textarea
						class="textarea mt-1 w-full min-h-[96px]"
						placeholder="Optional notes…"
						bind:value={notes}
						{disabled}
					></textarea>
				</label>
			</div>

			{#if error || localError}
				<div class="alert variant-filled-error">{error ?? localError}</div>
			{/if}

			<footer class="flex justify-end gap-2">
				<button
					type="button"
					class="btn variant-soft"
					on:click={() => dispatch('cancel')}
					{disabled}
				>
					Cancel
				</button>
				<button
					type="button"
					class="btn variant-filled-primary"
					on:click={submit}
					disabled={disabled || !!localError}
				>
					Save
				</button>
			</footer>
		</div>
	</div>
{/if}
