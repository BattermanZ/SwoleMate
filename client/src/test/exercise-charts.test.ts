import { render } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ExerciseCharts from '$lib/components/progress/ExerciseCharts.svelte';
import type { ExerciseProgress, VolumeStats } from '$lib/types';

const chartingMocks = vi.hoisted(() => ({
	upsertChart: vi.fn<
		(
			current: unknown,
			canvas: HTMLCanvasElement | null,
			config: { type?: string }
		) => {
			destroy: () => void;
		}
	>(() => ({ destroy: vi.fn() })),
	baseOptions: vi.fn(() => ({ scales: {} })),
	readTheme: vi.fn(() => ({
		isDark: false,
		text: '#111',
		mutedText: '#222',
		grid: '#333',
		primary: '#444',
		secondary: '#555',
		tertiary: '#666',
		success: '#777',
		warning: '#888',
		error: '#999'
	})),
	observeTheme: vi.fn(() => ({ disconnect: vi.fn() })),
	rgba: vi.fn((color: string) => color),
	sqliteWeekKeyToTimestamp: vi.fn((weekKey: string) => {
		const match = weekKey.match(/^(?<year>\d{4})-(?:W)?(?<week>\d{1,2})$/);
		if (!match?.groups) return null;
		return Date.UTC(Number(match.groups.year), 0, 1 + Number(match.groups.week) * 7);
	})
}));

const sampleVolumeStats: VolumeStats = {
	weekly_volume: [
		{
			week: '2026-W01',
			total_volume: 1000,
			max_weight: 120,
			total_reps: 50,
			total_sets: 10,
			max_estimated_1rm: 120
		}
	],
	monthly_volume: [
		{ month: '2026-01', total_volume: 4000, max_weight: 120, total_reps: 200, total_sets: 40 }
	],
	personal_records: {
		all_time_max_weight: 120,
		estimated_max_1rm: 130,
		max_volume: 5000,
		rep_prs: []
	}
};

vi.mock('$lib/progress/charting', () => ({
	upsertChart: chartingMocks.upsertChart,
	baseOptions: chartingMocks.baseOptions,
	readTheme: chartingMocks.readTheme,
	observeTheme: chartingMocks.observeTheme,
	rgba: chartingMocks.rgba,
	sqliteWeekKeyToTimestamp: chartingMocks.sqliteWeekKeyToTimestamp
}));

describe('ExerciseCharts', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('does not render charts when volume stats are absent', () => {
		render(ExerciseCharts, {
			props: {
				volumeStats: null,
				exerciseProgress: null
			}
		});
		expect(chartingMocks.upsertChart).not.toHaveBeenCalled();
	});

	it('renders only volume/monthly charts when exercise progress is empty', () => {
		render(ExerciseCharts, {
			props: {
				volumeStats: sampleVolumeStats,
				exerciseProgress: []
			}
		});

		const types = chartingMocks.upsertChart.mock.calls.map((call) => call[2]?.type).filter(Boolean);
		expect(types).toContain('line');
		expect(types).toContain('bar');
		expect(types).not.toContain('scatter');
	});

	it('plots weekly volume as weekly points on a monthly time axis', () => {
		render(ExerciseCharts, {
			props: {
				volumeStats: sampleVolumeStats,
				exerciseProgress: []
			}
		});

		const volumeConfig = chartingMocks.upsertChart.mock.calls.find(
			(call) => call[2]?.type === 'line'
		)?.[2] as {
			data?: { datasets?: Array<{ data?: Array<{ x: number; y: number }> }> };
			options?: {
				scales?: { x?: { type?: string; time?: { unit?: string }; title?: { text?: string } } };
			};
		};

		expect(volumeConfig.data?.datasets?.[0]?.data).toEqual([{ x: Date.UTC(2026, 0, 8), y: 1000 }]);
		expect(volumeConfig.options?.scales?.x?.type).toBe('time');
		expect(volumeConfig.options?.scales?.x?.time?.unit).toBe('month');
		expect(volumeConfig.options?.scales?.x?.title?.text).toBe('Month');
	});

	it('renders progress scatter chart when exercise progress exists', () => {
		render(ExerciseCharts, {
			props: {
				volumeStats: sampleVolumeStats,
				exerciseProgress: [
					{
						exercise: {
							id: 1,
							workout_id: 1,
							exercise_type: 'Bench Press',
							start_time: '2026-01-01T10:00:00.000Z',
							end_time: '2026-01-01T10:30:00.000Z',
							per_side_weight: false,
							split_weight: false
						},
						sets: [{ exercise_id: 1, reps: 8, weight: 60 }]
					}
				] as ExerciseProgress[]
			}
		});

		const types = chartingMocks.upsertChart.mock.calls.map((call) => call[2]?.type).filter(Boolean);
		expect(types).toContain('scatter');
	});
});
