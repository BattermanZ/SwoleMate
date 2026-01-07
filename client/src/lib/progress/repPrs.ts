export type RepPr = {
	reps: number;
	weight: number;
};

export function summarizeRepPrs(repPrs: RepPr[]): RepPr[] {
	const maxByReps = new Map<number, number>();

	for (const pr of repPrs) {
		if (!Number.isFinite(pr.reps) || !Number.isFinite(pr.weight)) continue;
		const current = maxByReps.get(pr.reps);
		if (current === undefined || pr.weight > current) {
			maxByReps.set(pr.reps, pr.weight);
		}
	}

	return Array.from(maxByReps.entries())
		.map(([reps, weight]) => ({ reps, weight }))
		.sort((a, b) => a.reps - b.reps);
}
