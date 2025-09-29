import type {
    Workout,
    Exercise,
    Set,
    CreateWorkoutRequest,
    UpdateWorkoutRequest,
    CreateExerciseRequest,
    UpdateExerciseRequest,
    CreateSetRequest,
    WorkoutStats,
    VolumeStats,
    ExerciseProgress
} from './types';
import { config } from './config';
import { saveToCache, getFromCache, isOnline as clientOnlineCheck } from './offlineCache';
import {
    loadWorkoutState,
    replaceExerciseId,
    replaceSetId,
    replaceWorkoutId,
    updateStoredWorkoutId
} from './workoutState';

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

type MutationQueueItem =
    | { type: 'createWorkout'; tempId: number; payload: CreateWorkoutRequest }
    | { type: 'endWorkout'; workoutId: number; payload: UpdateWorkoutRequest }
    | { type: 'createExercise'; tempId: number; workoutId: number; payload: CreateExerciseRequest }
    | { type: 'endExercise'; exerciseId: number; payload: UpdateExerciseRequest }
    | { type: 'createSet'; tempId: number; exerciseId: number; payload: CreateSetRequest }
    | { type: 'cancelExercise'; exerciseId: number }
    | { type: 'cancelWorkout'; workoutId: number };

type EntityType = 'workout' | 'exercise' | 'set';

type IdMap = {
    workout: Record<number, number>;
    exercise: Record<number, number>;
    set: Record<number, number>;
};

const MUTATION_QUEUE_KEY = 'swolemate:mutationQueue';
const ID_MAP_KEY = 'swolemate:idMap';
const TEMP_ID_PREFIX = 'swolemate:tempId:';

const DEFAULT_ID_MAP: IdMap = {
    workout: {},
    exercise: {},
    set: {}
};

function loadQueue(): MutationQueueItem[] {
    if (!isBrowser) return [];
    const raw = localStorage.getItem(MUTATION_QUEUE_KEY);
    if (!raw) return [];
    try {
        return JSON.parse(raw) as MutationQueueItem[];
    } catch (error) {
        console.error('Failed to parse mutation queue', error);
        return [];
    }
}

function saveQueue(queue: MutationQueueItem[]): void {
    if (!isBrowser) return;
    localStorage.setItem(MUTATION_QUEUE_KEY, JSON.stringify(queue));
}

function enqueueMutation(item: MutationQueueItem): void {
    if (!isBrowser) return;
    const queue = loadQueue();
    queue.push(item);
    saveQueue(queue);
}

function loadIdMap(): IdMap {
    if (!isBrowser) return { ...DEFAULT_ID_MAP };
    const raw = localStorage.getItem(ID_MAP_KEY);
    if (!raw) return { ...DEFAULT_ID_MAP };
    try {
        const parsed = JSON.parse(raw) as IdMap;
        return {
            workout: parsed.workout || {},
            exercise: parsed.exercise || {},
            set: parsed.set || {}
        };
    } catch (error) {
        console.error('Failed to parse ID map', error);
        return { ...DEFAULT_ID_MAP };
    }
}

function saveIdMap(map: IdMap): void {
    if (!isBrowser) return;
    localStorage.setItem(ID_MAP_KEY, JSON.stringify(map));
}

function generateTempId(entity: EntityType): number {
    if (!isBrowser) throw new Error('Temporary IDs are only available in the browser.');
    const key = `${TEMP_ID_PREFIX}${entity}`;
    const current = Number(localStorage.getItem(key) ?? '0') - 1;
    localStorage.setItem(key, String(current));
    return current;
}

function setIdMapping(entity: EntityType, tempId: number, realId: number, map: IdMap): void {
    map[entity][tempId] = realId;
    saveIdMap(map);
}

function resolveId(entity: EntityType, id: number, map: IdMap): number | null {
    if (id >= 0) {
        return id;
    }
    return map[entity][id] ?? null;
}

