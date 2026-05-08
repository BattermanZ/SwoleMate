<script lang="ts">
	import { createEventDispatcher, onDestroy } from 'svelte';
	import type { UiExercise } from '$lib/today/types';
	import { calculateExerciseVolumeKg } from '$lib/today/controller/metrics';
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
	let timerTargetSeconds = 0;
	let timerRemainingSeconds = 0;
	let timerEndsAt = 0;
	let timerInterval: number | undefined;

	$: notesDraft = exercise.notes;
	$: locked = disabled || (exercise.status === 'done' && !editing);
	$: if (exercise.status !== 'done') editing = false;
	$: tracksReps = exercise.tracksReps ?? true;
	$: tracksTime = exercise.tracksTime ?? false;
	$: tracksWeight = exercise.tracksWeight ?? true;
	$: tracksRepsForSet = tracksTime ? false : tracksReps;
	$: timerOverlayOpen = tracksTime && timerTargetSeconds > 0;
	$: timerComplete = timerTargetSeconds > 0 && timerRemainingSeconds <= 0 && !timerRunning;
	$: timerDisplaySeconds = timerTargetSeconds > 0 ? timerRemainingSeconds : setDurationSeconds;
	$: timerElapsedSeconds =
		timerTargetSeconds > 0
			? Math.max(0, timerTargetSeconds - Math.max(0, timerRemainingSeconds))
			: 0;
	$: timerCanSave = timerComplete || (!timerRunning && timerElapsedSeconds > 0);
	$: timerProgress =
		timerTargetSeconds > 0
			? Math.max(0, Math.min(1, timerRemainingSeconds / timerTargetSeconds))
			: 1;
	$: timerProgressPct = `${Math.round(timerProgress * 100)}%`;
	$: timerTone =
		timerComplete || timerProgress <= 0.15 ? 'steady' : timerProgress <= 0.4 ? 'warning' : 'danger';

	$: if (
		isOpen &&
		!didPrefillFromLast &&
		!locked &&
		exercise.status === 'active' &&
		(exercise.sets.length > 0 || lastTime?.sets?.length)
	) {
		const sourceSets = exercise.sets.length > 0 ? exercise.sets : lastTime?.sets;
		const first = sourceSets?.[exercise.sets.length > 0 ? sourceSets.length - 1 : 0];
		if (first) {
			setReps = first.reps;
			setDurationSeconds = lastUsedDuration(sourceSets ?? []) ?? setDurationSeconds;
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

	function durationForSets(sets: Array<{ durationSeconds?: number }>) {
		return sets.reduce((total, s) => total + (s.durationSeconds ?? 0), 0);
	}

	function setSummaryLabel() {
		const totalVolume = Math.round(calculateExerciseVolumeKg(exercise));
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

	function lastUsedDuration(sets: Array<{ durationSeconds?: number }>) {
		for (let index = sets.length - 1; index >= 0; index -= 1) {
			const durationSeconds = sets[index]?.durationSeconds;
			if (durationSeconds && durationSeconds > 0) return durationSeconds;
		}
		return undefined;
	}

	function addSet() {
		if (locked) return;
		const reps = tracksRepsForSet ? setReps : 0;
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
		timerRemainingSeconds = Math.max(0, Math.ceil((timerEndsAt - Date.now()) / 1000));
		if (timerRemainingSeconds > 0) return;
		timerRunning = false;
		stopTimerInterval();
		setDurationSeconds = timerTargetSeconds;
	}

	function startTimer() {
		if (locked || !tracksTime || timerRunning || typeof window === 'undefined') return;
		const target = timerTargetSeconds || Math.max(1, Math.round(setDurationSeconds));
		const remaining = timerRemainingSeconds > 0 ? timerRemainingSeconds : target;
		timerTargetSeconds = target;
		timerRemainingSeconds = remaining;
		setDurationSeconds = target;
		timerRunning = true;
		timerEndsAt = Date.now() + remaining * 1000;
		tickTimer();
		timerInterval = window.setInterval(tickTimer, 250);
	}

	function pauseTimer() {
		if (!timerRunning) return;
		tickTimer();
		timerRunning = false;
		stopTimerInterval();
	}

	function resetTimer() {
		timerRunning = false;
		timerTargetSeconds = 0;
		timerRemainingSeconds = 0;
		timerEndsAt = 0;
		stopTimerInterval();
	}

	function saveTimedSet() {
		if (timerRunning) pauseTimer();
		if (!timerCanSave) return;
		setDurationSeconds = timerComplete ? timerTargetSeconds : timerElapsedSeconds;
		addSet();
		resetTimer();
	}

	function resetCountdown() {
		const target = timerTargetSeconds || Math.max(1, Math.round(setDurationSeconds));
		timerRunning = false;
		timerTargetSeconds = target;
		timerRemainingSeconds = target;
		timerEndsAt = 0;
		stopTimerInterval();
		setDurationSeconds = target;
	}

	function updateDurationSeconds(value: string) {
		setDurationSeconds = Number(value);
		if (!timerRunning) resetTimer();
	}

	function updateTracking(patch: Partial<{ reps: boolean; time: boolean; weight: boolean }>) {
		if (locked) return;
		const next = {
			reps: patch.reps ?? tracksReps,
			time: patch.time ?? tracksTime,
			weight: patch.weight ?? tracksWeight
		};
		if (next.time) next.reps = false;
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
							class="btn variant-filled-primary w-full sm:w-auto sm:justify-self-auto"
							on:click={addSetting}
							disabled={locked || !newSettingKey.trim() || !newSettingValue.trim()}
						>
							Add
						</button>
					</div>
				</div>
			</section>

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
				<div>
					<h4 class="text-sm font-semibold opacity-80">Current sets</h4>
					<div
						class="mt-2 grid grid-cols-2 gap-x-4 gap-y-2 sm:flex sm:flex-wrap sm:items-center sm:gap-2"
					>
						{#if !tracksTime}
							<div class="flex items-center gap-2 text-xs opacity-80 select-none">
								<input
									type="checkbox"
									class="checkbox"
									aria-label="Track reps"
									checked={tracksReps}
									disabled={locked || tracksReps}
									on:change={(e) => updateTracking({ reps: e.currentTarget.checked })}
								/>
								<span aria-hidden="true">Reps</span>
							</div>
						{/if}
						<div class="flex items-center gap-2 text-xs opacity-80 select-none">
							<input
								type="checkbox"
								class="checkbox"
								aria-label="Track time"
								checked={tracksTime}
								disabled={locked}
								on:change={(e) => updateTracking({ time: e.currentTarget.checked })}
							/>
							<span aria-hidden="true">Time</span>
						</div>
						<div class="flex items-center gap-2 text-xs opacity-80 select-none">
							<input
								type="checkbox"
								class="checkbox"
								aria-label="Track weight"
								checked={tracksWeight}
								disabled={locked}
								on:change={(e) => updateTracking({ weight: e.currentTarget.checked })}
							/>
							<span aria-hidden="true">Weight</span>
						</div>
						{#if tracksWeight}
							<div class="flex items-center gap-2 text-xs opacity-80 select-none">
								<input
									type="checkbox"
									class="checkbox"
									aria-label="Per-side weights"
									checked={exercise.perSideWeight}
									disabled={locked}
									on:change={(e) => togglePerSideWeight(e.currentTarget.checked)}
								/>
								<span aria-hidden="true">Per-side</span>
							</div>
						{/if}
						{#if tracksWeight && exercise.perSideWeight}
							<div class="flex items-center gap-2 text-xs opacity-80 select-none">
								<input
									type="checkbox"
									class="checkbox"
									aria-label="Split left and right weights"
									checked={exercise.splitWeight}
									disabled={locked}
									on:change={(e) => toggleSplitWeight(e.currentTarget.checked)}
								/>
								<span aria-hidden="true">Split L/R</span>
							</div>
						{/if}
						{#if exercise.sets.length > 0}
							<button
								type="button"
								class="btn btn-xs variant-ghost-primary justify-self-start"
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
					{#if tracksRepsForSet}
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
								<span class="text-xs font-semibold opacity-70">Target duration (sec)</span>
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
								<button
									type="button"
									class="btn btn-xs variant-soft"
									on:click={startTimer}
									disabled={locked || timerRunning}
								>
									Start timer
								</button>
								<button
									type="button"
									class="btn btn-xs variant-ghost"
									on:click={resetTimer}
									disabled={locked || !timerOverlayOpen}
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
						on:click={addSet}
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

{#if timerOverlayOpen}
	<div class="timer-overlay" role="dialog" aria-modal="true" aria-label={`${exercise.name} timer`}>
		<div class="timer-panel">
			<div class="timer-heading">
				<div class="timer-kicker">{timerComplete ? 'Timer complete' : 'Timed set'}</div>
				<div class="timer-title">{exercise.name}</div>
			</div>

			<div
				class="timer-dial {!timerRunning ? 'timer-dial--paused' : ''}"
				data-tone={timerTone}
				style={`--timer-progress:${timerProgressPct}`}
			>
				<div class="timer-dial__inner">
					<div class="timer-state">
						{timerComplete ? 'Complete' : timerRunning ? 'Running' : 'Paused'}
					</div>
					<div class="timer-value">{formatDuration(timerDisplaySeconds)}</div>
					<div class="timer-caption">
						Target {formatDuration(timerTargetSeconds)} • Set {exercise.sets.length +
							1}{tracksWeight ? ' • weight preserved' : ''}
					</div>
				</div>
			</div>

			<div class="timer-actions">
				{#if timerRunning}
					<button type="button" class="btn variant-soft" on:click={pauseTimer}>Pause</button>
				{:else if timerComplete}
					<button type="button" class="btn variant-soft" on:click={resetCountdown}>Repeat</button>
				{:else}
					<button type="button" class="btn variant-soft" on:click={startTimer}>Resume</button>
				{/if}
				<button
					type="button"
					class="btn variant-filled-primary"
					on:click={saveTimedSet}
					disabled={!timerCanSave}
				>
					Add set
				</button>
				<button type="button" class="btn variant-ghost" on:click={resetCountdown}>Reset</button>
				<button type="button" class="btn variant-ghost" on:click={resetTimer}>Close</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.timer-overlay {
		position: fixed;
		inset: 0;
		z-index: 80;
		display: grid;
		place-items: center;
		padding: 1.25rem;
		background:
			radial-gradient(circle at center, rgb(255 255 255 / 0.08), transparent 18rem),
			rgb(2 6 23 / 0.62);
		backdrop-filter: blur(10px);
	}

	.timer-panel {
		width: min(100%, 24rem);
		display: grid;
		justify-items: center;
		gap: 1rem;
		color: white;
		text-align: center;
	}

	.timer-heading {
		display: grid;
		gap: 0.25rem;
	}

	.timer-kicker {
		font-size: 0.72rem;
		font-weight: 800;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		opacity: 0.72;
	}

	.timer-title {
		font-size: 1rem;
		font-weight: 800;
	}

	.timer-dial {
		width: min(76vw, 18rem);
		aspect-ratio: 1;
		border-radius: 9999px;
		display: grid;
		place-items: center;
		padding: 0.9rem;
		--timer-ring-color: var(--color-success-400);
		--timer-track-color: rgb(51 65 85);
		background:
			conic-gradient(
				from 0deg,
				var(--timer-ring-color) 0 var(--timer-progress),
				var(--timer-track-color) var(--timer-progress) 100%
			),
			rgb(15 23 42 / 0.92);
		box-shadow:
			0 1.5rem 5rem rgb(0 0 0 / 0.42),
			0 0 0 1px rgb(255 255 255 / 0.14);
		animation: timer-breathe 1.8s ease-in-out infinite;
	}

	.timer-dial--paused {
		background:
			conic-gradient(
				from 0deg,
				var(--timer-ring-color) 0 var(--timer-progress),
				var(--timer-track-color) var(--timer-progress) 100%
			),
			rgb(15 23 42 / 0.92);
		animation: none;
	}

	.timer-dial[data-tone='warning'] {
		--timer-ring-color: var(--color-warning-400);
	}

	.timer-dial[data-tone='danger'] {
		--timer-ring-color: var(--color-error-400);
	}

	.timer-dial__inner {
		width: 100%;
		height: 100%;
		border-radius: inherit;
		display: grid;
		place-items: center;
		align-content: center;
		gap: 0.5rem;
		background: rgb(15 23 42 / 0.94);
		box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.08);
	}

	.timer-state,
	.timer-caption {
		font-size: 0.78rem;
		font-weight: 800;
		opacity: 0.68;
	}

	.timer-value {
		font-size: clamp(3rem, 16vw, 5.25rem);
		font-weight: 900;
		line-height: 1;
		font-variant-numeric: tabular-nums;
		letter-spacing: 0;
	}

	.timer-actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 0.65rem;
	}

	@keyframes timer-breathe {
		0%,
		100% {
			transform: scale(1);
		}
		50% {
			transform: scale(1.025);
		}
	}
</style>
