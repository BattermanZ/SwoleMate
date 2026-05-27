<script lang="ts">
	import { Card } from '$lib/components/ui';

	interface Sample {
		start_time: string;
		exercise_count?: number;
	}

	interface Props {
		/** Rolling ~12-month session samples from the API. */
		samples: Sample[];
		/** Number of week-columns to render. */
		weeks?: number;
	}
	let { samples, weeks = 53 }: Props = $props();

	const DAY = 86_400_000;
	const DOW_LABELS = ['Mon', '', 'Wed', '', 'Fri', '', ''];
	const MONTHS = [
		'Jan',
		'Feb',
		'Mar',
		'Apr',
		'May',
		'Jun',
		'Jul',
		'Aug',
		'Sep',
		'Oct',
		'Nov',
		'Dec'
	];

	function dayKey(d: Date): string {
		return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
	}

	type Day = { sessions: number; exercises: number };
	type Cell = {
		key: string;
		date: Date;
		sessions: number;
		exercises: number;
		level: 0 | 1 | 2 | 3 | 4;
		future: boolean;
	};

	let model = $derived.by(() => {
		// Bucket sessions + exercises by local calendar day.
		const days = new Map<string, Day>();
		for (const s of samples) {
			const d = new Date(s.start_time);
			if (Number.isNaN(d.getTime())) continue;
			const k = dayKey(d);
			const cur = days.get(k) ?? { sessions: 0, exercises: 0 };
			cur.sessions += 1;
			cur.exercises += s.exercise_count ?? 0;
			days.set(k, cur);
		}

		// Adaptive colour scale: split non-empty days into four bands by their
		// exercise count so the gradient is meaningful regardless of routine size.
		const exCounts = [...days.values()]
			.map((d) => d.exercises)
			.filter((n) => n > 0)
			.sort((a, b) => a - b);
		const q = (p: number) =>
			exCounts.length
				? exCounts[Math.min(exCounts.length - 1, Math.floor(p * exCounts.length))]
				: 0;
		const t1 = Math.max(1, q(0.25));
		const t2 = Math.max(t1 + 1, q(0.55));
		const t3 = Math.max(t2 + 1, q(0.8));
		const level = (ex: number): Cell['level'] =>
			ex <= 0 ? 0 : ex <= t1 ? 1 : ex <= t2 ? 2 : ex <= t3 ? 3 : 4;

		const today = new Date();
		today.setHours(0, 0, 0, 0);
		const dowToday = (today.getDay() + 6) % 7; // Monday = 0
		const thisMonday = new Date(today.getTime() - dowToday * DAY);
		const firstMonday = new Date(thisMonday.getTime() - (weeks - 1) * 7 * DAY);

		const columns: Cell[][] = [];
		const monthLabels: Array<{ col: number; label: string }> = [];
		const weekHas: boolean[] = [];
		let lastMonth = -1;
		let totalSessions = 0;
		let totalExercises = 0;
		let activeDays = 0;

		for (let w = 0; w < weeks; w++) {
			const col: Cell[] = [];
			let weekHit = false;
			for (let dd = 0; dd < 7; dd++) {
				const date = new Date(firstMonday.getTime() + (w * 7 + dd) * DAY);
				const future = date.getTime() > today.getTime();
				const d = future ? undefined : days.get(dayKey(date));
				const sessions = d?.sessions ?? 0;
				const exercises = d?.exercises ?? 0;
				if (sessions > 0) {
					totalSessions += sessions;
					totalExercises += exercises;
					activeDays += 1;
					weekHit = true;
				}
				col.push({ key: dayKey(date), date, sessions, exercises, level: level(exercises), future });

				if (dd === 0) {
					const m = date.getMonth();
					if (m !== lastMonth) {
						monthLabels.push({ col: w, label: MONTHS[m] });
						lastMonth = m;
					}
				}
			}
			weekHas.push(weekHit);
			columns.push(col);
		}

		// Streaks measured in consecutive weeks with at least one session.
		let longestStreak = 0;
		let run = 0;
		for (const has of weekHas) {
			run = has ? run + 1 : 0;
			if (run > longestStreak) longestStreak = run;
		}
		let currentStreak = 0;
		let i = weekHas.length - 1;
		if (i >= 0 && !weekHas[i]) i--; // allow the in-progress week to be empty
		for (; i >= 0 && weekHas[i]; i--) currentStreak++;

		return {
			columns,
			monthLabels,
			totalSessions,
			totalExercises,
			activeDays,
			currentStreak,
			longestStreak
		};
	});

	function tip(c: Cell): string {
		const label = c.date.toLocaleDateString(undefined, {
			weekday: 'short',
			day: 'numeric',
			month: 'short'
		});
		if (c.future) return label;
		if (c.sessions === 0) return `No training · ${label}`;
		const ex = `${c.exercises} exercise${c.exercises === 1 ? '' : 's'}`;
		const se = c.sessions > 1 ? ` · ${c.sessions} sessions` : '';
		return `${ex}${se} · ${label}`;
	}
