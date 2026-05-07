import type { Exercise, Set } from '$lib/types';

export function getEffectiveWeight(
	set: Set,
	exercise: Pick<Exercise, 'per_side_weight' | 'split_weight'>
): number {
	const baseWeight = Number(set.weight);
	if (!Number.isFinite(baseWeight)) return 0;

	if (!exercise.per_side_weight) return baseWeight;

	if (exercise.split_weight && set.weight_left != null && set.weight_right != null) {
		const left = Number(set.weight_left);
		const right = Number(set.weight_right);
		if (Number.isFinite(left) && Number.isFinite(right)) return left + right;
	}

	return baseWeight * 2;
}

export function getSetVolume(
	set: Set,
	exercise: Pick<Exercise, 'per_side_weight' | 'split_weight'>
): number {
	const reps = Number(set.reps);
	if (!Number.isFinite(reps)) return 0;
	return reps * getEffectiveWeight(set, exercise);
}
