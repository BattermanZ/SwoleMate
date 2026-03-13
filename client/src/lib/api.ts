import type {
	Workout,
	Exercise,
	Set,
	CreateWorkoutRequest,
	UpdateWorkoutRequest,
	UpdateWorkoutTimesRequest,
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

export class ApiError extends Error {
	readonly status: number;

	constructor(status: number, message: string) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
	}
}

export function isApiError(e: unknown): e is ApiError {
	return e instanceof ApiError;
}

export function isUnauthorized(e: unknown): boolean {
	return isApiError(e) && e.status === 401;
}

type UnauthorizedHandler = () => void;
let unauthorizedHandler: UnauthorizedHandler | null = null;

export function setUnauthorizedHandler(handler: UnauthorizedHandler | null) {
	unauthorizedHandler = handler;
}

function withCredentials(init: RequestInit | undefined): RequestInit {
	return { credentials: 'include', ...init };
}

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

		if (response.status === 401) unauthorizedHandler?.();
		throw new ApiError(response.status, message);
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
	const response = await fetcher(
		`${API_BASE}/api/workouts`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(workout)
		})
	);
	return handleResponse(response);
}

export async function endWorkout(
	id: number,
	endTime: UpdateWorkoutRequest,
	fetcher: Fetcher = fetch
): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/workouts/${id}/end`,
		withCredentials({
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(endTime)
		})
	);
	return handleResponse(response);
}

export async function updateWorkoutTimes(
	id: number,
	times: UpdateWorkoutTimesRequest,
	fetcher: Fetcher = fetch
): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/workouts/${id}/times`,
		withCredentials({
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(times)
		})
	);
	return handleResponse(response);
}

export async function getWorkout(
	id: number,
	fetcher: Fetcher = fetch
): Promise<{ workout: Workout; exercises: Array<{ exercise: Exercise; sets: Set[] }> }> {
	const response = await fetcher(`${API_BASE}/api/workouts/${id}`, withCredentials(undefined));
	return handleResponse(response);
}

export async function createExercise(
	workoutId: number,
	exercise: CreateExerciseRequest,
	fetcher: Fetcher = fetch
): Promise<{ id: number }> {
	const response = await fetcher(
		`${API_BASE}/api/workouts/${workoutId}/exercises`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(exercise)
		})
	);
	return handleResponse(response);
}

export async function endExercise(
	id: number,
	endTime: UpdateExerciseRequest,
	fetcher: Fetcher = fetch
): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/exercises/${id}/end`,
		withCredentials({
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(endTime)
		})
	);
	return handleResponse(response);
}

export async function createSet(
	exerciseId: number,
	set: CreateSetRequest,
	fetcher: Fetcher = fetch
): Promise<{ id: number }> {
	const response = await fetcher(
		`${API_BASE}/api/exercises/${exerciseId}/sets`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(set)
		})
	);
	return handleResponse(response);
}

export async function replaceSets(
	exerciseId: number,
	sets: CreateSetRequest[],
	fetcher: Fetcher = fetch
): Promise<Set[]> {
	const response = await fetcher(
		`${API_BASE}/api/exercises/${exerciseId}/sets`,
		withCredentials({
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(sets)
		})
	);
	return handleResponse(response);
}

export async function getWorkouts(fetcher: Fetcher = fetch): Promise<Workout[]> {
	const response = await fetcher(`${API_BASE}/api/workouts`, withCredentials(undefined));
	return handleResponse(response);
}

export async function getExerciseTypes(fetcher: Fetcher = fetch): Promise<string[]> {
	const response = await fetcher(`${API_BASE}/api/exercises/types`, withCredentials(undefined));
	return handleResponse(response);
}

export async function getLastExerciseData(
	exerciseType: string,
	fetcher: Fetcher = fetch
): Promise<{ exercise: Exercise; sets: Set[] } | null> {
	const response = await fetcher(
		`${API_BASE}/api/exercises/last/${encodeURIComponent(exerciseType)}`,
		withCredentials(undefined)
	);
	const data = await handleResponse<[Exercise, Set[]]>(response);
	if (!data) return null;
	const [exercise, sets] = data;
	return { exercise, sets };
}

export async function cancelExercise(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/exercises/${id}`,
		withCredentials({
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json'
			}
		})
	);
	return handleResponse(response);
}

