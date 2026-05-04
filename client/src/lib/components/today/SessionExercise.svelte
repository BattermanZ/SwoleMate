<script lang="ts">
	import { createEventDispatcher, onDestroy } from 'svelte';
	import type { UiExercise } from '$lib/today/types';
	import SetPillsHybrid from '$lib/components/ui/SetPillsHybrid.svelte';
	import { formatDateShort } from '$lib/utils/date';

	export let exercise: UiExercise;
	export let isOpen = false;
	export let disabled = false;
	export let lastTime:
		| {
				startedAt: string;
				notes: string;
				sets: Array<{
					reps: number;
					weight: number;
					weightLeft?: number;
					weightRight?: number;
					durationSeconds?: number;
				}>;
				perSideWeight: boolean;
				splitWeight: boolean;
		  }
		| undefined = undefined;

	const dispatch = createEventDispatcher<{
		toggle: undefined;
		delete: undefined;
		markDone: undefined;
		addSet: {
			reps: number;
			weight: number;
			weightLeft?: number;
			weightRight?: number;
			durationSeconds?: number;
		};
		updateNotes: { notes: string };
		addSetting: { key: string; value: string };
		removeSetting: { id: string };
		updateSetting: { id: string; key: string; value: string };
		togglePerSideWeight: { enabled: boolean };
		toggleSplitWeight: { enabled: boolean };
		updateTracking: { reps: boolean; time: boolean; weight: boolean };
	}>();

	let setReps = 12;
	let setWeight = 0;
	let setWeightLeft = 0;
	let setWeightRight = 0;
	let setDurationSeconds = 60;
	let notesDraft = '';
	let newSettingKey = '';
	let newSettingValue = '';
	let editing = false;
	let locked = false;
	let didPrefillFromLast = false;
	let timerRunning = false;
	let timerStartedAt = 0;
	let timerBaseSeconds = 0;
	let timerElapsedSeconds = 0;
	let timerInterval: number | undefined;

	$: notesDraft = exercise.notes;
	$: locked = disabled || (exercise.status === 'done' && !editing);
	$: if (exercise.status !== 'done') editing = false;
	$: if (exercise.sets.length > 0) didPrefillFromLast = true;
	$: tracksReps = exercise.tracksReps ?? true;
	$: tracksTime = exercise.tracksTime ?? false;
	$: tracksWeight = exercise.tracksWeight ?? true;

	$: if (
		isOpen &&
		!didPrefillFromLast &&
		!locked &&
		exercise.status === 'active' &&
		lastTime?.sets?.length
	) {
		const first = lastTime.sets[0];
		if (first) {
			setReps = first.reps;
			if (first.durationSeconds) setDurationSeconds = first.durationSeconds;
			if (!exercise.perSideWeight) {
				setWeight = first.weight;
			} else if (!exercise.splitWeight) {
				setWeight = first.weight;
			} else {
				setWeightLeft = first.weightLeft ?? first.weight;
				setWeightRight = first.weightRight ?? first.weight;
			}
		}

		didPrefillFromLast = true;
	}

	function toggleEditing() {
		if (disabled) return;
		if (exercise.status !== 'done') return;
		if (editing) saveNotes();
		editing = !editing;
	}

	function setTotalWeight(
		set: { weight: number; weightLeft?: number; weightRight?: number },
		perSideWeight: boolean,
		splitWeight: boolean
	): number {
		if (!perSideWeight) return set.weight;
		if (!splitWeight) return set.weight * 2;
		const left = set.weightLeft ?? set.weight;
		const right = set.weightRight ?? set.weight;
		return left + right;
	}

	function volumeForSets(
		sets: Array<{ reps: number; weight: number; weightLeft?: number; weightRight?: number }>,
		perSideWeight: boolean,
		splitWeight: boolean
	) {
		return sets.reduce(
			(total, s) => total + s.reps * setTotalWeight(s, perSideWeight, splitWeight),
			0
		);
	}

	function durationForSets(sets: Array<{ durationSeconds?: number }>) {
		return sets.reduce((total, s) => total + (s.durationSeconds ?? 0), 0);
	}

	function setSummaryLabel() {
		const totalVolume = Math.round(
			volumeForSets(exercise.sets, exercise.perSideWeight, exercise.splitWeight)
		);
		if (totalVolume > 0) return `${totalVolume} kg`;
		const totalDuration = durationForSets(exercise.sets);
		if (totalDuration > 0) return formatDuration(totalDuration);
		return '0 kg';
	}

	function useLastSet() {
		const last = exercise.sets[exercise.sets.length - 1];
		if (!last) return;
		setReps = last.reps;
		if (last.durationSeconds) setDurationSeconds = last.durationSeconds;
		if (!exercise.perSideWeight) {
			setWeight = last.weight;
			return;
		}
		if (!exercise.splitWeight) {
			setWeight = last.weight;
			return;
		}
		setWeightLeft = last.weightLeft ?? last.weight;
		setWeightRight = last.weightRight ?? last.weight;
	}

	function addSet() {
		if (locked) return;
		const reps = tracksReps ? setReps : 0;
		const durationSeconds = tracksTime ? Math.round(setDurationSeconds) : undefined;
		if (reps <= 0 && !durationSeconds) return;
		if (durationSeconds !== undefined && durationSeconds <= 0) return;
		if (!tracksWeight) {
			dispatch('addSet', { reps, weight: 0, durationSeconds });
			return;
		}
		if (!exercise.perSideWeight) {
			if (setWeight < 0) return;
			dispatch('addSet', { reps, weight: setWeight, durationSeconds });
			return;
		}
		if (!exercise.splitWeight) {
			if (setWeight < 0) return;
			dispatch('addSet', { reps, weight: setWeight, durationSeconds });
			return;
		}
		if (setWeightLeft < 0 || setWeightRight < 0) return;
		dispatch('addSet', {
			reps,
			weight: (setWeightLeft + setWeightRight) / 2,
			weightLeft: setWeightLeft,
			weightRight: setWeightRight,
			durationSeconds
		});
	}

	function formatDuration(seconds: number): string {
		const value = Math.max(0, Math.round(seconds));
		const minutes = Math.floor(value / 60);
		const remaining = value % 60;
		return `${minutes}:${String(remaining).padStart(2, '0')}`;
	}

	function stopTimerInterval() {
		if (timerInterval !== undefined && typeof window !== 'undefined') {
			window.clearInterval(timerInterval);
		}
		timerInterval = undefined;
	}

	function tickTimer() {
		timerElapsedSeconds =
			timerBaseSeconds + Math.floor(Math.max(0, Date.now() - timerStartedAt) / 1000);
		setDurationSeconds = Math.max(1, timerElapsedSeconds);
	}

	function startTimer() {
		if (locked || !tracksTime || timerRunning || typeof window === 'undefined') return;
		timerRunning = true;
		timerStartedAt = Date.now();
		timerBaseSeconds = timerElapsedSeconds;
		tickTimer();
		timerInterval = window.setInterval(tickTimer, 250);
	}

	function pauseTimer() {
		if (!timerRunning) return;
		tickTimer();
		timerBaseSeconds = timerElapsedSeconds;
		timerRunning = false;
		stopTimerInterval();
	}

	function resetTimer() {
		timerRunning = false;
		timerBaseSeconds = 0;
		timerElapsedSeconds = 0;
		stopTimerInterval();
		setDurationSeconds = 60;
	}

	function saveTimedSet() {
		if (timerRunning) pauseTimer();
		if (timerElapsedSeconds > 0) setDurationSeconds = timerElapsedSeconds;
		addSet();
		resetTimer();
	}

	function updateDurationSeconds(value: string) {
		setDurationSeconds = Number(value);
		if (!timerRunning && timerElapsedSeconds > 0) {
			timerElapsedSeconds = 0;
			timerBaseSeconds = 0;
		}
	}

	function updateTracking(patch: Partial<{ reps: boolean; time: boolean; weight: boolean }>) {
		if (locked) return;
		const next = {
			reps: patch.reps ?? tracksReps,
			time: patch.time ?? tracksTime,
			weight: patch.weight ?? tracksWeight
		};
		if (!next.reps && !next.time) next.reps = true;
		dispatch('updateTracking', next);
	}

	onDestroy(stopTimerInterval);

	function saveNotes() {
		if (locked) return;
		const next = notesDraft.trim();
		if (next === exercise.notes) return;
		dispatch('updateNotes', { notes: next });
	}

	function addSetting() {
		if (locked) return;
		const key = newSettingKey.trim();
		const value = newSettingValue.trim();
		if (!key || !value) return;
		dispatch('addSetting', { key, value });
		newSettingKey = '';
		newSettingValue = '';
	}

	function togglePerSideWeight(enabled: boolean) {
		if (locked) return;
		if (enabled === exercise.perSideWeight) return;
		if (!enabled) {
			setWeight = exercise.splitWeight ? setWeightLeft + setWeightRight : setWeight * 2;
			setWeightLeft = 0;
			setWeightRight = 0;
			dispatch('togglePerSideWeight', { enabled: false });
			dispatch('toggleSplitWeight', { enabled: false });
			return;
		}

		setWeightLeft = setWeight;
		setWeightRight = setWeight;
		dispatch('togglePerSideWeight', { enabled: true });
	}

	function toggleSplitWeight(enabled: boolean) {
		if (locked) return;
		if (enabled === exercise.splitWeight) return;
		if (!exercise.perSideWeight) return;
		if (enabled) {
			setWeightLeft = setWeight;
			setWeightRight = setWeight;
		} else {
			setWeight = Math.max(setWeightLeft, setWeightRight);
		}
		dispatch('toggleSplitWeight', { enabled });
	}
