// Pure logic for grouping logged sets into segmented "spill" pills.
// Replaces the rendering portion of the old SetPillsHybrid.svelte — keep this
// logic identical so existing tests + behaviour still pass.

export type SetLike = {
	reps: number;
	weight: number;
	weightLeft?: number;
	weightRight?: number;
	durationSeconds?: number;
};

export type SpillGroup = {
	reps: number;
	durationLabel?: string;
	weightLabel?: string;
	bodyweight: boolean;
	count: number;
	sourceIndexes: number[];
	totalWeight: number;
	/** 0–1, computed by `groupSets` based on min/max totalWeight in the group. */
	intensity: number;
};

function formatDuration(seconds: number): string {
	const minutes = Math.floor(seconds / 60);
	const remaining = seconds % 60;
	if (minutes <= 0) return `${remaining}s`;
	return `${minutes}:${String(remaining).padStart(2, '0')}`;
}

function hasWeight(set: SetLike): boolean {
	return set.weight > 0 || set.weightLeft != null || set.weightRight != null;
}

function setTotalWeight(set: SetLike, perSideWeight: boolean, splitWeight: boolean): number {
	if (!perSideWeight) return set.weight;
	if (!splitWeight) return set.weight * 2;
	const left = set.weightLeft ?? set.weight;
	const right = set.weightRight ?? set.weight;
	return left + right;
}

function formatWeight(
	set: SetLike,
	perSideWeight: boolean,
	splitWeight: boolean
): string | undefined {
	if (!hasWeight(set)) return undefined;
	if (!perSideWeight) return `${set.weight}kg`;
	if (!splitWeight) return `${set.weight}kg/side`;
	const left = set.weightLeft ?? set.weight;
	const right = set.weightRight ?? set.weight;
	return left === right ? `${left}kg/side` : `${left}/${right}kg`;
}

export type GroupSetsOptions = {
	perSideWeight?: boolean;
	splitWeight?: boolean;
};

/**
 * Group identical sets into single pills with a count prefix. Identity is
 * determined by `(reps, durationLabel, weightLabel)`. Order is preserved
 * by first-occurrence. Intensity (0–1) is computed per group from min/max
 * totalWeight inside the resulting groups — heavier sets get a fuller weight
 * cell tint.
 */
export function groupSets(sets: SetLike[], opts: GroupSetsOptions = {}): SpillGroup[] {
	const perSideWeight = opts.perSideWeight ?? false;
	const splitWeight = opts.splitWeight ?? false;

	const groups: SpillGroup[] = [];
	for (const [index, set] of sets.entries()) {
		const durationLabel = set.durationSeconds ? formatDuration(set.durationSeconds) : undefined;
		const weightLabel = formatWeight(set, perSideWeight, splitWeight);
		const bodyweight = !hasWeight(set) && set.reps > 0 && !durationLabel;
		const key = `${set.reps}:${durationLabel ?? ''}:${weightLabel ?? ''}`;
		const existing = groups.find(
			(g) => `${g.reps}:${g.durationLabel ?? ''}:${g.weightLabel ?? ''}` === key
		);
		if (existing) {
			existing.count += 1;
			existing.sourceIndexes.push(index);
			continue;
		}
		groups.push({
			reps: set.reps,
			durationLabel,
			weightLabel,
			bodyweight,
			count: 1,
			sourceIndexes: [index],
			totalWeight: setTotalWeight(set, perSideWeight, splitWeight),
			intensity: 0.6 // filled in below
		});
	}

	// Intensity ramp 38–85% (matches the production SetPillsHybrid mapping).
	const totals = groups.map((g) => g.totalWeight);
	const min = totals.length ? Math.min(...totals) : 0;
	const max = totals.length ? Math.max(...totals) : 0;
	for (const g of groups) {
		if (!totals.length || min === max) {
			g.intensity = 0.65;
			continue;
		}
		const t = (g.totalWeight - min) / (max - min);
		g.intensity = 0.38 + t * 0.47;
	}

	return groups;
}

export function estimatedOneRepMax(set: SetLike, opts: GroupSetsOptions = {}): number | null {
	const perSideWeight = opts.perSideWeight ?? false;
	const splitWeight = opts.splitWeight ?? false;
	const effectiveWeight = setTotalWeight(set, perSideWeight, splitWeight);
	if (set.reps < 1 || set.reps > 12 || effectiveWeight <= 0) return null;
	return Math.round(effectiveWeight * (36 / (37 - set.reps)) * 100) / 100;
}

export function estimatedOneRepMaxPrGroupIndex(
	sets: SetLike[],
	baseline: number | null | undefined,
	opts: GroupSetsOptions = {}
): number | null {
	if (baseline == null) return null;
	let bestSetIndex: number | null = null;
	let bestEstimated = baseline;
	for (const [index, set] of sets.entries()) {
		const estimated = estimatedOneRepMax(set, opts);
		if (estimated == null || estimated <= bestEstimated) continue;
		bestEstimated = estimated;
		bestSetIndex = index;
	}
	if (bestSetIndex == null) return null;
	const groups = groupSets(sets, opts);
	const groupIndex = groups.findIndex((group) => group.sourceIndexes.includes(bestSetIndex));
	return groupIndex >= 0 ? groupIndex : null;
}
