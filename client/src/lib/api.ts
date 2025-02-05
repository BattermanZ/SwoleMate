import type { Workout, Exercise, Set } from './types';

const API_BASE = 'http://localhost:2469';

async function handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
        const error = await response.json().catch(() => ({ message: 'An error occurred' }));
        throw new Error(error.message || 'An error occurred');
    }
    return response.json();
}

export async function createWorkout(workout: Omit<Workout, 'id'>): Promise<{ id: number }> {
    const response = await fetch(`${API_BASE}/workouts`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(workout),
    });
    return handleResponse(response);
}

export async function getWorkout(id: number): Promise<{ workout: Workout; exercises: Array<{ exercise: Exercise; sets: Set[] }> }> {
    const response = await fetch(`${API_BASE}/workouts/${id}`);
    return handleResponse(response);
}

export async function createExercise(workoutId: number, exercise: Omit<Exercise, 'id' | 'workout_id'>): Promise<{ id: number }> {
    const response = await fetch(`${API_BASE}/workouts/${workoutId}/exercises`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(exercise),
    });
    return handleResponse(response);
}

export async function createSet(exerciseId: number, set: Omit<Set, 'id' | 'exercise_id'>): Promise<{ id: number }> {
    const response = await fetch(`${API_BASE}/exercises/${exerciseId}/sets`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(set),
    });
    return handleResponse(response);
} 