function removeQueuedItems(predicate: (item: MutationQueueItem) => boolean): void {
    if (!isBrowser) return;
    const queue = loadQueue();
    const filtered = queue.filter((item) => !predicate(item));
    saveQueue(filtered);
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

            const pendingState = loadWorkoutState();
            if (cacheKey === 'workouts' && pendingState?.workout && (pendingState.workout.id ?? 0) < 0) {
                const workouts = Array.isArray(data) ? (data as Workout[]) : [];
                if (!workouts.find((workout) => workout.id === pendingState.workout?.id)) {
                    saveToCache(cacheKey, [pendingState.workout, ...workouts]);
                }
            }

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

let queueProcessingPromise: Promise<void> | null = null;

async function sendJson<T>(url: string, method: 'POST' | 'PUT' | 'DELETE', body?: unknown): Promise<T> {
    const response = await fetch(url, {
        method,
        headers: {
            'Content-Type': 'application/json'
        },
        body: body !== undefined ? JSON.stringify(body) : undefined
    });
    return handleResponse<T>(response);
}

async function processQueueItem(item: MutationQueueItem, idMap: IdMap): Promise<boolean> {
    try {
        switch (item.type) {
            case 'createWorkout': {
                const result = await sendJson<{ id: number }>(`${API_BASE}/api/workouts`, 'POST', item.payload);
                setIdMapping('workout', item.tempId, result.id, idMap);
                replaceWorkoutId(item.tempId, result.id);
                updateStoredWorkoutId(item.tempId, result.id);
                return true;
            }
            case 'endWorkout': {
                const workoutId = resolveId('workout', item.workoutId, idMap);
                if (workoutId === null) return false;
                await sendJson(`${API_BASE}/api/workouts/${workoutId}/end`, 'PUT', item.payload);
                return true;
            }
            case 'createExercise': {
                const workoutId = resolveId('workout', item.workoutId, idMap);
                if (workoutId === null) return false;
                const result = await sendJson<{ id: number }>(`${API_BASE}/api/workouts/${workoutId}/exercises`, 'POST', item.payload);
                setIdMapping('exercise', item.tempId, result.id, idMap);
                replaceExerciseId(item.tempId, result.id);
                return true;
            }
            case 'endExercise': {
                const exerciseId = resolveId('exercise', item.exerciseId, idMap);
                if (exerciseId === null) return false;
                await sendJson(`${API_BASE}/api/exercises/${exerciseId}/end`, 'PUT', item.payload);
                return true;
            }
            case 'createSet': {
                const exerciseId = resolveId('exercise', item.exerciseId, idMap);
                if (exerciseId === null) return false;
                const result = await sendJson<{ id: number }>(`${API_BASE}/api/exercises/${exerciseId}/sets`, 'POST', item.payload);
                setIdMapping('set', item.tempId, result.id, idMap);
                replaceSetId(exerciseId, item.tempId, result.id);
                return true;
            }
            case 'cancelExercise': {
                const exerciseId = resolveId('exercise', item.exerciseId, idMap);
                if (exerciseId === null) {
                    return true;
                }
                await sendJson(`${API_BASE}/api/exercises/${exerciseId}`, 'DELETE');
                return true;
            }
            case 'cancelWorkout': {
                const workoutId = resolveId('workout', item.workoutId, idMap);
                if (workoutId === null) {
                    return true;
                }
                await sendJson(`${API_BASE}/api/workouts/${workoutId}`, 'DELETE');
                return true;
            }
            default:
                return true;
        }
    } catch (error) {
        console.error('Failed to process queued mutation', item, error);
        return false;
    }
}

export function hasPendingMutations(): boolean {
    if (!isBrowser) return false;
    return loadQueue().length > 0;
}

export async function syncOfflineMutations(): Promise<void> {
    if (!isBrowser) return;
    if (queueProcessingPromise) return queueProcessingPromise;

    queueProcessingPromise = (async () => {
        const queue = loadQueue();
        if (!queue.length) {
            queueProcessingPromise = null;
            return;
        }

        const idMap = loadIdMap();
        const remaining: MutationQueueItem[] = [];

        for (let index = 0; index < queue.length; index += 1) {
            const item = queue[index];
            const success = await processQueueItem(item, idMap);
            if (!success) {
                remaining.push(item, ...queue.slice(index + 1));
                break;
            }
        }

        saveQueue(remaining);
        queueProcessingPromise = null;
    })();

    try {
        await queueProcessingPromise;
    } finally {
        queueProcessingPromise = null;
    }
}

function ensureExerciseTypeCached(exerciseType: string): void {
    if (!isBrowser) return;
    try {
        const cached = getFromCache<string[]>('exercise-types') || [];
        if (!cached.includes(exerciseType)) {
            saveToCache('exercise-types', [exerciseType, ...cached]);
        }
    } catch (error) {
        console.error('Failed to update exercise types cache', error);
    }
}

function clearQueuedForExercise(exerciseId: number): void {
    removeQueuedItems((item) => {
        if (item.type === 'createExercise' && item.tempId === exerciseId) {
            return true;
        }
        if ('exerciseId' in item && item.exerciseId === exerciseId) {
            return true;
        }
        return false;
    });
}

function clearQueuedForWorkout(workoutId: number): void {
    if (!isBrowser) return;
    const queue = loadQueue();
    const exerciseIds = new Set<number>();

    queue.forEach((item) => {
        if (item.type === 'createExercise' && item.workoutId === workoutId) {
            exerciseIds.add(item.tempId);
        }
    });

    const filtered = queue.filter((item) => {
        if (item.type === 'createWorkout' && item.tempId === workoutId) {
            return false;
        }
        if ('workoutId' in item && item.workoutId === workoutId) {
            return false;
        }
        if ('exerciseId' in item && exerciseIds.has(item.exerciseId)) {
            return false;
        }
        return true;
    });

    saveQueue(filtered);
}

export async function createWorkout(workout: CreateWorkoutRequest): Promise<{ id: number }> {
    if (isBrowser && clientOnlineCheck()) {
        try {
            const response = await fetch(`${API_BASE}/api/workouts`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(workout)
            });
            return await handleResponse(response);
        } catch (error) {
            console.warn('Falling back to offline queue for createWorkout', error);
        }
    }

    if (!isBrowser) throw new Error('Offline workout creation is only supported in the browser.');
    const tempId = generateTempId('workout');
    enqueueMutation({ type: 'createWorkout', tempId, payload: workout });
    return { id: tempId };
}

