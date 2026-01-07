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
	ExerciseProgress,
	VolumeStats
} from './types';
import { config } from './config';

const API_BASE = config.apiUrl;
type Fetcher = typeof fetch;

async function handleResponse<T>(response: Response): Promise<T> {
	if (!response.ok) {
		const contentType = response.headers.get('content-type') ?? '';
		let message = `HTTP error! status: ${response.status}`;

		if (contentType.includes('application/json')) {
			const error = (await response.json().catch(() => null)) as {
				message?: string;
				error?: string;
			} | null;
			message = error?.message || error?.error || message;
		} else {
			const text = await response.text().catch(() => '');
			if (text.trim()) message = text;
		}

		throw new Error(message);
	}

	if (response.status === 204) {
		return undefined as T;
	}

	const contentType = response.headers.get('content-type') ?? '';
	if (!contentType.includes('application/json')) {
		// Defensive: treat non-JSON success bodies as "no payload" for this app.
		return undefined as T;
	}

	// Defensive: some endpoints may return an empty body with 200/201.
	const text = await response.text().catch(() => '');
	if (!text.trim()) return undefined as T;
	return JSON.parse(text) as T;
}

export async function createWorkout(
	workout: CreateWorkoutRequest,
	fetcher: Fetcher = fetch
): Promise<{ id: number }> {
	const response = await fetcher(`${API_BASE}/api/workouts`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(workout)
	});
	return handleResponse(response);
}

export async function endWorkout(
	id: number,
	endTime: UpdateWorkoutRequest,
	fetcher: Fetcher = fetch
): Promise<void> {
	const response = await fetcher(`${API_BASE}/api/workouts/${id}/end`, {
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(endTime)
	});
	return handleResponse(response);
}

export async function getWorkout(
	id: number,
	fetcher: Fetcher = fetch
): Promise<{ workout: Workout; exercises: Array<{ exercise: Exercise; sets: Set[] }> }> {
	const response = await fetcher(`${API_BASE}/api/workouts/${id}`);
	return handleResponse(response);
}

export async function createExercise(
	workoutId: number,
	exercise: CreateExerciseRequest,
	fetcher: Fetcher = fetch
): Promise<{ id: number }> {
	const response = await fetcher(`${API_BASE}/api/workouts/${workoutId}/exercises`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(exercise)
	});
	return handleResponse(response);
}

export async function endExercise(
	id: number,
	endTime: UpdateExerciseRequest,
	fetcher: Fetcher = fetch
): Promise<void> {
	const response = await fetcher(`${API_BASE}/api/exercises/${id}/end`, {
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(endTime)
	});
	return handleResponse(response);
}

export async function createSet(
	exerciseId: number,
	set: CreateSetRequest,
	fetcher: Fetcher = fetch
): Promise<{ id: number }> {
	const response = await fetcher(`${API_BASE}/api/exercises/${exerciseId}/sets`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(set)
	});
	return handleResponse(response);
}

export async function replaceSets(
	exerciseId: number,
	sets: CreateSetRequest[],
	fetcher: Fetcher = fetch
): Promise<Set[]> {
	const response = await fetcher(`${API_BASE}/api/exercises/${exerciseId}/sets`, {
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(sets)
	});
	return handleResponse(response);
}

export async function getWorkouts(fetcher: Fetcher = fetch): Promise<Workout[]> {
	const response = await fetcher(`${API_BASE}/api/workouts`);
	return handleResponse(response);
}

export async function getExerciseTypes(fetcher: Fetcher = fetch): Promise<string[]> {
	const response = await fetcher(`${API_BASE}/api/exercises/types`);
	return handleResponse(response);
}

export async function getLastExerciseData(
	exerciseType: string,
	fetcher: Fetcher = fetch
): Promise<{ exercise: Exercise; sets: Set[] } | null> {
	const response = await fetcher(
		`${API_BASE}/api/exercises/last/${encodeURIComponent(exerciseType)}`
	);
	const data = await handleResponse<[Exercise, Set[]]>(response);
	if (!data) return null;
	const [exercise, sets] = data;
	return { exercise, sets };
}

export async function cancelExercise(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(`${API_BASE}/api/exercises/${id}`, {
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		}
	});
	return handleResponse(response);
}

export async function cancelWorkout(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(`${API_BASE}/api/workouts/${id}`, {
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		}
	});
	return handleResponse(response);
}

export interface BackupInfo {
	filename: string;
	created_at: string;
	backup_type: 'Auto' | 'Manual';
}

export async function getBackups(fetcher: Fetcher = fetch): Promise<BackupInfo[]> {
	const response = await fetcher(`${API_BASE}/api/backups`);
	return handleResponse(response);
}

export async function createBackup(fetcher: Fetcher = fetch): Promise<BackupInfo> {
	const response = await fetcher(`${API_BASE}/api/backups`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		}
	});
	return handleResponse(response);
}

export async function restoreBackup(filename: string, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/backups/${encodeURIComponent(filename)}/restore`,
		{
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			}
		}
	);
	return handleResponse(response);
}

export async function deleteBackup(filename: string, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(`${API_BASE}/api/backups/${encodeURIComponent(filename)}`, {
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		}
	});
	return handleResponse(response);
}

export async function getWorkoutStats(fetcher: Fetcher = fetch): Promise<WorkoutStats> {
	const response = await fetcher(`${API_BASE}/api/progress/workout-stats`);
	return handleResponse(response);
}

export async function getExerciseProgress(
	exerciseType: string,
	fetcher: Fetcher = fetch
): Promise<ExerciseProgress[]> {
	const response = await fetcher(
		`${API_BASE}/api/progress/exercise/${encodeURIComponent(exerciseType)}`
	);
	return handleResponse(response);
}

export async function getVolumeStats(
	exerciseType: string,
	fetcher: Fetcher = fetch
): Promise<VolumeStats> {
	const response = await fetcher(
		`${API_BASE}/api/progress/volume?exercise_type=${encodeURIComponent(exerciseType)}`
	);
	return handleResponse(response);
}