export async function cancelWorkout(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/workouts/${id}`,
		withCredentials({
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json'
			}
		})
	);
	return handleResponse(response);
}

export interface BackupInfo {
	filename: string;
	created_at: string;
	backup_type: 'Auto' | 'Manual';
}

export interface McpTokenSummary {
	id: number;
	name: string;
	scopes: string[];
	expires_at: string | null;
	revoked_at: string | null;
	last_used_at: string | null;
	created_at: string;
}

export interface CreatedMcpToken {
	id: number;
	token: string;
	name: string;
	scopes: string[];
	expires_at: string | null;
}

export async function getMcpTokens(fetcher: Fetcher = fetch): Promise<McpTokenSummary[]> {
	const response = await fetcher(`${API_BASE}/api/mcp/tokens`, withCredentials(undefined));
	return handleResponse(response);
}

export async function createMcpToken(
	args: { name: string; scopes: string[]; expires_in_days?: number },
	fetcher: Fetcher = fetch
): Promise<CreatedMcpToken> {
	const response = await fetcher(
		`${API_BASE}/api/mcp/tokens`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(args)
		})
	);
	return handleResponse(response);
}

export async function revokeMcpToken(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/mcp/tokens/${id}/revoke`,
		withCredentials({
			method: 'POST'
		})
	);
	return handleResponse(response);
}

export async function getBackups(fetcher: Fetcher = fetch): Promise<BackupInfo[]> {
	const response = await fetcher(`${API_BASE}/api/backups`, withCredentials(undefined));
	return handleResponse(response);
}

export async function createBackup(fetcher: Fetcher = fetch): Promise<BackupInfo> {
	const response = await fetcher(
		`${API_BASE}/api/backups`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			}
		})
	);
	return handleResponse(response);
}

export async function restoreBackup(filename: string, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/backups/${encodeURIComponent(filename)}/restore`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			}
		})
	);
	return handleResponse(response);
}

export async function deleteBackup(filename: string, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/backups/${encodeURIComponent(filename)}`,
		withCredentials({
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json'
			}
		})
	);
	return handleResponse(response);
}

export async function getWorkoutStats(fetcher: Fetcher = fetch): Promise<WorkoutStats> {
	const response = await fetcher(
		`${API_BASE}/api/progress/workout-stats`,
		withCredentials(undefined)
	);
	return handleResponse(response);
}

export async function getExerciseProgress(
	exerciseType: string,
	fetcher: Fetcher = fetch
): Promise<ExerciseProgress[]> {
	const response = await fetcher(
		`${API_BASE}/api/progress/exercise/${encodeURIComponent(exerciseType)}`,
		withCredentials(undefined)
	);
	return handleResponse(response);
}

export async function getVolumeStats(
	exerciseType: string,
	fetcher: Fetcher = fetch
): Promise<VolumeStats> {
	const response = await fetcher(
		`${API_BASE}/api/progress/volume?exercise_type=${encodeURIComponent(exerciseType)}`,
		withCredentials(undefined)
	);
	return handleResponse(response);
}

export type UserRole = 'admin' | 'user';
export interface PublicUser {
	id: number;
	username: string;
	role: UserRole;
	must_change_password: boolean;
}

export async function authMe(fetcher: Fetcher = fetch): Promise<PublicUser> {
	const response = await fetcher(`${API_BASE}/api/auth/me`, withCredentials(undefined));
	return handleResponse(response);
}

export async function authLogin(
	username: string,
	password: string,
	fetcher: Fetcher = fetch
): Promise<PublicUser> {
	const response = await fetcher(
		`${API_BASE}/api/auth/login`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ username, password })
		})
	);
	const data = await handleResponse<{ status: 'ok'; user: PublicUser }>(response);
	return data.user;
}

export async function authLogout(fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/auth/logout`,
		withCredentials({
			method: 'POST'
		})
	);
	return handleResponse(response);
}

export async function authChangePassword(
	currentPassword: string,
	newPassword: string,
	fetcher: Fetcher = fetch
): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/auth/change-password`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
		})
	);
	return handleResponse(response);
}

export interface AdminUserListItem {
	id: number;
	username: string;
	role: UserRole;
	disabled_at: string | null;
}

export async function adminListUsers(fetcher: Fetcher = fetch): Promise<AdminUserListItem[]> {
	const response = await fetcher(`${API_BASE}/api/admin/users`, withCredentials(undefined));
	return handleResponse(response);
}

export async function adminCreateUser(
	args: { username: string; password: string; role?: UserRole },
	fetcher: Fetcher = fetch
): Promise<{ id: number }> {
	const response = await fetcher(
		`${API_BASE}/api/admin/users`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(args)
		})
	);
	return handleResponse(response);
}

export async function adminDisableUser(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/admin/users/${id}/disable`,
		withCredentials({
			method: 'POST'
		})
	);
	return handleResponse(response);
}

export async function adminResetUserPassword(
	id: number,
	newPassword: string,
	fetcher: Fetcher = fetch
): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/admin/users/${id}/reset-password`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ new_password: newPassword })
		})
	);
	return handleResponse(response);
}

export async function adminDeleteUser(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/admin/users/${id}`,
		withCredentials({
			method: 'DELETE'
		})
	);
	return handleResponse(response);
}