export async function endWorkout(id: number, endTime: UpdateWorkoutRequest): Promise<void> {
    if (isBrowser && clientOnlineCheck() && id >= 0) {
        try {
            const response = await fetch(`${API_BASE}/api/workouts/${id}/end`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(endTime)
            });
            await handleResponse(response);
            return;
        } catch (error) {
            console.warn('Falling back to offline queue for endWorkout', error);
        }
    }

    enqueueMutation({ type: 'endWorkout', workoutId: id, payload: endTime });
}

export async function getWorkout(id: number): Promise<{ workout: Workout; exercises: Array<{ exercise: Exercise; sets: Set[] }> }> {
    if (id < 0 && isBrowser) {
        const stored = loadWorkoutState();
        if (stored?.workout && stored.workout.id === id) {
            return {
                workout: stored.workout,
                exercises: stored.exercises
            };
        }
    }

    const url = `${API_BASE}/api/workouts/${id}`;
    return fetchWithCache(url, `workout:${id}`);
}

export async function createExercise(workoutId: number, exercise: CreateExerciseRequest): Promise<{ id: number }> {
    if (isBrowser && clientOnlineCheck() && workoutId >= 0) {
        try {
            const response = await fetch(`${API_BASE}/api/workouts/${workoutId}/exercises`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(exercise)
            });
            const result = await handleResponse<{ id: number }>(response);
            ensureExerciseTypeCached(exercise.exercise_type);
            return result;
        } catch (error) {
            console.warn('Falling back to offline queue for createExercise', error);
        }
    }

    if (!isBrowser) throw new Error('Offline exercise creation is only supported in the browser.');
    const tempId = generateTempId('exercise');
    enqueueMutation({ type: 'createExercise', tempId, workoutId, payload: exercise });
    ensureExerciseTypeCached(exercise.exercise_type);
    return { id: tempId };
}

