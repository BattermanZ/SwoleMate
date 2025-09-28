import type { 
    Workout, Exercise, Set, 
    CreateWorkoutRequest, UpdateWorkoutRequest,
    CreateExerciseRequest, UpdateExerciseRequest,
    CreateSetRequest 
} from './types';
import { config } from './config';
import { saveToCache, getFromCache, isOnline as clientOnlineCheck } from './offlineCache';

const isBrowser = typeof window !== 'undefined';

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

async function fetchWithCache<T>(url: string, cacheKey: string): Promise<T> {
    if (!isBrowser) {
        const response = await fetch(url);
        return handleResponse<T>(response);
    }

    if (clientOnlineCheck()) {
        try {
            const response = await fetch(url);
            const data = await handleResponse<T>(response);
            saveToCache(cacheKey, data);
            return data;
        } catch (error) {
            const cached = getFromCache<T>(cacheKey);
            if (cached) {
                return cached;
            }
            throw error;
        }
    }

    const cached = getFromCache<T>(cacheKey);
    if (cached) {
        return cached;
    }

    throw new Error('Offline and no cached data available');
}

export async function createWorkout(workout: CreateWorkoutRequest): Promise<{ id: number }> {
    const response = await fetch(`${API_BASE}/api/workouts`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(workout),
    });
    return handleResponse(response);
}

export async function endWorkout(id: number, endTime: UpdateWorkoutRequest): Promise<void> {
    const response = await fetch(`${API_BASE}/api/workouts/${id}/end`, {
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(endTime),
    });
    return handleResponse(response);
}

export async function getWorkout(id: number): Promise<{ workout: Workout; exercises: Array<{ exercise: Exercise; sets: Set[] }> }> {
    const url = `${API_BASE}/api/workouts/${id}`;
    return fetchWithCache(url, `workout:${id}`);
}

export async function createExercise(workoutId: number, exercise: CreateExerciseRequest): Promise<{ id: number }> {
    const response = await fetch(`${API_BASE}/api/workouts/${workoutId}/exercises`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(exercise),
    });
    return handleResponse(response);
}

export async function endExercise(id: number, endTime: UpdateExerciseRequest): Promise<void> {
    const response = await fetch(`${API_BASE}/api/exercises/${id}/end`, {
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(endTime),
    });
    return handleResponse(response);
}

export async function createSet(exerciseId: number, set: CreateSetRequest): Promise<{ id: number }> {
    console.log('Creating set:', { exerciseId, set });
    const response = await fetch(`${API_BASE}/api/exercises/${exerciseId}/sets`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(set),
    });
    return handleResponse(response);
}

export async function getWorkouts(): Promise<Workout[]> {
    const url = `${API_BASE}/api/workouts`;
    return fetchWithCache(url, 'workouts');
}

export async function getExerciseTypes(): Promise<string[]> {
    const url = `${API_BASE}/api/exercises/types`;
    return fetchWithCache(url, 'exercise-types');
}

export async function getLastExerciseData(exerciseType: string): Promise<{ exercise: Exercise; sets: Set[] } | null> {
    const url = `${API_BASE}/api/exercises/last/${encodeURIComponent(exerciseType)}`;
    const data = await fetchWithCache<[Exercise, Set[]] | null>(url, `last-exercise:${exerciseType.toLowerCase()}`);
    if (!data) return null;
    const [exercise, sets] = data;
    return { exercise, sets };
}

export async function cancelExercise(id: number): Promise<void> {
    const response = await fetch(`${API_BASE}/api/exercises/${id}`, {
        method: 'DELETE',
        headers: {
            'Content-Type': 'application/json',
        },
    });
    return handleResponse(response);
}

export async function cancelWorkout(id: number): Promise<void> {
    const response = await fetch(`${API_BASE}/api/workouts/${id}`, {
        method: 'DELETE',
        headers: {
            'Content-Type': 'application/json',
        },
    });
    return handleResponse(response);
}

export interface BackupInfo {
    filename: string;
    created_at: string;
    backup_type: 'Auto' | 'Manual';
}

export async function getBackups(): Promise<BackupInfo[]> {
    const url = `${API_BASE}/api/backups`;
    return fetchWithCache(url, 'backups');
}

export async function createBackup(): Promise<BackupInfo> {
    const response = await fetch(`${API_BASE}/api/backups`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
    });
    return handleResponse(response);
}

export async function restoreBackup(filename: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/backups/${encodeURIComponent(filename)}/restore`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
    });
    return handleResponse(response);
}

export async function deleteBackup(filename: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/backups/${encodeURIComponent(filename)}`, {
        method: 'DELETE',
        headers: {
            'Content-Type': 'application/json',
        },
    });
    return handleResponse(response);
}

export async function getWorkoutStats(): Promise<WorkoutStats> {
    const url = `${API_BASE}/api/progress/workout-stats`;
    return fetchWithCache(url, 'workout-stats');
}

export async function getExerciseProgress(exerciseType: string): Promise<ExerciseProgress[]> {
    const url = `${API_BASE}/api/progress/exercise/${encodeURIComponent(exerciseType)}`;
    return fetchWithCache(url, `exercise-progress:${exerciseType.toLowerCase()}`);
}

export async function getVolumeStats(exerciseType: string): Promise<VolumeStats> {
    const url = `${API_BASE}/api/progress/volume?exercise_type=${encodeURIComponent(exerciseType)}`;
    return fetchWithCache(url, `volume-stats:${exerciseType.toLowerCase()}`);
}
