export type UiMood = '😊' | '😐' | '😞';

export type UiSet = {
	id: number;
	reps: number;
	weight: number;
	weightLeft?: number;
	weightRight?: number;
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
