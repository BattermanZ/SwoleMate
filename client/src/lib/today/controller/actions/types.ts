import type { UiSession } from '$lib/today/types';

export type ExerciseSeedOptions = {
	notes?: string;
	perSideWeight?: boolean;
	splitWeight?: boolean;
	tracksReps?: boolean;
	tracksTime?: boolean;
	tracksWeight?: boolean;
	settings?: Array<{ key: string; value: string }>;
};

export type SeedSet = {
	reps: number;
	weight: number;
	weightLeft?: number;
	weightRight?: number;
	durationSeconds?: number;
};

export type LastTime = {
	startedAt: string;
	notes: string;
	sets: UiSession['exercises'][number]['sets'];
	settings: UiSession['exercises'][number]['settings'];
	tracksReps?: boolean;
	tracksTime?: boolean;
	tracksWeight?: boolean;
	perSideWeight: boolean;
	splitWeight: boolean;
};
