<script lang="ts">
	import { Btn, SetPillList, Chip } from '$lib/components/ui';
	import { formatDateRelative, formatTime } from '$lib/utils/date';
	import type { UiSession } from '$lib/today/types';

	type AddPayload = {
		name: string;
		perSideWeight?: boolean;
		splitWeight?: boolean;
		tracksReps?: boolean;
		tracksTime?: boolean;
		tracksWeight?: boolean;
		settings?: Array<{ key: string; value: string }>;
	};

	interface Props {
		sessions: UiSession[];
		canAdd?: boolean;
		disabled?: boolean;
		onAddExercise?: (payload: AddPayload) => void;
	}
	let { sessions, canAdd = false, disabled = false, onAddExercise }: Props = $props();

	function durationMinutes(session: UiSession): number | null {
		if (!session.endedAt) return null;
		const start = new Date(session.startedAt).getTime();
		const end = new Date(session.endedAt).getTime();
		const diff = Math.max(0, end - start);
		return Math.round(diff / 60_000);
	}
</script>

<section class="rs">
	<header>
		<h2>Past 2 sessions <em>— quick recall</em></h2>
		<p>Tap "Add" on any exercise to drop it into today's session.</p>
	</header>

	{#if sessions.length === 0}
		<div class="empty">No sessions yet.</div>
	{:else}
		{#each sessions as session (session.id)}
			<article class="session-card">
				<div class="meta">
					<span class="day">{formatDateRelative(session.startedAt)}</span>
					{#if session.mood}<span class="mood" aria-label="Session mood">{session.mood}</span>{/if}
					<span class="time">
						{formatTime(session.startedAt)}
						{#if session.endedAt}– {formatTime(session.endedAt)}{/if}
						{#if durationMinutes(session) !== null}
							· {durationMinutes(session)}m
						{/if}
					</span>
				</div>
				{#if session.notes}<p class="notes">{session.notes}</p>{/if}

				<div class="ex-list">
					{#each session.exercises as ex (ex.id)}
						<div class="ex-mini">
							<div class="left">
								<div class="name">{ex.name}</div>
								{#if ex.settings.length > 0}
									<div class="setbadges">
										{#each ex.settings.slice(0, 2) as s (s.id)}
											<Chip size="xs">{s.key}: {s.value}</Chip>
										{/each}
										{#if ex.settings.length > 2}
											<Chip size="xs">+{ex.settings.length - 2}</Chip>
										{/if}
									</div>
								{/if}
								<div class="pills">
									<SetPillList
										sets={ex.sets}
										perSideWeight={ex.perSideWeight}
										splitWeight={ex.splitWeight}
										size="xs"
									/>
								</div>
								{#if ex.notes}<div class="ex-notes">Notes: {ex.notes}</div>{/if}
							</div>
							{#if canAdd}
								<Btn
									variant="primary"
									size="sm"
									{disabled}
									onclick={() =>
										onAddExercise?.({
											name: ex.name,
											perSideWeight: ex.perSideWeight,
											splitWeight: ex.splitWeight,
											tracksReps: ex.tracksReps,
											tracksTime: ex.tracksTime,
											tracksWeight: ex.tracksWeight,
											settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
										})}
								>
									Add →
								</Btn>
							{/if}
						</div>
					{/each}
				</div>
			</article>
		{/each}
	{/if}
</section>

<style>
	.rs {
		min-width: 0;
	}
	header {
		padding: 0 4px 10px;
	}
	header h2 {
		margin: 0;
		font:
			800 18px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.015em;
	}
	header h2 em {
		font: italic 400 14px/1 'Instrument Serif';
		color: var(--ink-soft);
		margin-left: 6px;
		font-weight: 400;
	}
	header p {
		margin: 4px 0 0;
		font:
			500 12px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}

	.empty {
		text-align: center;
		opacity: 0.8;
		padding: 16px;
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 18px;
	}

	.session-card {
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 18px;
		padding: 14px;
		box-shadow: 0 4px 12px -8px var(--shadow-card);
		margin-top: 10px;
	}
	.meta {
		display: flex;
		align-items: baseline;
		gap: 8px;
		flex-wrap: wrap;
	}
	.day {
		font:
			800 16px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.mood {
		font:
			400 14px/1 'Onest',
			system-ui,
			sans-serif;
		opacity: 0.85;
	}
	.time {
		font:
			500 12px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.notes {
		margin: 6px 0 0;
		font: italic 400 13px/1.4 'Instrument Serif';
		color: var(--ink-2);
	}

	.ex-list {
		margin-top: 10px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.ex-mini {
		padding: 10px 12px;
		border-radius: 12px;
		background: var(--card-3);
		border: 1px solid var(--line);
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 10px;
		align-items: start;
	}
	.left {
		min-width: 0;
	}
	.name {
		font:
			800 14px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.setbadges {
		margin-top: 6px;
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.pills {
		margin-top: 6px;
	}
	.ex-notes {
		margin-top: 6px;
		font: italic 400 12px/1.4 'Instrument Serif';
		color: var(--ink-soft);
	}
</style>
