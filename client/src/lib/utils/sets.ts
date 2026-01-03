export function compressSets(
	sets: Array<{ reps: number; weight: number }>
): Array<{ count: number; reps: number; weight: number }> {
	const compressed: Array<{ count: number; reps: number; weight: number }> = [];

	for (const set of sets) {
		const existing = compressed.find((c) => c.reps === set.reps && c.weight === set.weight);
		if (existing) existing.count++;
		else compressed.push({ count: 1, reps: set.reps, weight: set.weight });
	}

	return compressed;
}
