import type { Workout, Exercise, Set, CreateExerciseRequest, CreateSetRequest } from './types';
import { config } from './config';

const API_BASE = config.apiUrl;

async function handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
        const error = await response.json().catch(() => ({ 
            message: `HTTP error! status: ${response.status}` 
        }));
        console.error('API Error:', {
            status: response.status,
            statusText: response.statusText,
            error
        });
        throw new Error(error.message || `HTTP error! status: ${response.status}`);
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

export async function createExercise(workoutId: number, exercise: CreateExerciseRequest): Promise<{ id: number }> {
    console.log('Creating exercise:', { workoutId, exercise });
    const response = await fetch(`${API_BASE}/workouts/${workoutId}/exercises`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(exercise),
    });
    return handleResponse(response);
}

export async function createSet(exerciseId: number, set: CreateSetRequest): Promise<{ id: number }> {
    console.log('Creating set:', { exerciseId, set });
    const response = await fetch(`${API_BASE}/exercises/${exerciseId}/sets`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(set),
    });
    return handleResponse(response);
}

export async function getWorkouts(): Promise<Workout[]> {
    const response = await fetch(`${API_BASE}/workouts`);
    return handleResponse(response);
} 