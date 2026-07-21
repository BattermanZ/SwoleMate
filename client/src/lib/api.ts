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
	CreateWorkoutTemplateFromWorkoutRequest,
	CreateWorkoutTemplateRequest,
	WorkoutStats,
	DuplicateWorkoutTemplateRequest,
	ExerciseProgress,
	StartWorkoutFromTemplateRequest,
	UpdateWorkoutTemplateRequest,
	VolumeStats,
	ProgressOverview,
	WorkoutTemplate,
	WorkoutTemplateDetail
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

// Ceiling on any single request. A fetch against a dead socket (common the
// instant connectivity flaps back) can otherwise hang indefinitely and wedge the
// reconnect sync loop / stall writes (F-MED-7). Generous enough for normal API
// calls; callers needing longer (or none) can pass their own signal.
const REQUEST_TIMEOUT_MS = 30_000;

function requestTimeoutSignal(): AbortSignal | undefined {
	if (typeof AbortSignal !== 'undefined' && typeof AbortSignal.timeout === 'function') {
		return AbortSignal.timeout(REQUEST_TIMEOUT_MS);
	}
	return undefined;
}

// JSON headers plus an optional Idempotency-Key so a retried offline-sync create
// (whose original response was lost) is deduped server-side instead of creating a
// duplicate workout/exercise (F-HIGH-3).
function idempotencyHeaders(idempotencyKey?: string): Record<string, string> {
	const headers: Record<string, string> = { 'Content-Type': 'application/json' };
	if (idempotencyKey) headers['Idempotency-Key'] = idempotencyKey;
	return headers;
}

function withCredentials(init: RequestInit | undefined): RequestInit {
	const base: RequestInit = { credentials: 'include', ...init };
	if (!base.signal) {
		const signal = requestTimeoutSignal();
		if (signal) base.signal = signal;
	}
	return base;
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
	const text = await response.text().catch(() => '');

	if (!contentType.includes('application/json')) {
		// A 2xx carrying a non-empty, non-JSON body (e.g. an intercepting proxy's
		// HTML page) never reached the backend as a real write. Reject it instead of
		// reporting a successful no-payload write, so callers like the reconnect sync
		// don't delete their retry copy after a write that didn't actually happen
		// (F-LOW-3). A genuinely empty body stays an acceptable void success.
		if (text.trim()) {
			throw new ApiError(
				response.status,
				`Unexpected non-JSON response (content-type: ${contentType || 'none'})`
			);
		}
		return undefined as T;
	}

	// Defensive: some endpoints may return an empty body with 200/201.
	if (!text.trim()) return undefined as T;
	return JSON.parse(text) as T;
}

