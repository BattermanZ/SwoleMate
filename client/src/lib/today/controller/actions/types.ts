import type { UiSession } from '$lib/today/types';

export type ExerciseSeedOptions = {
	notes?: string;
	perSideWeight?: boolean;
	splitWeight?: boolean;
	settings?: Array<{ key: string; value: string }>;
};

export type SeedSet = { reps: number; weight: number; weightLeft?: number; weightRight?: number };

export type LastTime = {
	startedAt: string;
	notes: string;
	sets: UiSession['exercises'][number]['sets'];
	settings: UiSession['exercises'][number]['settings'];
	perSideWeight: boolean;
	splitWeight: boolean;
};
