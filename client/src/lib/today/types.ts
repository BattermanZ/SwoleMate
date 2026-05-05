export type UiMood = '😊' | '😐' | '😞';

export type UiSet = {
	id: number;
	reps: number;
	weight: number;
	weightLeft?: number;
	weightRight?: number;
	durationSeconds?: number;
};

export type UiExerciseSetting = {
	id: string;
	key: string;
	value: string;
};

export type UiExercise = {
	id: number;
	name: string;
	notes: string;
	startedAt: string;
	endedAt: string;
	sets: UiSet[];
	settings: UiExerciseSetting[];
	tracksReps?: boolean;
	tracksTime?: boolean;
	tracksWeight?: boolean;
	perSideWeight: boolean;
	splitWeight: boolean;
	status: 'active' | 'done';
};

export type UiSession = {
	id: number;
	startedAt: string;
	timezoneOffsetMinutes?: number;
	endedAt?: string;
	notes: string;
	mood?: UiMood;
	exercises: UiExercise[];
};

export type PlannedTemplateExercise = {
	id: number;
	name: string;
	notes?: string;
	perSideWeight?: boolean;
	splitWeight?: boolean;
	tracksReps?: boolean;
	tracksTime?: boolean;
	tracksWeight?: boolean;
	settings?: Array<{ key: string; value: string }>;
};
