import type { Exercise, Set, Workout, FeedbackEmoji } from './types';

export type StoredExerciseState = {
    exercise: Exercise;
    sets: Set[];
};

export type StoredWorkoutState = {
    workout: Workout | null;
    exercises: StoredExerciseState[];
    activeExerciseId: number | null;
    sessionNotes: string;
    sessionFeedback: FeedbackEmoji | null;
};

const STATE_KEY = 'swolemate:currentWorkoutState';
export const CURRENT_WORKOUT_ID_KEY = 'currentWorkoutId';

const isBrowser = typeof window !== 'undefined';

export function saveWorkoutState(state: StoredWorkoutState): void {
    if (!isBrowser) return;
    try {
        localStorage.setItem(STATE_KEY, JSON.stringify(state));
    } catch (error) {
        console.error('Failed to persist workout state', error);
    }
}

export function loadWorkoutState(): StoredWorkoutState | null {
    if (!isBrowser) return null;
    try {
        const raw = localStorage.getItem(STATE_KEY);
        return raw ? (JSON.parse(raw) as StoredWorkoutState) : null;
    } catch (error) {
        console.error('Failed to read workout state', error);
        return null;
    }
}

export function clearWorkoutState(): void {
    if (!isBrowser) return;
    localStorage.removeItem(STATE_KEY);
}

export function updateStoredWorkoutId(oldId: number, newId: number): void {
    if (!isBrowser) return;
    const savedId = localStorage.getItem(CURRENT_WORKOUT_ID_KEY);
    if (savedId && Number(savedId) === oldId) {
        localStorage.setItem(CURRENT_WORKOUT_ID_KEY, String(newId));
    }
}

export function replaceWorkoutId(oldId: number, newId: number): void {
    if (!isBrowser) return;
    const state = loadWorkoutState();
    if (!state || !state.workout || state.workout.id !== oldId) {
        return;
    }

    state.workout.id = newId;
    state.exercises = state.exercises.map((entry) => {
        entry.exercise.workout_id = newId;
        return entry;
    });

    saveWorkoutState(state);
    updateStoredWorkoutId(oldId, newId);
}

export function replaceExerciseId(oldId: number, newId: number): void {
    if (!isBrowser) return;
    const state = loadWorkoutState();
    if (!state) {
        return;
    }

    let updated = false;
    state.exercises = state.exercises.map((entry) => {
        if (entry.exercise.id === oldId) {
            entry.exercise.id = newId;
            entry.sets = entry.sets.map((set) => ({
                ...set,
                exercise_id: set.exercise_id === oldId ? newId : set.exercise_id
            }));
            if (state.activeExerciseId === oldId) {
                state.activeExerciseId = newId;
            }
            updated = true;
        }
        return entry;
    });

    if (updated) {
        saveWorkoutState(state);
    }
}

export function replaceSetId(exerciseId: number, oldId: number, newId: number): void {
    if (!isBrowser) return;
    const state = loadWorkoutState();
    if (!state) {
        return;
    }

    let changed = false;
    state.exercises = state.exercises.map((entry) => {
        if (entry.exercise.id === exerciseId) {
            entry.sets = entry.sets.map((set) => {
                if (set.id === oldId) {
                    changed = true;
                    return { ...set, id: newId };
                }
                return set;
            });
        }
        return entry;
    });

    if (changed) {
        saveWorkoutState(state);
    }
}