</script>

<article
	class="card variant-glass-surface p-4 border border-surface-200/60 dark:border-surface-700/50"
>
	<header class="flex items-start justify-between gap-4">
		<button
			type="button"
			class="min-w-0 flex-1 text-left"
			on:click={() => dispatch('toggle')}
			aria-expanded={isOpen}
		>
			<div class="flex items-center gap-2">
				<h3 class="text-lg font-semibold tracking-tight truncate">{exercise.name}</h3>
				{#if exercise.status === 'done'}
					<span class="badge variant-filled-success text-xs">Done</span>
				{:else}
					<span class="badge variant-soft text-xs">In progress</span>
				{/if}
				<span class="ml-auto opacity-60">{isOpen ? '▾' : '▸'}</span>
			</div>
			<div class="mt-1 flex flex-wrap gap-2 text-sm opacity-80">
				<span>{exercise.sets.length} set{exercise.sets.length === 1 ? '' : 's'}</span>
				<span class="opacity-50">•</span>
				<span>{setSummaryLabel()}</span>
			</div>
		</button>

		<div class="flex items-center gap-2 shrink-0 mt-0.5">
			{#if exercise.status === 'done'}
				<button
					type="button"
					class="badge text-xs {editing
						? 'variant-filled-tertiary'
						: 'variant-soft'} cursor-pointer select-none transition disabled:opacity-50 disabled:cursor-not-allowed"
					on:click={toggleEditing}
					{disabled}
					aria-label={editing ? 'Stop editing exercise' : 'Edit exercise'}
				>
					{editing ? 'Done' : 'Edit'}
				</button>
			{/if}
			<button
				type="button"
				class="btn btn-xs variant-soft-error"
				on:click={() => dispatch('delete')}
				{disabled}
				aria-label="Remove exercise"
			>
				✕
			</button>
		</div>
	</header>

	{#if isOpen}
		<div class="mt-4 space-y-4">
			{#if lastTime}
				<div
					class="rounded-xl border border-surface-200/50 bg-surface-50/70 p-3 text-sm dark:border-surface-700/50 dark:bg-surface-950/30"
				>
					<div class="flex flex-wrap items-center gap-2">
						<span class="font-semibold opacity-80">Last time</span>
						<span class="opacity-60">({formatDateShort(lastTime.startedAt)})</span>
						<SetPillsHybrid
							sets={lastTime.sets}
							perSideWeight={lastTime.perSideWeight}
							splitWeight={lastTime.splitWeight}
							size="xs"
						/>
					</div>
					{#if lastTime.notes}
						<p class="mt-2 opacity-80">Notes: {lastTime.notes}</p>
					{/if}
				</div>
			{/if}

			<section class="space-y-2">
				<div class="flex items-center justify-between">
					<h4 class="text-sm font-semibold opacity-80">Settings</h4>
					<p class="text-xs opacity-60">Optional equipment setup</p>
				</div>

				{#if exercise.settings.length === 0}
					<p class="text-sm opacity-70">
						Add things like bench angle, seat height, handle position, pins, or cues.
					</p>
				{/if}

				<div class="space-y-2">
					{#each exercise.settings as s (s.id)}
						<div class="grid grid-cols-1 sm:grid-cols-[1fr_1fr_auto] gap-2">
							<input
								class="input min-w-0"
								value={s.key}
								placeholder="Setting"
								disabled={locked}
								on:input={(e) =>
									dispatch('updateSetting', {
										id: s.id,
										key: e.currentTarget.value,
										value: s.value
									})}
							/>
							<input
								class="input min-w-0"
								value={s.value}
								placeholder="Value"
								disabled={locked}
								on:input={(e) =>
									dispatch('updateSetting', {
										id: s.id,
										key: s.key,
										value: e.currentTarget.value
									})}
							/>
							<button
								type="button"
								class="btn variant-soft-error justify-self-end w-10"
								on:click={() => dispatch('removeSetting', { id: s.id })}
								disabled={locked}
								aria-label="Remove setting"
							>
								✕
							</button>
						</div>
					{/each}

					<div class="grid grid-cols-1 sm:grid-cols-[1fr_1fr_auto] gap-2">
						<input
							class="input min-w-0"
							bind:value={newSettingKey}
							placeholder="e.g. Bench angle"
							disabled={locked}
							on:keydown={(e) => e.key === 'Enter' && addSetting()}
						/>
						<input
							class="input min-w-0"
							bind:value={newSettingValue}
							placeholder="e.g. 30°"
							disabled={locked}
							on:keydown={(e) => e.key === 'Enter' && addSetting()}
						/>
						<button
							type="button"
							class="btn variant-filled-primary justify-self-end sm:justify-self-auto"
							on:click={addSetting}
							disabled={locked || !newSettingKey.trim() || !newSettingValue.trim()}
						>
							Add
						</button>
					</div>
				</div>
			</section>

			<section class="space-y-2">
				<div class="flex items-center justify-between">
					<h4 class="text-sm font-semibold opacity-80">Sets</h4>
					<div class="flex flex-wrap items-center gap-2">
						<label class="flex items-center gap-2 text-xs opacity-80 select-none">
							<input
								type="checkbox"
								class="checkbox"
								checked={tracksReps}
								disabled={locked || (!tracksTime && tracksReps)}
								on:change={(e) => updateTracking({ reps: e.currentTarget.checked })}
							/>
							Track reps
						</label>
						<label class="flex items-center gap-2 text-xs opacity-80 select-none">
							<input
								type="checkbox"
								class="checkbox"
								checked={tracksTime}
								disabled={locked || (!tracksReps && tracksTime)}
								on:change={(e) => updateTracking({ time: e.currentTarget.checked })}
							/>
							Track time
						</label>
						<label class="flex items-center gap-2 text-xs opacity-80 select-none">
							<input
								type="checkbox"
								class="checkbox"
								checked={tracksWeight}
								disabled={locked}
								on:change={(e) => updateTracking({ weight: e.currentTarget.checked })}
							/>
							Track weight
						</label>
						{#if tracksWeight}
							<label class="flex items-center gap-2 text-xs opacity-80 select-none">
								<input
									type="checkbox"
									class="checkbox"
									checked={exercise.perSideWeight}
									disabled={locked}
									on:change={(e) => togglePerSideWeight(e.currentTarget.checked)}
								/>
								Per-side weights
							</label>
						{/if}
						{#if tracksWeight && exercise.perSideWeight}
							<label class="flex items-center gap-2 text-xs opacity-80 select-none">
								<input
									type="checkbox"
									class="checkbox"
									checked={exercise.splitWeight}
									disabled={locked}
									on:change={(e) => toggleSplitWeight(e.currentTarget.checked)}
								/>
								Different L/R
							</label>
						{/if}
						{#if exercise.sets.length > 0}
							<button
								type="button"
								class="btn btn-xs variant-ghost-primary"
								on:click={useLastSet}
								disabled={locked}
							>
								Use last
							</button>
						{/if}
					</div>
				</div>

				{#if exercise.sets.length === 0}
					<p class="text-sm opacity-70">Add your first set for this exercise.</p>
				{:else}
					<SetPillsHybrid
						sets={exercise.sets}
						perSideWeight={exercise.perSideWeight}
						splitWeight={exercise.splitWeight}
						size="sm"
					/>
				{/if}

				<div class="grid grid-cols-1 sm:grid-cols-[1fr_1fr_auto] gap-2 items-end">
					{#if tracksReps}
						<label class="block">
							<span class="text-xs font-semibold opacity-70">Reps</span>
							<input
								type="number"
								min="0"
								inputmode="numeric"
								pattern="[0-9]*"
								class="input w-full min-w-0"
								bind:value={setReps}
								disabled={locked}
							/>
						</label>
					{/if}
					{#if tracksTime}
						<div class="block">
							<label class="block">
								<span class="text-xs font-semibold opacity-70">Duration (sec)</span>
								<input
									type="number"
									min="1"
									inputmode="numeric"
									pattern="[0-9]*"
									class="input w-full min-w-0"
									value={setDurationSeconds}
									on:input={(e) => updateDurationSeconds(e.currentTarget.value)}
									disabled={locked || timerRunning}
								/>
							</label>
							<div class="mt-2 flex flex-wrap items-center gap-2">
								<span class="text-sm font-bold tabular-nums"
									>{formatDuration(timerElapsedSeconds)}</span
								>
								{#if timerRunning}
									<button type="button" class="btn btn-xs variant-soft" on:click={pauseTimer}>
										Pause
									</button>
								{:else}
									<button
										type="button"
										class="btn btn-xs variant-soft"
										on:click={startTimer}
										disabled={locked}
									>
										Start
									</button>
								{/if}
								<button
									type="button"
									class="btn btn-xs variant-ghost"
									on:click={resetTimer}
									disabled={locked || (!timerElapsedSeconds && !timerRunning)}
								>
									Reset
								</button>
							</div>
						</div>
					{/if}
					{#if tracksWeight && !exercise.perSideWeight}
						<label class="block">
							<span class="text-xs font-semibold opacity-70">Weight (kg)</span>
							<input
								type="number"
								min="0"
								step="0.5"
								inputmode="decimal"
								class="input w-full min-w-0"
								bind:value={setWeight}
								disabled={locked}
							/>
						</label>
					{:else if tracksWeight && !exercise.splitWeight}
						<label class="block">
							<span class="text-xs font-semibold opacity-70">Per side (kg)</span>
							<input
								type="number"
								min="0"
								step="0.5"
								inputmode="decimal"
								class="input w-full min-w-0"
								bind:value={setWeight}
								disabled={locked}
							/>
						</label>
					{:else if tracksWeight}
						<div class="grid grid-cols-2 gap-2 min-w-0">
							<label class="block">
								<span class="text-xs font-semibold opacity-70">Left (kg)</span>
								<input
									type="number"
									min="0"
									step="0.5"
									inputmode="decimal"
									class="input w-full min-w-0"
									bind:value={setWeightLeft}
									disabled={locked}
								/>
							</label>
							<label class="block">
								<span class="text-xs font-semibold opacity-70">Right (kg)</span>
								<input
									type="number"
									min="0"
									step="0.5"
									inputmode="decimal"
									class="input w-full min-w-0"
									bind:value={setWeightRight}
									disabled={locked}
								/>
							</label>
						</div>
					{/if}
					<button
						type="button"
						class="btn variant-filled-primary w-full sm:w-auto"
						on:click={timerElapsedSeconds > 0 ? saveTimedSet : addSet}
						disabled={locked}
					>
						Add set
					</button>
				</div>
			</section>

			<section class="space-y-2">
				<h4 class="text-sm font-semibold opacity-80">Notes</h4>
				<textarea
					class="textarea"
					rows="2"
					bind:value={notesDraft}
					placeholder="Cues, tempo, how it felt…"
					disabled={locked}
					on:blur={saveNotes}
				></textarea>
			</section>

			<footer class="flex flex-wrap gap-2 justify-end">
				<button
					type="button"
					class="btn variant-soft"
					on:click={() => dispatch('toggle')}
					{disabled}
				>
					Collapse
				</button>
				<button
					type="button"
					class="btn variant-filled-success"
					on:click={() => dispatch('markDone')}
					disabled={disabled || exercise.status === 'done'}
				>
					Mark done
				</button>
			</footer>
		</div>
	{/if}
</article>
