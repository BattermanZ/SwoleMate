import type { Exercise, Set, Workout } from '$lib/types';
import type { UiExercise, UiExerciseSetting, UiSession, UiSet } from '$lib/today/types';
import {
	decodeTrackingFields,
	isTrackingFieldsSetting,
	TRACKING_FIELDS_SETTING_KEY
} from '$lib/today/tracking';

function ms(iso: string): number {
	return new Date(iso).getTime();
}

export function workoutIsActive(workout: Workout): boolean {
	return ms(workout.end_time) <= ms(workout.start_time);
}

export function exerciseIsDone(exercise: Exercise): boolean {
	return ms(exercise.end_time) > ms(exercise.start_time);
}

export function toUiSet(set: Set): UiSet {
	return {
		id: set.id ?? 0,
		reps: Number(set.reps),
		weight: set.weight,
		weightLeft: set.weight_left ?? undefined,
		weightRight: set.weight_right ?? undefined,
		durationSeconds: set.duration_seconds ?? undefined
	};
}

export function toUiExercise(
	exercise: Exercise,
	sets: Set[],
	settings: UiExerciseSetting[] = []
): UiExercise {
	const id = exercise.id ?? 0;
	const resolvedSettings: UiExerciseSetting[] =
		settings.length > 0
			? settings
			: (exercise.settings ?? []).map((s, idx) => ({
					id: s.id != null ? String(s.id) : `${id}:${idx}`,
					key: s.key,
					value: s.value
				}));
	const tracking = decodeTrackingFields(
		resolvedSettings.find((s) => s.key === TRACKING_FIELDS_SETTING_KEY)?.value
	);

	return {
		id,
		name: exercise.exercise_type,
		notes: exercise.notes ?? '',
		startedAt: exercise.start_time,
		endedAt: exercise.end_time,
		perSideWeight: exercise.per_side_weight ?? false,
		splitWeight: exercise.split_weight ?? false,
		status: exerciseIsDone(exercise) ? 'done' : 'active',
		settings: resolvedSettings.filter((s) => !isTrackingFieldsSetting(s)),
		tracksReps: tracking.reps,
		tracksTime: tracking.time,
		tracksWeight: tracking.weight,
		sets: sets.map(toUiSet)
	};
}

export function toUiSession(
	workout: Workout,
	exercises: Array<{ exercise: Exercise; sets: Set[] }>
): UiSession {
	const id = workout.id ?? 0;
	const endedAt = workoutIsActive(workout) ? undefined : workout.end_time;
	return {
		id,
		startedAt: workout.start_time,
		endedAt,
		notes: workout.notes ?? '',
		mood: (workout.feedback as UiSession['mood']) ?? undefined,
		exercises: exercises.map((e) => toUiExercise(e.exercise, e.sets))
	};
}
