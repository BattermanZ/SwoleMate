<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { UiExercise } from '$lib/mocks/today';

	export let exercise: UiExercise;
	export let isOpen = false;
	export let disabled = false;
	export let lastTime:
		| {
				startedAt: string;
				notes: string;
				sets: Array<{ reps: number; weight: number; weightLeft?: number; weightRight?: number }>;
				perSideWeight: boolean;
				splitWeight: boolean;
		  }
		| undefined = undefined;

	const dispatch = createEventDispatcher<{
		toggle: undefined;
		delete: undefined;
		markDone: undefined;
		addSet: { reps: number; weight: number; weightLeft?: number; weightRight?: number };
		updateNotes: { notes: string };
		addSetting: { key: string; value: string };
		removeSetting: { id: string };
		updateSetting: { id: string; key: string; value: string };
		togglePerSideWeight: { enabled: boolean };
		toggleSplitWeight: { enabled: boolean };
	}>();

	let setReps = 10;
	let setWeight = 0;
	let setWeightLeft = 0;
	let setWeightRight = 0;
	let notesDraft = '';
	let newSettingKey = '';
	let newSettingValue = '';

	$: notesDraft = exercise.notes;

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

	function formatWeight(
		set: { weight: number; weightLeft?: number; weightRight?: number },
		perSideWeight: boolean,
		splitWeight: boolean
	): string {
		if (!perSideWeight) return `${set.weight}kg`;
		if (!splitWeight) return `${set.weight}kg/side`;
		const left = set.weightLeft ?? set.weight;
		const right = set.weightRight ?? set.weight;
		return left === right ? `${left}kg/side` : `${left}/${right}kg`;
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

	function compressSetLabels(
		sets: Array<{ reps: number; weight: number; weightLeft?: number; weightRight?: number }>,
		perSideWeight: boolean,
		splitWeight: boolean
	): Array<{ count: number; label: string }> {
		const compressed: Array<{ count: number; label: string }> = [];
		for (const set of sets) {
			const label = `${set.reps}×${formatWeight(set, perSideWeight, splitWeight)}`;
			const existing = compressed.find((c) => c.label === label);
			if (existing) existing.count += 1;
			else compressed.push({ count: 1, label });
		}
		return compressed;
	}

	function useLastSet() {
		const last = exercise.sets[exercise.sets.length - 1];
		if (!last) return;
		setReps = last.reps;
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
		if (exercise.status === 'done') return;
		if (setReps <= 0) return;
		if (!exercise.perSideWeight) {
			if (setWeight < 0) return;
			dispatch('addSet', { reps: setReps, weight: setWeight });
			return;
		}
		if (!exercise.splitWeight) {
			if (setWeight < 0) return;
			dispatch('addSet', { reps: setReps, weight: setWeight });
			return;
		}
		if (setWeightLeft < 0 || setWeightRight < 0) return;
		dispatch('addSet', {
			reps: setReps,
			weight: (setWeightLeft + setWeightRight) / 2,
			weightLeft: setWeightLeft,
			weightRight: setWeightRight
		});
	}

	function saveNotes() {
		const next = notesDraft.trim();
		if (next === exercise.notes) return;
		dispatch('updateNotes', { notes: next });
	}

	function addSetting() {
		const key = newSettingKey.trim();
		const value = newSettingValue.trim();
		if (!key || !value) return;
		dispatch('addSetting', { key, value });
		newSettingKey = '';
		newSettingValue = '';
	}

	function togglePerSideWeight(enabled: boolean) {
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
				{#if exercise.notes}
					<span class="opacity-60 text-sm" aria-label="Has notes">📝</span>
				{/if}
				{#if exercise.settings.length > 0}
					<span class="opacity-60 text-sm" aria-label="Has settings">⚙️</span>
				{/if}
			</div>
			<div class="mt-1 flex flex-wrap gap-2 text-sm opacity-80">
				<span>{exercise.sets.length} set{exercise.sets.length === 1 ? '' : 's'}</span>
				<span class="opacity-50">•</span>
				<span
					>{Math.round(volumeForSets(exercise.sets, exercise.perSideWeight, exercise.splitWeight))} kg</span
				>
			</div>
		</button>

		<div class="flex items-center gap-2 shrink-0">
			<span class="opacity-60">{isOpen ? '▾' : '▸'}</span>
			<button
				type="button"
				class="btn btn-sm variant-soft-error"
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
						<span class="opacity-60">({new Date(lastTime.startedAt).toLocaleDateString()})</span>
						<div class="flex flex-wrap gap-1">
							{#each compressSetLabels(lastTime.sets, lastTime.perSideWeight, lastTime.splitWeight) as s}
								<span class="badge variant-filled-secondary text-xs">{s.count}×{s.label}</span>
							{/each}
						</div>
					</div>
					{#if lastTime.notes}
						<p class="mt-2 opacity-80">{lastTime.notes}</p>
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
						<div class="grid grid-cols-[1fr_1fr_auto] gap-2">
							<input
								class="input"
								value={s.key}
								placeholder="Setting"
								{disabled}
								on:input={(e) =>
									dispatch('updateSetting', {
										id: s.id,
										key: e.currentTarget.value,
										value: s.value
									})}
							/>
							<input
								class="input"
								value={s.value}
								placeholder="Value"
								{disabled}
								on:input={(e) =>
									dispatch('updateSetting', {
										id: s.id,
										key: s.key,
										value: e.currentTarget.value
									})}
							/>
							<button
								type="button"
								class="btn variant-soft-error"
								on:click={() => dispatch('removeSetting', { id: s.id })}
								{disabled}
								aria-label="Remove setting"
							>
								✕
							</button>
						</div>
					{/each}

					<div class="grid grid-cols-[1fr_1fr_auto] gap-2">
						<input
							class="input"
							bind:value={newSettingKey}
							placeholder="e.g. Bench angle"
							{disabled}
							on:keydown={(e) => e.key === 'Enter' && addSetting()}
						/>
						<input
							class="input"
							bind:value={newSettingValue}
							placeholder="e.g. 30°"
							{disabled}
							on:keydown={(e) => e.key === 'Enter' && addSetting()}
						/>
						<button
							type="button"
							class="btn variant-filled-primary"
							on:click={addSetting}
							disabled={disabled || !newSettingKey.trim() || !newSettingValue.trim()}
						>
							Add
						</button>
					</div>
				</div>
			</section>

			<section class="space-y-2">
				<div class="flex items-center justify-between">
					<h4 class="text-sm font-semibold opacity-80">Sets</h4>
					<div class="flex items-center gap-2">
						<label class="flex items-center gap-2 text-xs opacity-80 select-none">
							<input
								type="checkbox"
								class="checkbox"
								checked={exercise.perSideWeight}
								disabled={disabled || exercise.status === 'done'}
								on:change={(e) => togglePerSideWeight(e.currentTarget.checked)}
							/>
							Per-side weights
						</label>
						{#if exercise.perSideWeight}
							<label class="flex items-center gap-2 text-xs opacity-80 select-none">
								<input
									type="checkbox"
									class="checkbox"
									checked={exercise.splitWeight}
									disabled={disabled || exercise.status === 'done'}
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
								disabled={disabled || exercise.status === 'done'}
							>
								Use last
							</button>
						{/if}
					</div>
				</div>

				{#if exercise.sets.length === 0}
					<p class="text-sm opacity-70">Add your first set for this exercise.</p>
				{:else}
					<div class="flex flex-wrap gap-2">
						{#each exercise.sets as s, idx (s.id)}
							<span class="chip variant-filled text-sm">
								{idx + 1}. {s.reps}×{formatWeight(s, exercise.perSideWeight, exercise.splitWeight)}
							</span>
						{/each}
					</div>
				{/if}

				<div class="grid grid-cols-[1fr_1fr_auto] gap-2 items-end">
					<label class="block">
						<span class="text-xs font-semibold opacity-70">Reps</span>
						<input
							type="number"
							min="0"
							inputmode="numeric"
							pattern="[0-9]*"
							class="input w-full"
							bind:value={setReps}
							disabled={disabled || exercise.status === 'done'}
						/>
					</label>
					{#if !exercise.perSideWeight}
						<label class="block">
							<span class="text-xs font-semibold opacity-70">Weight (kg)</span>
							<input
								type="number"
								min="0"
								step="0.5"
								inputmode="decimal"
								class="input w-full"
								bind:value={setWeight}
								disabled={disabled || exercise.status === 'done'}
							/>
						</label>
					{:else if !exercise.splitWeight}
						<label class="block">
							<span class="text-xs font-semibold opacity-70">Per side (kg)</span>
							<input
								type="number"
								min="0"
								step="0.5"
								inputmode="decimal"
								class="input w-full"
								bind:value={setWeight}
								disabled={disabled || exercise.status === 'done'}
							/>
						</label>
					{:else}
						<div class="grid grid-cols-2 gap-2">
							<label class="block">
								<span class="text-xs font-semibold opacity-70">Left (kg)</span>
								<input
									type="number"
									min="0"
									step="0.5"
									inputmode="decimal"
									class="input w-full"
									bind:value={setWeightLeft}
									disabled={disabled || exercise.status === 'done'}
								/>
							</label>
							<label class="block">
								<span class="text-xs font-semibold opacity-70">Right (kg)</span>
								<input
									type="number"
									min="0"
									step="0.5"
									inputmode="decimal"
									class="input w-full"
									bind:value={setWeightRight}
									disabled={disabled || exercise.status === 'done'}
								/>
							</label>
						</div>
					{/if}
					<button
						type="button"
						class="btn variant-filled-primary"
						on:click={addSet}
						disabled={disabled || exercise.status === 'done'}
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
					{disabled}
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