export async function createWorkout(
	workout: CreateWorkoutRequest,
	fetcher: Fetcher = fetch,
	idempotencyKey?: string
): Promise<{ id: number }> {
	const response = await fetcher(
		`${API_BASE}/api/workouts`,
		withCredentials({
			method: 'POST',
			headers: idempotencyHeaders(idempotencyKey),
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
	fetcher: Fetcher = fetch,
	idempotencyKey?: string
): Promise<{ id: number }> {
	const response = await fetcher(
		`${API_BASE}/api/workouts/${workoutId}/exercises`,
		withCredentials({
			method: 'POST',
			headers: idempotencyHeaders(idempotencyKey),
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
): Promise<Set> {
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

export async function getWorkoutTemplates(fetcher: Fetcher = fetch): Promise<WorkoutTemplate[]> {
	const response = await fetcher(`${API_BASE}/api/templates`, withCredentials(undefined));
	return handleResponse(response);
}

export async function getWorkoutTemplate(
	id: number,
	fetcher: Fetcher = fetch
): Promise<WorkoutTemplateDetail> {
	const response = await fetcher(`${API_BASE}/api/templates/${id}`, withCredentials(undefined));
	return handleResponse(response);
}

export async function createWorkoutTemplate(
	template: CreateWorkoutTemplateRequest,
	fetcher: Fetcher = fetch
): Promise<WorkoutTemplateDetail> {
	const response = await fetcher(
		`${API_BASE}/api/templates`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(template)
		})
	);
	return handleResponse(response);
}

export async function updateWorkoutTemplate(
	id: number,
	template: UpdateWorkoutTemplateRequest,
	fetcher: Fetcher = fetch
): Promise<WorkoutTemplateDetail> {
	const response = await fetcher(
		`${API_BASE}/api/templates/${id}`,
		withCredentials({
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(template)
		})
	);
	return handleResponse(response);
}

export async function deleteWorkoutTemplate(id: number, fetcher: Fetcher = fetch): Promise<void> {
	const response = await fetcher(
		`${API_BASE}/api/templates/${id}`,
		withCredentials({
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json'
			}
		})
	);
	return handleResponse(response);
}

export async function duplicateWorkoutTemplate(
	id: number,
	payload: DuplicateWorkoutTemplateRequest,
	fetcher: Fetcher = fetch
): Promise<WorkoutTemplateDetail> {
	const response = await fetcher(
		`${API_BASE}/api/templates/${id}/duplicate`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload)
		})
	);
	return handleResponse(response);
}

export async function createWorkoutTemplateFromWorkout(
	workoutId: number,
	payload: CreateWorkoutTemplateFromWorkoutRequest,
	fetcher: Fetcher = fetch
): Promise<WorkoutTemplateDetail> {
	const response = await fetcher(
		`${API_BASE}/api/workouts/${workoutId}/template`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload)
		})
	);
	return handleResponse(response);
}

export async function startWorkoutFromTemplate(
	id: number,
	payload: StartWorkoutFromTemplateRequest,
	fetcher: Fetcher = fetch
): Promise<{ id: number }> {
	const response = await fetcher(
		`${API_BASE}/api/templates/${id}/start`,
		withCredentials({
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload)
		})
	);
	return handleResponse(response);
}

export async function getExerciseTypes(fetcher: Fetcher = fetch): Promise<string[]> {
	const response = await fetcher(`${API_BASE}/api/exercises/types`, withCredentials(undefined));
	return handleResponse(response);
}

export async function getLastExerciseData(
	exerciseType: string,
	options: { excludeWorkoutId?: number } = {},
	fetcher: Fetcher = fetch
): Promise<{ exercise: Exercise; sets: Set[] } | null> {
	const query =
		options.excludeWorkoutId != null ? `?exclude_workout_id=${options.excludeWorkoutId}` : '';
	const response = await fetcher(
		`${API_BASE}/api/exercises/last/${encodeURIComponent(exerciseType)}${query}`,
		withCredentials(undefined)
	);
	const data = await handleResponse<{ exercise: Exercise; sets: Set[] }>(response);
	if (!data) return null;
	return { exercise: data.exercise, sets: data.sets };
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

export async function rotateMcpToken(
	id: number,
	fetcher: Fetcher = fetch
): Promise<CreatedMcpToken> {
	const response = await fetcher(
		`${API_BASE}/api/mcp/tokens/${id}/rotate`,
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

export async function getProgressOverview(fetcher: Fetcher = fetch): Promise<ProgressOverview> {
	const timezoneOffsetMinutes = new Date().getTimezoneOffset();
	const response = await fetcher(
		`${API_BASE}/api/progress/overview?timezone_offset_minutes=${encodeURIComponent(
			timezoneOffsetMinutes
		)}`,
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
	options: { excludeWorkoutId?: number } = {},
	fetcher: Fetcher = fetch
): Promise<VolumeStats> {
	const exclude =
		options.excludeWorkoutId != null ? `&exclude_workout_id=${options.excludeWorkoutId}` : '';
	const response = await fetcher(
		`${API_BASE}/api/progress/volume?exercise_type=${encodeURIComponent(exerciseType)}${exclude}`,
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
