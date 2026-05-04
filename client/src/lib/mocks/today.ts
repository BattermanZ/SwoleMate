import { createId } from '$lib/utils/id';

export type UiMood = '😊' | '😐' | '😞';

export type UiSet = {
	id: string;
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
	id: string;
	name: string;
	notes: string;
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
	id: string;
	startedAt: string;
	endedAt?: string;
	notes: string;
	mood?: UiMood;
	exercises: UiExercise[];
};

export const FEEDBACK_OPTIONS = ['😊', '😐', '😞'] as const;

export const EXERCISE_LIBRARY: string[] = [
	'Bench Press',
	'Incline Dumbbell Press',
	'Overhead Press',
	'Pull Ups',
	'Lat Pulldown',
	'Barbell Row',
	'Cable Row',
	'Dips',
	'Tricep Pushdown',
	'Bicep Curl',
	'Squat',
	'Front Squat',
	'Deadlift',
	'Romanian Deadlift',
	'Hip Thrust',
	'Leg Press',
	'Leg Extension',
	'Hamstring Curl',
	'Calf Raises',
	'Plank'
];

function atLocalTime(base: Date, daysAgo: number, hour: number, minute: number): Date {
	const date = new Date(base);
	date.setDate(date.getDate() - daysAgo);
	date.setHours(hour, minute, 0, 0);
	return date;
}

function sessionFrom(
	base: Date,
	opts: {
		daysAgo: number;
		startHour: number;
		startMinute: number;
		durationMinutes: number;
		mood?: UiMood;
		notes?: string;
		exercises: Array<
			Pick<UiExercise, 'name' | 'notes'> & {
				sets: Array<Pick<UiSet, 'reps' | 'weight'>>;
				settings?: Array<{ key: string; value: string }>;
				perSideWeight?: boolean;
				splitWeight?: boolean;
			}
		>;
	}
): UiSession {
	const start = atLocalTime(base, opts.daysAgo, opts.startHour, opts.startMinute);
	const end = new Date(start.getTime() + opts.durationMinutes * 60_000);

	return {
		id: createId('session'),
		startedAt: start.toISOString(),
		endedAt: end.toISOString(),
		notes: opts.notes ?? '',
		mood: opts.mood,
		exercises: opts.exercises.map((e, idx) => ({
			id: createId('ex'),
			name: e.name,
			notes: e.notes,
			status: 'done',
			perSideWeight: e.perSideWeight ?? false,
			splitWeight: e.splitWeight ?? false,
			settings:
				e.settings?.map((s) => ({
					id: createId(`setting_${idx}`),
					key: s.key,
					value: s.value
				})) ?? [],
			sets: e.sets.map((s) => ({
				id: createId(`set_${idx}`),
				reps: s.reps,
				weight: s.weight
			}))
		}))
	};
}

export function createMockRecentSessions(base: Date = new Date()): UiSession[] {
	return [
		sessionFrom(base, {
			daysAgo: 2,
			startHour: 18,
			startMinute: 5,
			durationMinutes: 67,
			mood: '😊',
			notes: 'Felt strong. Keep elbows tucked on bench and control the eccentric.',
			exercises: [
				{
					name: 'Bench Press',
					notes: '1s pause on chest',
					settings: [
						{ key: 'Bench', value: 'Flat' },
						{ key: 'Rack height', value: '6' }
					],
					sets: [
						{ reps: 8, weight: 60 },
						{ reps: 8, weight: 60 },
						{ reps: 6, weight: 65 }
					]
				},
				{
					name: 'Cable Row',
					notes: 'Full stretch at the front',
					settings: [
						{ key: 'Handle', value: 'Neutral' },
						{ key: 'Seat', value: '4' }
					],
					sets: [
						{ reps: 12, weight: 50 },
						{ reps: 12, weight: 50 },
						{ reps: 10, weight: 55 }
					]
				},
				{
					name: 'Incline Dumbbell Press',
					notes: 'Don’t flare wrists',
					perSideWeight: true,
					settings: [{ key: 'Bench angle', value: '30°' }],
					sets: [
						{ reps: 12, weight: 22.5 },
						{ reps: 10, weight: 25 },
						{ reps: 10, weight: 25 }
					]
				}
			]
		}),
		sessionFrom(base, {
			daysAgo: 5,
			startHour: 18,
			startMinute: 20,
			durationMinutes: 72,
			mood: '😐',
			notes: 'Energy was average. Warm-up longer next time.',
			exercises: [
				{
					name: 'Squat',
					notes: 'Depth was good, brace harder on last sets',
					settings: [
						{ key: 'Shoes', value: 'Lifters' },
						{ key: 'Stance', value: 'Medium' }
					],
					sets: [
						{ reps: 5, weight: 90 },
						{ reps: 5, weight: 90 },
						{ reps: 5, weight: 95 }
					]
				},
				{
					name: 'Romanian Deadlift',
					notes: 'Stretch hamstrings, keep lats tight',
					settings: [{ key: 'Straps', value: 'No' }],
					sets: [
						{ reps: 10, weight: 70 },
						{ reps: 10, weight: 70 },
						{ reps: 8, weight: 75 }
					]
				},
				{
					name: 'Calf Raises',
					notes: '2s pause at the top',
					sets: [
						{ reps: 12, weight: 40 },
						{ reps: 12, weight: 40 },
						{ reps: 12, weight: 40 }
					]
				}
			]
		})
	];
}

export function createEmptySession(base: Date = new Date()): UiSession {
	const now = new Date(base);
	return {
		id: createId('session'),
		startedAt: now.toISOString(),
		notes: '',
		exercises: []
	};
}

export function createDemoSession(base: Date = new Date()): UiSession {
	const startedAt = new Date(base);
	startedAt.setMinutes(startedAt.getMinutes() - 18);

	return {
		id: createId('session'),
		startedAt: startedAt.toISOString(),
		notes: 'Today: focus on clean reps and consistent rest times.',
		exercises: [
			{
				id: createId('ex'),
				name: 'Bench Press',
				notes: 'Pause + smooth press',
				status: 'active',
				perSideWeight: false,
				splitWeight: false,
				settings: [
					{ id: createId('setting'), key: 'Bench', value: 'Flat' },
					{ id: createId('setting'), key: 'Grip', value: 'Index on ring' }
				],
				sets: [
					{ id: createId('set'), reps: 8, weight: 60 },
					{ id: createId('set'), reps: 8, weight: 60 }
				]
			},
			{
				id: createId('ex'),
				name: 'Cable Row',
				notes: '',
				status: 'active',
				perSideWeight: false,
				splitWeight: false,
				settings: [{ id: createId('setting'), key: 'Handle', value: 'Neutral' }],
				sets: [{ id: createId('set'), reps: 12, weight: 50 }]
			},
			{
				id: createId('ex'),
				name: 'Incline Dumbbell Press',
				notes: 'Match elbows, slight arch',
				status: 'active',
				perSideWeight: true,
				splitWeight: false,
				settings: [{ id: createId('setting'), key: 'Bench angle', value: '30°' }],
				sets: [{ id: createId('set'), reps: 12, weight: 22.5 }]
			}
		]
	};
}
