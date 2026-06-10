import { scopedKey } from '$lib/auth/scope';
import { kvDelete, kvGet, kvSet } from '$lib/offline/storage';
import type { PlannedTemplateExercise } from '$lib/today/types';

const BASE_KEY = 'today.plannedTemplate';

type StoredPlannedTemplate = {
	sessionId: number;
	exercises: PlannedTemplateExercise[];
};

function storageKey(): string {
	return scopedKey(BASE_KEY);
}

function stripLegacyNotes(exercises: PlannedTemplateExercise[]): PlannedTemplateExercise[] {
	return exercises.map((exercise) => {
		const sanitized = { ...exercise };
		delete sanitized.notes;
		return sanitized;
	});
}

export async function persistPlannedTemplate(
	sessionId: number,
	exercises: PlannedTemplateExercise[]
): Promise<void> {
	try {
		if (exercises.length === 0) {
			await kvDelete(storageKey());
			return;
		}
		await kvSet<StoredPlannedTemplate>(storageKey(), {
			sessionId,
			exercises: stripLegacyNotes(exercises)
		});
	} catch {
		// best-effort; ignore storage failures
	}
}

export async function loadPlannedTemplate(
	sessionId: number
): Promise<PlannedTemplateExercise[] | null> {
	try {
		const stored = await kvGet<StoredPlannedTemplate>(storageKey());
		if (!stored || stored.sessionId !== sessionId) return null;
		return stripLegacyNotes(stored.exercises);
	} catch {
		return null;
	}
}

export async function clearPlannedTemplate(): Promise<void> {
	try {
		await kvDelete(storageKey());
	} catch {
		// ignore
	}
}