export async function endExercise(id: number, endTime: UpdateExerciseRequest): Promise<void> {
    if (isBrowser && clientOnlineCheck() && id >= 0) {
        try {
            const response = await fetch(`${API_BASE}/api/exercises/${id}/end`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(endTime)
            });
            await handleResponse(response);
            return;
        } catch (error) {
            console.warn('Falling back to offline queue for endExercise', error);
        }
    }

    enqueueMutation({ type: 'endExercise', exerciseId: id, payload: endTime });
}

export async function createSet(exerciseId: number, set: CreateSetRequest): Promise<{ id: number }> {
    if (isBrowser && clientOnlineCheck() && exerciseId >= 0) {
        try {
            const response = await fetch(`${API_BASE}/api/exercises/${exerciseId}/sets`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(set)
            });
            return await handleResponse(response);
        } catch (error) {
            console.warn('Falling back to offline queue for createSet', error);
        }
    }

    if (!isBrowser) throw new Error('Offline set creation is only supported in the browser.');
    const tempId = generateTempId('set');
    enqueueMutation({ type: 'createSet', tempId, exerciseId, payload: set });
    return { id: tempId };
}

export async function getWorkouts(): Promise<Workout[]> {
    const url = `${API_BASE}/api/workouts`;
    const workouts = await fetchWithCache<Workout[]>(url, 'workouts');

    if (isBrowser) {
        const stored = loadWorkoutState();
        if (stored?.workout && (stored.workout.id ?? 0) < 0) {
            const exists = workouts.some((w) => w.id === stored.workout?.id);
            if (!exists) {
                return [stored.workout, ...workouts];
            }
        }
    }

    return workouts;
}

export async function getExerciseTypes(): Promise<string[]> {
    const url = `${API_BASE}/api/exercises/types`;
    return fetchWithCache(url, 'exercise-types');
}

export async function getLastExerciseData(exerciseType: string): Promise<{ exercise: Exercise; sets: Set[] } | null> {
    const lower = exerciseType.toLowerCase();
    const url = `${API_BASE}/api/exercises/last/${encodeURIComponent(exerciseType)}`;

    try {
        const data = await fetchWithCache<[Exercise, Set[]] | null>(url, `last-exercise:${lower}`);
        if (!data) return null;
        const [exercise, sets] = data;
        return { exercise, sets };
    } catch (error) {
        if (isBrowser) {
            const stored = loadWorkoutState();
            if (stored) {
                const reversed = [...stored.exercises].reverse();
                for (const entry of reversed) {
                    if (entry.exercise.exercise_type.toLowerCase() === lower) {
                        return { exercise: entry.exercise, sets: entry.sets };
                    }
                }
            }
        }
        throw error;
    }
}

export async function cancelExercise(id: number): Promise<void> {
    if (id < 0) {
        clearQueuedForExercise(id);
        return Promise.resolve();
    }

    if (isBrowser && clientOnlineCheck()) {
        try {
            const response = await fetch(`${API_BASE}/api/exercises/${id}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json'
                }
            });
            await handleResponse(response);
            return;
        } catch (error) {
            console.warn('Falling back to offline queue for cancelExercise', error);
        }
    }

    enqueueMutation({ type: 'cancelExercise', exerciseId: id });
}

export async function cancelWorkout(id: number): Promise<void> {
    if (id < 0) {
        clearQueuedForWorkout(id);
        return Promise.resolve();
    }

    if (isBrowser && clientOnlineCheck()) {
        try {
            const response = await fetch(`${API_BASE}/api/workouts/${id}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json'
                }
            });
            await handleResponse(response);
            return;
        } catch (error) {
            console.warn('Falling back to offline queue for cancelWorkout', error);
        }
    }

    enqueueMutation({ type: 'cancelWorkout', workoutId: id });
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
            'Content-Type': 'application/json'
        }
    });
    return handleResponse(response);
}

export async function restoreBackup(filename: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/backups/${encodeURIComponent(filename)}/restore`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        }
    });
    return handleResponse(response);
}

export async function deleteBackup(filename: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/backups/${encodeURIComponent(filename)}`, {
        method: 'DELETE',
        headers: {
            'Content-Type': 'application/json'
        }
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