</script>

<Card>
	{#snippet title()}Training calendar <em>— last 12 months</em>{/snippet}
	{#snippet actions()}
		<div class="summary">
			<span><b>{model.totalExercises}</b> exercises</span>
			<span><b>{model.activeDays}</b> active days</span>
			<span><b>{model.currentStreak}</b>w streak</span>
			<span>best <b>{model.longestStreak}</b>w</span>
		</div>
	{/snippet}

	<div class="heatmap" style="--cols: {model.columns.length}">
		<div class="months">
			{#each model.monthLabels as m (m.col)}
				<span class="month" style="grid-column: {m.col + 1}">{m.label}</span>
			{/each}
		</div>
		<div class="body">
			<div class="dow">
				{#each DOW_LABELS as d, i (i)}
					<span>{d}</span>
				{/each}
			</div>
			<div class="grid">
				{#each model.columns as col, ci (ci)}
					<div class="col">
						{#each col as cell (cell.key)}
							<span
								class="cell"
								class:future={cell.future}
								data-level={cell.level}
								title={tip(cell)}
							></span>
						{/each}
					</div>
				{/each}
			</div>
		</div>
	</div>

	<div class="legend">
		<span class="lbl">Fewer</span>
		<span class="cell" data-level="0"></span>
		<span class="cell" data-level="1"></span>
		<span class="cell" data-level="2"></span>
		<span class="cell" data-level="3"></span>
		<span class="cell" data-level="4"></span>
		<span class="lbl">More exercises</span>
	</div>
</Card>

<style>
	.summary {
		display: flex;
		gap: 14px;
		font:
			600 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		flex-wrap: wrap;
	}
	.summary b {
		font-weight: 800;
		color: var(--ink);
		font-variant-numeric: tabular-nums;
	}

	.heatmap {
		--dow-w: 28px;
		--body-gap: 8px;
		--gap: 4px;
		width: 100%;
	}

	/* Month header divides the same width as the day grid into equal columns,
	   offset by the day-of-week gutter so labels sit above their week. */
	.months {
		display: grid;
		grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
		margin-left: calc(var(--dow-w) + var(--body-gap));
		margin-bottom: 6px;
		height: 12px;
	}
	.month {
		font:
			700 9px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.04em;
		color: var(--ink-soft);
		white-space: nowrap;
	}

	.body {
		display: flex;
		gap: var(--body-gap);
		align-items: stretch;
	}
	.dow {
		width: var(--dow-w);
		flex: none;
		display: flex;
		flex-direction: column;
		gap: var(--gap);
	}
	.dow span {
		flex: 1;
		display: flex;
		align-items: center;
		font:
			700 9px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-dim);
	}

	/* Columns share the remaining width equally; cells stay square via aspect-ratio,
	   so the grid grows to fill whatever the card offers. */
	.grid {
		flex: 1;
		min-width: 0;
		display: flex;
		gap: var(--gap);
	}
	.col {
		flex: 1 1 0;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: var(--gap);
	}
	.cell {
		width: 100%;
		aspect-ratio: 1 / 1;
		border-radius: 3px;
		background: var(--card-3);
		border: 1px solid color-mix(in oklab, var(--line) 60%, transparent);
		transition: transform 120ms ease;
	}
	.cell[data-level='1'] {
		background: color-mix(in oklab, var(--clay) 32%, var(--card));
		border-color: transparent;
	}
	.cell[data-level='2'] {
		background: color-mix(in oklab, var(--clay) 55%, var(--card));
		border-color: transparent;
	}
	.cell[data-level='3'] {
		background: color-mix(in oklab, var(--clay) 80%, var(--card));
		border-color: transparent;
	}
	.cell[data-level='4'] {
		background: var(--clay);
		border-color: transparent;
		box-shadow: 0 0 0 1px color-mix(in oklab, var(--clay) 35%, transparent);
	}
	.cell.future {
		background: transparent;
		border-color: transparent;
	}
	.cell:not(.future):hover {
		transform: scale(1.18);
	}

	.legend {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-top: 14px;
		justify-content: flex-end;
	}
	.legend .cell {
		width: 13px;
		height: 13px;
		aspect-ratio: auto;
		transition: none;
	}
	.legend .lbl {
		font:
			700 9px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin: 0 4px;
	}
</style>
