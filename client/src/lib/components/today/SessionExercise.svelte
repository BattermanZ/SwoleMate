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
	let setDurationMinutes = $state(1);
	let setDurationSecondsRem = $state(0);
	let setDurationSeconds = $derived(setDurationMinutes * 60 + setDurationSecondsRem);

	function setDurationFromSeconds(total: number) {
		const t = Math.max(0, Math.round(total));
		setDurationMinutes = Math.floor(t / 60);
		setDurationSecondsRem = t % 60;
	}
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
		setDurationFromSeconds(set.durationSeconds ?? 60);
	}

	function resetSetForm() {
		editingSetId = null;
		setReps = 12;
		setWeight = 0;
		setWeightLeft = 0;
		setWeightRight = 0;
		setDurationFromSeconds(60);
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
		setDurationFromSeconds(timerTargetSeconds);
	}

	function startTimer() {
		if (locked || !tracksTime || timerRunning || typeof window === 'undefined') return;
		const target = timerTargetSeconds || Math.max(1, Math.round(setDurationSeconds));
		const remaining = timerRemainingSeconds > 0 ? timerRemainingSeconds : target;
		timerTargetSeconds = target;
		timerRemainingSeconds = remaining;
		setDurationFromSeconds(target);
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
		setDurationFromSeconds(target);
	}

	function saveTimedSet() {
		if (timerRunning) pauseTimer();
		if (!timerCanSave) return;
		setDurationFromSeconds(timerComplete ? timerTargetSeconds : timerElapsedSeconds);
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
							<div class="field duration-field">
								<span class="field-lbl">Duration</span>
								<div class="duration-row">
									<StepperPill
										bind:value={setDurationMinutes}
										label="Minutes"
										step={1}
										min={0}
										max={59}
										unit="min"
									/>
									<StepperPill
										bind:value={setDurationSecondsRem}
										label="Seconds"
										step={5}
										min={0}
										max={59}
										unit="sec"
									/>
								</div>
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
						{#if tracksTime && editingSetId === null}
							<button
								type="button"
								class="start-timer"
								onclick={startTimer}
								disabled={locked || timerRunning || setDurationSeconds < 1}
								aria-label="Start countdown timer"
							>
								▶ Start timer
							</button>
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
	{@const mm = Math.floor(Math.max(0, timerDisplaySeconds) / 60)}
	{@const ss = String(Math.max(0, timerDisplaySeconds) % 60).padStart(2, '0')}
	{@const stateLabel = timerComplete ? 'Done' : timerRunning ? 'Hold' : 'Paused'}
	<div
		class="hold-stage"
		role="dialog"
		aria-modal="true"
		aria-label={`${exercise.name} timer`}
		data-state={timerComplete ? 'complete' : timerRunning ? 'running' : 'paused'}
	>
		<div class="hold-stage__atmo" aria-hidden="true"></div>

		<header class="hold-stage__top">
			<span class="hold-eyebrow" data-tone={timerTone}>
				<span class="hold-eyebrow__dot" aria-hidden="true"></span>
				{stateLabel}
			</span>
			<button
				type="button"
				class="hold-dismiss"
				onclick={resetTimer}
				aria-label="Close timer"
			>
				Close
			</button>
		</header>

		<div class="hold-stage__center">
			<div
				class="hold-readout"
				class:hold-readout--running={timerRunning}
				class:hold-readout--complete={timerComplete}
			>
				<span class="hold-readout__seg">{String(mm).padStart(2, '0')}</span>
				<span class="hold-readout__colon" aria-hidden="true">:</span>
				<span class="hold-readout__seg">{ss}</span>
			</div>

			<div
				class="hold-wick"
				data-tone={timerTone}
				class:hold-wick--complete={timerComplete}
				style={`--p:${timerProgressPct}`}
			>
				<span class="hold-wick__rail" aria-hidden="true"></span>
				<span class="hold-wick__burn" aria-hidden="true"></span>
				<span class="hold-wick__ember" aria-hidden="true"></span>
			</div>

			<p class="hold-meta">
				<em>{exercise.name}</em>
				<span class="hold-meta__sep" aria-hidden="true">·</span>
				set {exercise.sets.length + 1}
				<span class="hold-meta__sep" aria-hidden="true">·</span>
				target {formatDuration(timerTargetSeconds)}
			</p>

			{#if timerComplete}
				<p class="hold-flourish">well held.</p>
			{/if}

			<div class="hold-actions">
				{#if timerComplete}
					<button type="button" class="hold-cta" onclick={saveTimedSet}>
						<span>Log this set</span>
						<span class="hold-cta__arrow" aria-hidden="true">→</span>
					</button>
				{:else if timerRunning}
					<button type="button" class="hold-cta hold-cta--quiet" onclick={pauseTimer}>
						<span>Pause</span>
					</button>
				{:else}
					<button type="button" class="hold-cta" onclick={startTimer}>
						<span>Resume</span>
						<span class="hold-cta__arrow" aria-hidden="true">▶</span>
					</button>
				{/if}

				<div class="hold-secondary">
					{#if !timerComplete && timerCanSave}
						<button type="button" class="hold-link" onclick={saveTimedSet}>Add as-is</button>
					{/if}
					<button type="button" class="hold-link" onclick={resetCountdown}>Restart</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	@keyframes ex-enter {
		from {
			transform: translateY(18px);
			opacity: 0;
		}
	}
	.ex {
		background: var(--card);
		border-radius: 22px;
		padding: 16px;
		border: 1px solid var(--line);
		box-shadow: 0 8px 20px -12px var(--shadow-card);
		animation: ex-enter 280ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
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

	/* ── Timed-set duration + countdown overlay ──────────────────────── */
	.duration-field {
		grid-column: 1 / -1;
	}
	.duration-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		gap: 8px;
	}
	.start-timer {
		grid-column: 1 / -1;
		border: 0;
		border-radius: 999px;
		padding: 14px 18px;
		background: var(--ink);
		color: var(--card);
		font:
			800 13px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.02em;
		cursor: pointer;
		box-shadow: 0 10px 24px -14px rgba(24, 19, 13, 0.55);
	}
	.start-timer:disabled {
		opacity: 0.45;
		cursor: not-allowed;
		box-shadow: none;
	}

	/* ── Editorial Hold (timer overlay) ──────────────────────────────── */
	.hold-stage {
		position: fixed;
		inset: 0;
		z-index: 80;
		display: grid;
		grid-template-rows: auto 1fr;
		padding:
			calc(1.25rem + var(--sat)) calc(1.25rem + var(--sar))
			calc(1.25rem + var(--sab)) calc(1.25rem + var(--sal));
		color: var(--on-deep);
		background: #120d09;
		isolation: isolate;
		overflow: hidden;
		animation: hold-fade-in 280ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
	}
	.hold-stage::before,
	.hold-stage::after {
		content: '';
		position: absolute;
		pointer-events: none;
		z-index: -1;
	}
	.hold-stage::before {
		inset: -20%;
		background:
			radial-gradient(40% 45% at 12% 14%, rgba(255, 122, 42, 0.42), transparent 65%),
			radial-gradient(35% 38% at 88% 92%, rgba(213, 162, 58, 0.28), transparent 70%),
			radial-gradient(50% 55% at 90% 8%, rgba(79, 125, 84, 0.12), transparent 72%);
		filter: blur(12px);
		opacity: 1;
		transition: opacity 320ms ease;
	}
	.hold-stage[data-state='paused']::before {
		opacity: 0.45;
	}
	.hold-stage[data-state='complete']::before {
		background:
			radial-gradient(60% 60% at 50% 30%, rgba(111, 160, 116, 0.32), transparent 70%),
			radial-gradient(35% 38% at 88% 92%, rgba(213, 162, 58, 0.2), transparent 70%);
	}
	.hold-stage__atmo {
		position: absolute;
		inset: 0;
		z-index: -1;
		background-image:
			radial-gradient(rgba(243, 236, 225, 0.05) 1px, transparent 1px),
			radial-gradient(rgba(243, 236, 225, 0.03) 1px, transparent 1px);
		background-size:
			3px 3px,
			7px 7px;
		background-position:
			0 0,
			1px 2px;
		mix-blend-mode: screen;
		opacity: 0.6;
	}

	@keyframes hold-fade-in {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	/* — top eyebrow + dismiss — */
	.hold-stage__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.hold-eyebrow {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px 6px 10px;
		border-radius: 999px;
		background: rgba(243, 236, 225, 0.06);
		border: 1px solid rgba(243, 236, 225, 0.1);
		font:
			800 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.28em;
		text-transform: uppercase;
		color: var(--on-deep);
	}
	.hold-eyebrow__dot {
		width: 7px;
		height: 7px;
		border-radius: 999px;
		background: var(--clay);
		box-shadow: 0 0 0 4px rgba(255, 94, 31, 0.18);
	}
	.hold-eyebrow[data-tone='warning'] .hold-eyebrow__dot {
		background: var(--gold);
		box-shadow: 0 0 0 4px rgba(213, 162, 58, 0.18);
	}
	.hold-eyebrow[data-tone='steady'] .hold-eyebrow__dot {
		background: var(--success);
		box-shadow: 0 0 0 4px rgba(79, 125, 84, 0.22);
	}
	.hold-stage[data-state='running'] .hold-eyebrow__dot {
		animation: hold-dot-pulse 1.6s ease-in-out infinite;
	}
	.hold-stage[data-state='paused'] .hold-eyebrow__dot {
		opacity: 0.5;
	}
	@keyframes hold-dot-pulse {
		0%,
		100% {
			transform: scale(1);
			filter: brightness(1);
		}
		50% {
			transform: scale(1.25);
			filter: brightness(1.25);
		}
	}

	.hold-dismiss {
		appearance: none;
		border: 0;
		background: transparent;
		color: var(--on-deep-soft);
		font:
			italic 400 16px/1 'Instrument Serif',
			serif;
		cursor: pointer;
		padding: 6px 4px;
	}
	.hold-dismiss:hover {
		color: var(--on-deep);
	}

	/* — center readout — */
	.hold-stage__center {
		display: grid;
		justify-items: center;
		align-content: center;
		gap: clamp(1rem, 4vh, 1.75rem);
		text-align: center;
	}
	.hold-readout {
		display: flex;
		align-items: baseline;
		justify-content: center;
		gap: 0.02em;
		font:
			italic 400 clamp(5rem, 28vw, 11rem) / 0.9 'Instrument Serif',
			'Times New Roman',
			serif;
		letter-spacing: -0.04em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
		text-shadow: 0 4px 30px rgba(0, 0, 0, 0.45);
		animation: hold-readout-in 520ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
	}
	@keyframes hold-readout-in {
		from {
			transform: translateY(8px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}
	.hold-readout__seg {
		display: inline-block;
	}
	.hold-readout__colon {
		display: inline-block;
		transform: translateY(-0.06em);
		padding: 0 0.05em;
		color: var(--clay-2);
		opacity: 0.95;
	}
	.hold-readout--running .hold-readout__colon {
		animation: hold-colon-blink 1s steps(2, jump-none) infinite;
	}
	.hold-readout--complete {
		color: var(--success);
	}
	.hold-readout--complete .hold-readout__colon {
		color: var(--success);
	}
	@keyframes hold-colon-blink {
		0%,
		49% {
			opacity: 1;
		}
		50%,
		100% {
			opacity: 0.18;
		}
	}

	/* — wick (horizontal burning progress) — */
	.hold-wick {
		position: relative;
		width: min(78vw, 24rem);
		height: 14px;
		--wick-color: var(--clay);
	}
	.hold-wick[data-tone='warning'] {
		--wick-color: var(--gold);
	}
	.hold-wick[data-tone='steady'] {
		--wick-color: var(--success);
	}
	.hold-wick--complete {
		--wick-color: var(--success);
	}
	.hold-wick__rail {
		position: absolute;
		inset: 50% 0 auto 0;
		height: 1px;
		transform: translateY(-50%);
		background:
			linear-gradient(
				to right,
				transparent 0,
				rgba(243, 236, 225, 0.16) 8%,
				rgba(243, 236, 225, 0.16) 92%,
				transparent 100%
			);
	}
	.hold-wick__burn {
		position: absolute;
		left: 0;
		top: 50%;
		height: 3px;
		width: var(--p, 0%);
		transform: translateY(-50%);
		background: linear-gradient(
			to right,
			rgba(79, 125, 84, 0.5) 0%,
			var(--gold) 35%,
			var(--clay-2) 70%,
			var(--wick-color) 100%
		);
		border-radius: 999px;
		box-shadow:
			0 0 12px rgba(255, 94, 31, 0.38),
			0 0 26px rgba(255, 94, 31, 0.22);
		transition:
			width 250ms linear,
			background 320ms ease;
	}
	.hold-wick--complete .hold-wick__burn {
		background: var(--success);
		box-shadow: 0 0 14px rgba(111, 160, 116, 0.38);
	}
	.hold-wick__ember {
		position: absolute;
		top: 50%;
		left: var(--p, 0%);
		width: 14px;
		height: 14px;
		border-radius: 999px;
		transform: translate(-50%, -50%);
		background: radial-gradient(
			circle at 50% 50%,
			#fff7e9 0%,
			var(--clay-2) 35%,
			var(--clay) 60%,
			transparent 75%
		);
		filter: blur(0.3px);
		transition: left 250ms linear;
		opacity: 1;
	}
	.hold-stage[data-state='paused'] .hold-wick__ember {
		opacity: 0.55;
	}
	.hold-wick--complete .hold-wick__ember {
		opacity: 0;
	}
	.hold-stage[data-state='running'] .hold-wick__ember {
		animation: hold-ember-flicker 1.4s ease-in-out infinite;
	}
	@keyframes hold-ember-flicker {
		0%,
		100% {
			filter: blur(0.3px) brightness(1);
		}
		50% {
			filter: blur(0.6px) brightness(1.25);
		}
	}

	/* — meta + complete flourish — */
	.hold-meta {
		margin: 0;
		font:
			500 13px/1.5 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
		letter-spacing: 0.02em;
	}
	.hold-meta em {
		font:
			italic 400 16px/1 'Instrument Serif',
			serif;
		color: var(--on-deep);
		margin-right: 2px;
	}
	.hold-meta__sep {
		margin: 0 6px;
		color: rgba(243, 236, 225, 0.3);
	}
	.hold-flourish {
		margin: 0;
		font:
			italic 400 22px/1 'Instrument Serif',
			serif;
		color: var(--success);
		animation: hold-flourish-in 460ms 80ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
	}
	@keyframes hold-flourish-in {
		from {
			transform: translateY(6px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}

	/* — actions (sit right under the timer for thumb reach) — */
	.hold-actions {
		display: grid;
		gap: 14px;
		justify-items: center;
		width: 100%;
		max-width: 22rem;
		margin: clamp(0.5rem, 2vh, 1rem) auto 0;
	}
	.hold-cta {
		appearance: none;
		border: 0;
		cursor: pointer;
		width: 100%;
		padding: 18px 22px;
		border-radius: 999px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 12px;
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		color: #fff7ed;
		font:
			800 15px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.02em;
		box-shadow:
			0 18px 36px -12px rgba(255, 94, 31, 0.55),
			inset 0 -3px 0 var(--clay-deep);
		transition:
			transform 120ms ease,
			box-shadow 220ms ease;
	}
	.hold-cta:active {
		transform: translateY(1px);
		box-shadow:
			0 10px 24px -14px rgba(255, 94, 31, 0.5),
			inset 0 -2px 0 var(--clay-deep);
	}
	.hold-cta__arrow {
		font-size: 18px;
		line-height: 1;
		opacity: 0.92;
	}
	.hold-cta--quiet {
		background: rgba(243, 236, 225, 0.08);
		color: var(--on-deep);
		box-shadow: inset 0 0 0 1px rgba(243, 236, 225, 0.14);
	}
	.hold-cta--quiet:active {
		box-shadow: inset 0 0 0 1px rgba(243, 236, 225, 0.22);
	}

	.hold-secondary {
		display: flex;
		gap: 18px;
		align-items: center;
		justify-content: center;
	}
	.hold-link {
		appearance: none;
		border: 0;
		background: transparent;
		cursor: pointer;
		color: var(--on-deep-soft);
		font:
			italic 400 15px/1 'Instrument Serif',
			serif;
		padding: 6px 4px;
		position: relative;
	}
	.hold-link::after {
		content: '';
		position: absolute;
		left: 4px;
		right: 4px;
		bottom: 2px;
		height: 1px;
		background: currentColor;
		opacity: 0.4;
	}
	.hold-link:hover {
		color: var(--on-deep);
	}
	.hold-link:hover::after {
		opacity: 0.7;
	}

	@media (prefers-reduced-motion: reduce) {
		.hold-stage,
		.hold-readout,
		.hold-flourish,
		.hold-readout__colon,
		.hold-eyebrow__dot,
		.hold-wick__ember,
		.hold-wick__burn {
			animation: none !important;
			transition: none !important;
		}
	}
</style>
