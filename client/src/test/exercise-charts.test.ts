import { render } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ExerciseCharts from '$lib/components/progress/ExerciseCharts.svelte';
import type { ExerciseProgress, VolumeStats } from '$lib/types';

const chartingMocks = vi.hoisted(() => ({
	upsertChart: vi.fn<
		(current: unknown, canvas: HTMLCanvasElement | null, config: { type?: string }) => {
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
	rgba: vi.fn((color: string) => color)
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
	monthly_volume: [{ month: '2026-01', total_volume: 4000, max_weight: 120, total_reps: 200, total_sets: 40 }],
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
	rgba: chartingMocks.rgba
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

		const types = chartingMocks.upsertChart.mock.calls
			.map((call) => call[2]?.type)
			.filter(Boolean);
		expect(types).toContain('line');
		expect(types).toContain('bar');
		expect(types).not.toContain('scatter');
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

		const types = chartingMocks.upsertChart.mock.calls
			.map((call) => call[2]?.type)
			.filter(Boolean);
		expect(types).toContain('scatter');
	});
});
