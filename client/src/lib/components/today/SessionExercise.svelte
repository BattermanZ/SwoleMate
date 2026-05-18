<script lang="ts">
	import { Btn, Badge, Chk, SetPillList, StepperPill } from '$lib/components/ui';
	import LastTime from './LastTime.svelte';
	import type { UiExercise } from '$lib/today/types';
	import type { SetLike } from '$lib/today/setPills';

	type LastTimeData = {
		startedAt: string;
		notes: string;
		sets: SetLike[];
		perSideWeight: boolean;
		splitWeight: boolean;
	};

	type AddSetPayload = {
		reps: number;
		weight: number;
		weightLeft?: number;
		weightRight?: number;
		durationSeconds?: number;
	};

	interface Props {
		exercise: UiExercise;
		isOpen?: boolean;
		disabled?: boolean;
		lastTime?: LastTimeData | undefined;
		onToggle?: () => void;
		onDelete?: () => void;
		onMarkDone?: () => void;
		onAddSet?: (payload: AddSetPayload) => void;
		onUpdateSet?: (setId: number, payload: AddSetPayload) => void;
		onRemoveSet?: (setId: number) => void;
		onUpdateNotes?: (notes: string) => void;
		onAddSetting?: (key: string, value: string) => void;
		onRemoveSetting?: (id: string) => void;
		onUpdateSetting?: (id: string, key: string, value: string) => void;
		onTogglePerSideWeight?: (enabled: boolean) => void;
		onToggleSplitWeight?: (enabled: boolean) => void;
		onUpdateTracking?: (next: { reps: boolean; time: boolean; weight: boolean }) => void;
	}

	let {
		exercise,
		isOpen = false,
		disabled = false,
		lastTime,
		onToggle,
		onDelete,
		onMarkDone,
		onAddSet,
		onUpdateSet,
		onRemoveSet,
		onUpdateNotes,
		onAddSetting,
		onRemoveSetting,
		onUpdateSetting,
		onTogglePerSideWeight,
		onToggleSplitWeight,
		onUpdateTracking
	}: Props = $props();

	// Add-set form state
	let setReps = $state(12);
	let setWeight = $state(0);
	let setWeightLeft = $state(0);
	let setWeightRight = $state(0);
	let setDurationSeconds = $state(60);
	let notesDraft = $derived(exercise.notes);
	let newSettingKey = $state('');
	let newSettingValue = $state('');
	let editing = $state(false);
	let editingSetId = $state<number | null>(null);
	let settingsOpen = $state(false);

	// Timer overlay state (countdown for timed sets)
	let timerRunning = $state(false);
	let timerTargetSeconds = $state(0);
	let timerRemainingSeconds = $state(0);
	let timerEndsAt = 0;
	let timerInterval: ReturnType<typeof setInterval> | undefined;

	let locked = $derived(exercise.status === 'done' && !editing);

	let tracksReps = $derived(exercise.tracksReps ?? true);
	let tracksTime = $derived(exercise.tracksTime ?? false);
	let tracksWeight = $derived(exercise.tracksWeight ?? true);

	let timerOverlayOpen = $derived(tracksTime && timerTargetSeconds > 0);
	let timerComplete = $derived(
		timerTargetSeconds > 0 && timerRemainingSeconds <= 0 && !timerRunning
	);
	let timerDisplaySeconds = $derived(
		timerTargetSeconds > 0 ? timerRemainingSeconds : setDurationSeconds
	);
	let timerElapsedSeconds = $derived(
		timerTargetSeconds > 0
			? Math.max(0, timerTargetSeconds - Math.max(0, timerRemainingSeconds))
			: 0
	);
	let timerCanSave = $derived(timerComplete || (!timerRunning && timerElapsedSeconds > 0));
	let timerProgress = $derived(
		timerTargetSeconds > 0
			? Math.max(0, Math.min(1, timerRemainingSeconds / timerTargetSeconds))
			: 1
	);
	let timerProgressPct = $derived(`${Math.round(timerProgress * 100)}%`);
	let timerTone = $derived(
		timerComplete || timerProgress <= 0.15
			? 'steady'
			: timerProgress <= 0.4
				? 'warning'
				: 'danger'
	);

	$effect(() => stopTimerInterval);

	let setCount = $derived(exercise.sets.length);
	let summary = $derived.by(() => {
		if (setCount === 0) return 'No sets yet';
		const last = exercise.sets[setCount - 1];
		if (exercise.perSideWeight && exercise.splitWeight) {
			return `last ${last.weightLeft ?? last.weight}/${last.weightRight ?? last.weight}kg`;
		}
		if (exercise.perSideWeight) return `last ${last.weight}kg/side × ${last.reps}`;
		if (last.durationSeconds && !last.weight) return `last ${last.durationSeconds}s`;
		return `last ${last.weight}kg × ${last.reps}`;
	});

	function setFormFromSet(set: SetLike & { id?: number }) {
		if (typeof set.id === 'number') editingSetId = set.id;
		setReps = set.reps;
		setWeight = set.weight;
		setWeightLeft = set.weightLeft ?? 0;
		setWeightRight = set.weightRight ?? 0;
		setDurationSeconds = set.durationSeconds ?? 60;
	}

	function resetSetForm() {
		editingSetId = null;
		setReps = 12;
		setWeight = 0;
		setWeightLeft = 0;
		setWeightRight = 0;
		setDurationSeconds = 60;
	}

	function commitSet() {
		if (locked) return;
		const payload: AddSetPayload = {
			reps: tracksReps ? setReps : 0,
			weight: tracksWeight ? (exercise.splitWeight ? 0 : setWeight) : 0
		};
		if (tracksTime) payload.durationSeconds = setDurationSeconds;
		if (tracksWeight && exercise.perSideWeight && exercise.splitWeight) {
			payload.weightLeft = setWeightLeft;
			payload.weightRight = setWeightRight;
		}
		if (editingSetId !== null) {
			onUpdateSet?.(editingSetId, payload);
			resetSetForm();
			return;
		}
		onAddSet?.(payload);
	}

	function addSetting() {
		const k = newSettingKey.trim();
		const v = newSettingValue.trim();
		if (!k || !v) return;
		onAddSetting?.(k, v);
		newSettingKey = '';
		newSettingValue = '';
	}

	function saveNotes() {
		if (locked) return;
		const next = notesDraft.trim();
		if (next === exercise.notes) return;
		onUpdateNotes?.(next);
	}

	function setTracking(patch: Partial<{ reps: boolean; time: boolean; weight: boolean }>) {
		if (locked) return;
		const next = {
			reps: patch.reps ?? tracksReps,
			time: patch.time ?? tracksTime,
			weight: patch.weight ?? tracksWeight
		};
		if (next.time) next.reps = false;
		if (!next.reps && !next.time) next.reps = true;
		onUpdateTracking?.(next);
	}

	function togglePerSide(enabled: boolean) {
		if (locked) return;
		onTogglePerSideWeight?.(enabled);
		if (!enabled) onToggleSplitWeight?.(false);
	}

	function formatDuration(value: number) {
		const v = Math.max(0, Math.round(value));
		return `${Math.floor(v / 60)}:${String(v % 60).padStart(2, '0')}`;
	}

	function stopTimerInterval() {
		if (timerInterval !== undefined) clearInterval(timerInterval);
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
		timerInterval = setInterval(tickTimer, 250);
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

	function resetCountdown() {
		const target = timerTargetSeconds || Math.max(1, Math.round(setDurationSeconds));
		timerRunning = false;
		timerTargetSeconds = target;
		timerRemainingSeconds = target;
		timerEndsAt = 0;
		stopTimerInterval();
		setDurationSeconds = target;
	}

	function saveTimedSet() {
		if (timerRunning) pauseTimer();
		if (!timerCanSave) return;
		setDurationSeconds = timerComplete ? timerTargetSeconds : timerElapsedSeconds;
		commitSet();
		resetTimer();
	}

	let prGroupIndex = $derived.by(() => {
		// crude heuristic — highlight the heaviest set as PR for visual demo
		if (exercise.sets.length === 0) return null;
		let maxIdx = 0;
		let max = -Infinity;
		exercise.sets.forEach((s, i) => {
			const w = s.weight + (s.weightLeft ?? 0) + (s.weightRight ?? 0);
			if (w > max) {
				max = w;
				maxIdx = i;
			}
		});
		return maxIdx;
	});
</script>

<article class="ex" class:live={exercise.status === 'active'}>
	<header>
		<button
			class="head-tap"
			type="button"
			onclick={onToggle}
			aria-expanded={isOpen}
			aria-controls="ex-body-{exercise.id}"
		>
			<div class="title">
				<h3>{exercise.name}</h3>
				{#if exercise.status === 'done'}
					<Badge tone="done">Done</Badge>
				{:else}
					<Badge tone="live">In progress</Badge>
				{/if}
			</div>
			<div class="summary">
				<span>{setCount} {setCount === 1 ? 'set' : 'sets'}</span>
				<span class="dot">•</span>
				<span>{summary}</span>
			</div>
		</button>
		<div class="actions">
			{#if exercise.status === 'done'}
				<button
					class="badge-btn"
					type="button"
					onclick={() => (editing = !editing)}
					{disabled}
					aria-label={editing ? 'Stop editing' : 'Edit'}>{editing ? 'Done' : 'Edit'}</button
				>
			{/if}
			<button class="toggle" type="button" onclick={onToggle} aria-label="Toggle details"
				>{isOpen ? '▾' : '▸'}</button
			>
			<button class="del" type="button" onclick={onDelete} {disabled} aria-label="Remove exercise"
				>✕</button
			>
		</div>
	</header>

	{#if isOpen}
		<div class="body" id="ex-body-{exercise.id}">
			<!-- Settings -->
			<section class="sub">
				<button
					class="sub-head sub-toggle"
					type="button"
					onclick={() => (settingsOpen = !settingsOpen)}
					aria-expanded={settingsOpen}
					aria-controls="ex-settings-{exercise.id}"
				>
					<div class="sub-head-left">
						<h4>Settings</h4>
						{#if settingsOpen}
							<span class="help">optional equipment setup</span>
						{:else if exercise.settings.length > 0}
							<div class="settings-chips" aria-label="Saved settings">
								{#each exercise.settings as s (s.id)}
									<span class="setting-chip"
										><span class="k">{s.key}</span><span class="v">{s.value}</span></span
									>
								{/each}
							</div>
						{:else}
							<span class="settings-empty">none yet — tap to add</span>
						{/if}
					</div>
					<span class="sub-caret" aria-hidden="true">{settingsOpen ? '▾' : '▸'}</span>
				</button>
				{#if settingsOpen}
					<div class="settings" id="ex-settings-{exercise.id}">
						{#each exercise.settings as s (s.id)}
							<div class="setting-row">
								<input
									class="input"
									value={s.key}
									placeholder="Setting"
									disabled={locked}
									oninput={(e) => onUpdateSetting?.(s.id, e.currentTarget.value, s.value)}
								/>
								<input
									class="input"
									value={s.value}
									placeholder="Value"
									disabled={locked}
									oninput={(e) => onUpdateSetting?.(s.id, s.key, e.currentTarget.value)}
								/>
								<button
									class="x-btn"
									type="button"
									aria-label="Remove setting"
									disabled={locked}
									onclick={() => onRemoveSetting?.(s.id)}>✕</button
								>
							</div>
						{/each}
						<div class="setting-row">
							<input
								class="input"
								bind:value={newSettingKey}
								placeholder="Bench angle"
								disabled={locked}
								onkeydown={(e) => e.key === 'Enter' && addSetting()}
							/>
							<input
								class="input"
								bind:value={newSettingValue}
								placeholder="30°"
								disabled={locked}
								onkeydown={(e) => e.key === 'Enter' && addSetting()}
							/>
							<button
								class="add-btn"
								type="button"
								onclick={addSetting}
								disabled={locked || !newSettingKey.trim() || !newSettingValue.trim()}>Add</button
							>
						</div>
					</div>
				{/if}
			</section>

			<!-- Last time -->
			{#if lastTime}
				<section class="sub">
					<div class="sub-head"><h4>Last time</h4></div>
					<LastTime
						startedAt={lastTime.startedAt}
						notes={lastTime.notes}
						sets={lastTime.sets}
						perSideWeight={lastTime.perSideWeight}
						splitWeight={lastTime.splitWeight}
					/>
				</section>
			{/if}

			<!-- Current sets -->
			<section class="sub">
				<div class="sub-head">
					<h4>Current sets</h4>
					<span class="help">edit or remove logged sets</span>
				</div>

				<div class="tracking">
					{#if !tracksTime}
						<Chk
							label="Reps"
							checked={tracksReps}
							disabled={locked || tracksReps}
							onchange={(v) => setTracking({ reps: v })}
						/>
					{/if}
					<Chk
						label="Time"
						checked={tracksTime}
						disabled={locked}
						onchange={(v) => setTracking({ time: v })}
					/>
					<Chk
						label="Weight"
						checked={tracksWeight}
						disabled={locked}
						onchange={(v) => setTracking({ weight: v })}
					/>
					{#if tracksWeight}
						<Chk
							label="Per-side"
							checked={exercise.perSideWeight}
							disabled={locked}
							onchange={togglePerSide}
						/>
					{/if}
					{#if tracksWeight && exercise.perSideWeight}
						<Chk
							label="Split L/R"
							checked={exercise.splitWeight}
							disabled={locked}
							onchange={(v) => onToggleSplitWeight?.(v)}
						/>
					{/if}
				</div>

				{#if setCount > 0}
					<div class="set-pills">
						<span class="lbl">{setCount} done</span>
						<SetPillList
							sets={exercise.sets}
							perSideWeight={exercise.perSideWeight}
							splitWeight={exercise.splitWeight}
							prGroupIndex={exercise.status === 'active' ? prGroupIndex : null}
						/>
					</div>
					{#if !locked}
						<div class="set-actions" aria-label="Logged set actions">
							{#each exercise.sets as set, i (set.id)}
								<div class="set-action-row">
									<span>Set {i + 1}</span>
									<div>
										<button type="button" onclick={() => setFormFromSet(set)}>Edit</button>
										<button type="button" onclick={() => onRemoveSet?.(set.id)}>Remove</button>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				{/if}

				{#if !locked}
					<div class="add-set" class:split={exercise.perSideWeight && exercise.splitWeight}>
						{#if tracksReps}
							<div class="field reps-field">
								<span class="field-lbl">Reps</span>
								<StepperPill bind:value={setReps} label="Reps" min={0} max={200} />
							</div>
						{/if}
						{#if tracksTime}
							<div class="field">
								<span class="field-lbl">Duration · s</span>
								<StepperPill
									bind:value={setDurationSeconds}
									label="Duration"
									step={5}
									min={1}
									unit="s"
								/>
								<button
									type="button"
									class="start-timer"
									onclick={startTimer}
									disabled={locked || timerRunning}
									aria-label="Start countdown timer"
								>
									▶ Start timer
								</button>
							</div>
						{/if}
						{#if tracksWeight && !exercise.perSideWeight}
							<div class="field">
								<span class="field-lbl">Weight</span>
								<StepperPill bind:value={setWeight} label="Weight" step={2.5} min={0} unit="kg" />
							</div>
						{:else if tracksWeight && !exercise.splitWeight}
							<div class="field">
								<span class="field-lbl">Per side · kg</span>
								<StepperPill bind:value={setWeight} label="Per side" step={2.5} min={0} />
							</div>
						{:else if tracksWeight}
							<div class="field">
								<span class="field-lbl">Left · kg</span>
								<StepperPill bind:value={setWeightLeft} label="Left" step={2.5} min={0} />
							</div>
							<div class="field">
								<span class="field-lbl">Right · kg</span>
								<StepperPill bind:value={setWeightRight} label="Right" step={2.5} min={0} />
							</div>
						{/if}
						{#if editingSetId !== null}
							<button class="cancel-edit" type="button" onclick={resetSetForm}>Cancel edit</button>
						{/if}
						<button class="commit-set" type="button" onclick={commitSet}>
							{editingSetId === null ? `▶ Log set ${setCount + 1}` : '✓ Save set'}
						</button>
					</div>
				{/if}
			</section>

			<!-- Notes -->
			<section class="sub">
				<div class="sub-head"><h4>Notes</h4></div>
				<textarea
					class="notes"
					rows="2"
					bind:value={notesDraft}
					placeholder="Cues, tempo, how it felt…"
					disabled={locked}
					onblur={saveNotes}
				></textarea>
			</section>

			<!-- Footer actions -->
			<footer>
				<Btn variant="soft" onclick={onToggle} {disabled}>Collapse</Btn>
				<Btn
					variant="success"
					onclick={onMarkDone}
					disabled={disabled || exercise.status === 'done'}
				>
					✓ Mark done
				</Btn>
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
				class="timer-dial"
				class:paused={!timerRunning && !timerComplete}
				data-tone={timerTone}
				style={`--timer-progress:${timerProgressPct}`}
			>
				<div class="timer-dial__inner">
					<div class="timer-state">
						{timerComplete ? 'Complete' : timerRunning ? 'Running' : 'Paused'}
					</div>
					<div class="timer-value">{formatDuration(timerDisplaySeconds)}</div>
					<div class="timer-caption">
						Target {formatDuration(timerTargetSeconds)} • Set {exercise.sets.length + 1}
					</div>
				</div>
			</div>

			<div class="timer-actions">
				{#if timerRunning}
					<button type="button" class="t-btn t-btn--soft" onclick={pauseTimer}>Pause</button>
				{:else if timerComplete}
					<button type="button" class="t-btn t-btn--soft" onclick={resetCountdown}>Repeat</button>
				{:else}
					<button type="button" class="t-btn t-btn--soft" onclick={startTimer}>Resume</button>
				{/if}
				<button
					type="button"
					class="t-btn t-btn--primary"
					onclick={saveTimedSet}
					disabled={!timerCanSave}
				>
					Add set
				</button>
				<button type="button" class="t-btn t-btn--ghost" onclick={resetCountdown}>Reset</button>
				<button type="button" class="t-btn t-btn--ghost" onclick={resetTimer}>Close</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.ex {
		background: var(--card);
		border-radius: 22px;
		padding: 16px;
		border: 1px solid var(--line);
		box-shadow: 0 8px 20px -12px var(--shadow-card);
	}
	.ex.live {
		border-color: color-mix(in oklab, var(--clay) 40%, var(--line));
		box-shadow:
			0 14px 32px -14px rgba(255, 94, 31, 0.32),
			inset 0 0 0 1px color-mix(in oklab, var(--clay) 18%, transparent);
	}

	header {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 10px;
		align-items: start;
	}
	.head-tap {
		min-width: 0;
		text-align: left;
		background: transparent;
		border: 0;
		padding: 0;
		cursor: pointer;
		color: inherit;
	}
	.title {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	h3 {
		margin: 0;
		font:
			800 18px/1.1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.01em;
	}
	.summary {
		margin-top: 6px;
		font:
			500 12px/1.35 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}
	.dot {
		color: var(--ink-dim);
	}

	.actions {
		display: flex;
		align-items: center;
		gap: 6px;
		flex: none;
	}
	.toggle {
		width: 38px;
		height: 38px;
		border-radius: 10px;
		border: 0;
		background: var(--bg-2);
		color: var(--ink-soft);
		font:
			700 14px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
	}
	.toggle:active {
		background: var(--ink);
		color: var(--bg);
	}
	.del {
		width: 38px;
		height: 38px;
		border-radius: 10px;
		border: 0;
		background: color-mix(in oklab, var(--clay) 10%, transparent);
		color: var(--clay-text);
		cursor: pointer;
		font:
			700 13px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.badge-btn {
		background: var(--bg-2);
		color: var(--ink-2);
		border: 0;
		padding: 6px 10px;
		border-radius: 999px;
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
	}

	.body {
		margin-top: 14px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.sub {
		padding-top: 14px;
		border-top: 1px solid var(--line);
	}
	.sub:first-child {
		padding-top: 0;
		border-top: 0;
	}
	.sub-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin-bottom: 10px;
	}
	.sub-head h4 {
		margin: 0;
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.sub-head .help {
		font: italic 400 11px/1 'Instrument Serif';
		color: var(--ink-dim);
	}
	.sub-toggle {
		width: 100%;
		background: transparent;
		border: 0;
		padding: 0;
		cursor: pointer;
		color: inherit;
		text-align: left;
		gap: 12px;
	}
	.sub-head-left {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
		min-width: 0;
		flex: 1 1 auto;
	}
	.sub-caret {
		flex: none;
		width: 24px;
		height: 24px;
		display: grid;
		place-items: center;
		border-radius: 8px;
		background: var(--bg-2);
		color: var(--ink-soft);
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.settings-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.setting-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 4px 4px 10px;
		border-radius: 999px;
		background: var(--card-3);
		border: 1px solid var(--line);
		font:
			600 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-2);
		max-width: 100%;
	}
	.setting-chip .k {
		font-weight: 700;
		letter-spacing: 0.02em;
		color: var(--ink-soft);
		text-transform: lowercase;
	}
	.setting-chip .v {
		padding: 4px 8px;
		border-radius: 999px;
		background: var(--card);
		color: var(--ink);
		font-variant-numeric: tabular-nums;
		max-width: 140px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.settings-empty {
		font: italic 400 12px/1.1 'Instrument Serif';
		color: var(--ink-dim);
	}

	.settings {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.setting-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
		gap: 6px;
		align-items: center;
	}
	.input {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 10px;
		padding: 9px 10px;
		font:
			500 13px/1.2 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		outline: 0;
		min-width: 0;
		width: 100%;
	}
	.input:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.x-btn {
		min-width: 38px;
		height: 38px;
		border: 0;
		border-radius: 10px;
		background: color-mix(in oklab, var(--clay) 10%, transparent);
		color: var(--clay-text);
		font:
			800 13px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
	}
	.add-btn {
		min-width: 56px;
		height: 38px;
		border: 0;
		border-radius: 10px;
		background: var(--ink);
		color: var(--card);
		font:
			700 12px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
	}

	.tracking {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.set-pills {
		margin-top: 10px;
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
		align-items: center;
	}
	.set-pills .lbl {
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-right: 4px;
	}
	.set-actions {
		margin-top: 8px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.set-action-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 10px;
		background: var(--card-3);
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.set-action-row div {
		display: flex;
		gap: 6px;
	}
	.set-action-row button {
		border: 0;
		border-radius: 999px;
		padding: 6px 9px;
		background: var(--bg-2);
		color: var(--ink-2);
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
	}

	.add-set {
		margin-top: 12px;
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		gap: 10px;
		align-items: end;
	}
	.add-set.split {
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
	}
	.add-set.split .reps-field {
		grid-column: 1 / -1;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}
	.field-lbl {
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		padding-left: 4px;
	}
	.commit-set {
		grid-column: 1 / -1;
		padding: 16px 10px;
		border: 0;
		border-radius: 999px;
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		color: white;
		font:
			800 14px/1 'Onest',
			system-ui,
			sans-serif;
		box-shadow:
			0 14px 28px -10px rgba(255, 94, 31, 0.55),
			inset 0 -3px 0 var(--clay-deep);
		cursor: pointer;
	}
	.cancel-edit {
		grid-column: 1 / -1;
		border: 0;
		border-radius: 999px;
		padding: 12px 10px;
		background: var(--bg-2);
		color: var(--ink-2);
		font:
			800 12px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
	}

	.notes {
		width: 100%;
		min-height: 48px;
		resize: vertical;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 10px;
		padding: 10px 12px;
		font:
			500 13px/1.45 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		outline: 0;
	}
	.notes:focus {
		border-color: var(--clay);
	}

	footer {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	/* ── Timed-set countdown overlay ─────────────────────────────────── */
	.start-timer {
		margin-top: 2px;
		align-self: start;
		border: 0;
		border-radius: 999px;
		padding: 8px 14px;
		background: var(--ink);
		color: var(--card);
		font:
			800 11px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.04em;
		cursor: pointer;
	}
	.start-timer:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.timer-overlay {
		position: fixed;
		inset: 0;
		z-index: 80;
		display: grid;
		place-items: center;
		padding: 1.25rem;
		padding-bottom: calc(1.25rem + var(--sab));
		background:
			radial-gradient(900px 600px at 0% 0%, rgba(255, 122, 42, 0.18), transparent 60%),
			radial-gradient(900px 600px at 100% 100%, rgba(213, 162, 58, 0.12), transparent 55%),
			rgba(24, 19, 13, 0.78);
		backdrop-filter: blur(14px);
		-webkit-backdrop-filter: blur(14px);
	}

	.timer-panel {
		width: min(100%, 22rem);
		display: grid;
		justify-items: center;
		gap: 1.1rem;
		color: var(--on-deep);
		text-align: center;
	}

	.timer-heading {
		display: grid;
		gap: 0.3rem;
	}
	.timer-kicker {
		font:
			800 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.22em;
		text-transform: uppercase;
		color: var(--on-deep-soft);
	}
	.timer-title {
		font:
			800 18px/1.15 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.01em;
	}

	.timer-dial {
		width: min(74vw, 17rem);
		aspect-ratio: 1;
		border-radius: 9999px;
		display: grid;
		place-items: center;
		padding: 0.85rem;
		--timer-ring-color: var(--clay);
		--timer-track-color: rgba(243, 236, 225, 0.12);
		background:
			conic-gradient(
				from -90deg,
				var(--timer-ring-color) 0 var(--timer-progress),
				var(--timer-track-color) var(--timer-progress) 100%
			),
			rgba(24, 19, 13, 0.85);
		box-shadow:
			0 24px 60px -18px rgba(0, 0, 0, 0.55),
			inset 0 0 0 1px rgba(243, 236, 225, 0.06);
		transition: filter 220ms ease;
	}
	.timer-dial[data-tone='danger'] {
		--timer-ring-color: var(--clay);
	}
	.timer-dial[data-tone='warning'] {
		--timer-ring-color: var(--gold);
	}
	.timer-dial[data-tone='steady'] {
		--timer-ring-color: var(--success);
	}
	.timer-dial.paused {
		filter: saturate(0.55) brightness(0.92);
	}

	.timer-dial__inner {
		width: 100%;
		height: 100%;
		border-radius: inherit;
		background:
			radial-gradient(circle at 50% 35%, rgba(243, 236, 225, 0.06), transparent 60%),
			#1a140f;
		display: grid;
		place-items: center;
		gap: 0.35rem;
		padding: 1rem;
	}
	.timer-state {
		font:
			800 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.22em;
		text-transform: uppercase;
		color: var(--on-deep-soft);
	}
	.timer-value {
		font:
			800 clamp(2.6rem, 11vw, 3.4rem) / 1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.02em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
	}
	.timer-caption {
		font:
			500 11px/1.3 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
		max-width: 14rem;
	}

	.timer-actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 8px;
		width: 100%;
	}
	.t-btn {
		border: 0;
		border-radius: 999px;
		padding: 12px 18px;
		font:
			800 13px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
		min-width: 92px;
	}
	.t-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.t-btn--primary {
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		color: white;
		box-shadow:
			0 14px 28px -10px rgba(255, 94, 31, 0.55),
			inset 0 -3px 0 var(--clay-deep);
	}
	.t-btn--soft {
		background: rgba(243, 236, 225, 0.14);
		color: var(--on-deep);
	}
	.t-btn--ghost {
		background: transparent;
		color: var(--on-deep-soft);
	}
</style>
