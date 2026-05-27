<script lang="ts">
	import { Card } from '$lib/components/ui';

	interface Props {
		/** ISO session start timestamps (rolling ~12 months from the API). */
		dates: string[];
		/** Number of week-columns to render. */
		weeks?: number;
	}
	let { dates, weeks = 53 }: Props = $props();

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

	type Cell = {
		key: string;
		date: Date;
		count: number;
		level: 0 | 1 | 2 | 3;
		future: boolean;
	};

	let model = $derived.by(() => {
		// Bucket sessions by local calendar day.
		const counts = new Map<string, number>();
		for (const iso of dates) {
			const d = new Date(iso);
			if (Number.isNaN(d.getTime())) continue;
			const k = dayKey(d);
			counts.set(k, (counts.get(k) ?? 0) + 1);
		}

		const today = new Date();
		today.setHours(0, 0, 0, 0);
		// Monday-anchored weeks. getDay(): 0=Sun … 6=Sat → shift so Mon=0.
		const dowToday = (today.getDay() + 6) % 7;
		const thisMonday = new Date(today.getTime() - dowToday * DAY);
		const firstMonday = new Date(thisMonday.getTime() - (weeks - 1) * 7 * DAY);

		const level = (c: number): 0 | 1 | 2 | 3 => (c <= 0 ? 0 : c === 1 ? 1 : c === 2 ? 2 : 3);

		const columns: Cell[][] = [];
		const monthLabels: Array<{ col: number; label: string }> = [];
		const weekHas: boolean[] = [];
		let lastMonth = -1;
		let total = 0;
		let activeDays = 0;

		for (let w = 0; w < weeks; w++) {
			const col: Cell[] = [];
			let weekCount = 0;
			for (let dd = 0; dd < 7; dd++) {
				const date = new Date(firstMonday.getTime() + (w * 7 + dd) * DAY);
				const future = date.getTime() > today.getTime();
				const count = future ? 0 : (counts.get(dayKey(date)) ?? 0);
				if (!future && count > 0) {
					total += count;
					activeDays += 1;
					weekCount += count;
				}
				col.push({ key: dayKey(date), date, count, level: level(count), future });

				// Month label sits on the top cell of the column where the month flips.
				if (dd === 0) {
					const m = date.getMonth();
					if (m !== lastMonth) {
						monthLabels.push({ col: w, label: MONTHS[m] });
						lastMonth = m;
					}
				}
			}
			weekHas.push(weekCount > 0);
			columns.push(col);
		}

		// Streaks in consecutive *weeks* with at least one session — a truer read of
		// training consistency than raw day counts for a lift-on-rest-day routine.
		let longestStreak = 0;
		let run = 0;
		for (const has of weekHas) {
			run = has ? run + 1 : 0;
			if (run > longestStreak) longestStreak = run;
		}
		let currentStreak = 0;
		// Allow the in-progress final week to be empty without breaking the streak.
		let i = weekHas.length - 1;
		if (i >= 0 && !weekHas[i]) i--;
		for (; i >= 0 && weekHas[i]; i--) currentStreak++;

		return { columns, monthLabels, total, activeDays, currentStreak, longestStreak };
	});

	function tip(c: Cell): string {
		const label = c.date.toLocaleDateString(undefined, {
			weekday: 'short',
			day: 'numeric',
			month: 'short'
		});
		if (c.future) return label;
		if (c.count === 0) return `No sessions · ${label}`;
		return `${c.count} session${c.count === 1 ? '' : 's'} · ${label}`;
	}
</script>

<Card>
	{#snippet title()}Training calendar <em>— last 12 months</em>{/snippet}
	{#snippet actions()}
		<div class="summary">
			<span><b>{model.total}</b> sessions</span>
			<span><b>{model.activeDays}</b> active days</span>
			<span><b>{model.currentStreak}</b>w streak</span>
			<span>best <b>{model.longestStreak}</b>w</span>
		</div>
	{/snippet}

	<div class="scroll">
		<div class="grid-wrap">
			<div class="months" style="--cols: {model.columns.length}">
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
	</div>

	<div class="legend">
		<span class="lbl">Less</span>
		<span class="cell" data-level="0"></span>
		<span class="cell" data-level="1"></span>
		<span class="cell" data-level="2"></span>
		<span class="cell" data-level="3"></span>
		<span class="lbl">More</span>
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

	.scroll {
		overflow-x: auto;
		padding-bottom: 4px;
	}
	.grid-wrap {
		min-width: max-content;
	}

	/* Cell sizing shared by grid + month header so labels line up. */
	.grid,
	.months {
		--cell: 13px;
		--gap: 3px;
	}

	.months {
		display: grid;
		grid-template-columns: repeat(var(--cols), calc(var(--cell) + var(--gap)));
		margin-left: 30px;
		margin-bottom: 4px;
		height: 13px;
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
		gap: 6px;
	}
	.dow {
		display: grid;
		grid-template-rows: repeat(7, var(--cell, 13px));
		gap: var(--gap, 3px);
		width: 24px;
		flex: none;
	}
	.dow span {
		font:
			700 8px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-dim);
		align-self: center;
	}

	.grid {
		display: flex;
		gap: var(--gap);
	}
	.col {
		display: grid;
		grid-template-rows: repeat(7, var(--cell));
		gap: var(--gap);
	}
	.cell {
		width: var(--cell, 13px);
		height: var(--cell, 13px);
		border-radius: 3px;
		background: var(--card-3);
		border: 1px solid color-mix(in oklab, var(--line) 60%, transparent);
		transition: transform 120ms ease;
	}
	.cell[data-level='1'] {
		background: color-mix(in oklab, var(--clay) 45%, var(--card));
		border-color: transparent;
	}
	.cell[data-level='2'] {
		background: color-mix(in oklab, var(--clay) 72%, var(--card));
		border-color: transparent;
	}
	.cell[data-level='3'] {
		background: var(--clay);
		border-color: transparent;
		box-shadow: 0 0 0 1px color-mix(in oklab, var(--clay) 40%, transparent);
	}
	.cell.future {
		background: transparent;
		border-color: transparent;
	}
	.cell:not(.future):hover {
		transform: scale(1.25);
	}

	.legend {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-top: 12px;
		justify-content: flex-end;
	}
	.legend .cell {
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
