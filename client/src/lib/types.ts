export type FeedbackEmoji = '😊' | '😐' | '😞';

export interface Workout {
	id?: number;
	date: string; // ISO date string
	start_time: string; // ISO date string
	end_time: string; // ISO date string
	notes?: string | null;
	feedback?: FeedbackEmoji | null;
	auto_closed_at?: string | null;
	exercise_count?: number;
	timezone_offset_minutes?: number | null;
}

export interface Exercise {
	id?: number;
	workout_id: number;
	exercise_type: string;
	start_time: string; // ISO date string
	end_time: string; // ISO date string
	notes?: string | null;
	per_side_weight?: boolean;
	split_weight?: boolean;
	settings?: ExerciseSetting[];
}

export interface ExerciseSetting {
	id?: number;
	exercise_id: number;
	key: string;
	value: string;
}

export interface CreateWorkoutRequest {
	date: string; // ISO date string
	start_time: string; // ISO date string
	notes?: string | null;
	timezone_offset_minutes?: number | null;
}

export interface UpdateWorkoutRequest {
	end_time: string; // ISO date string
	notes?: string | null;
	feedback?: FeedbackEmoji | null;
}

export interface UpdateWorkoutTimesRequest {
	start_time: string; // ISO date string
	end_time: string; // ISO date string
	notes?: string | null;
	feedback?: FeedbackEmoji | null;
}

export interface CreateExerciseRequest {
	exercise_type: string;
	start_time: string; // ISO date string
	notes?: string | null;
	per_side_weight?: boolean;
	split_weight?: boolean;
	settings?: Array<Pick<ExerciseSetting, 'key' | 'value'>>;
}

export interface UpdateExerciseRequest {
	end_time: string; // ISO date string
	notes?: string | null;
	per_side_weight?: boolean;
	split_weight?: boolean;
	settings?: Array<Pick<ExerciseSetting, 'key' | 'value'>>;
}

export interface Set {
	id?: number;
	exercise_id: number;
	reps: number;
	weight: number;
	weight_left?: number;
	weight_right?: number;
	duration_seconds?: number;
	notes?: string | null;
}

export interface CreateSetRequest {
	reps: number;
	weight: number;
	duration_seconds?: number;
	notes?: string | null;
	weight_left?: number;
	weight_right?: number;
}

export interface WorkoutWithExercises extends Workout {
	exercises: Array<{
		exercise: Exercise;
		sets: Set[];
	}>;
}

export interface WorkoutTemplate {
	id: number;
	name: string;
	exercise_count: number;
	created_at: string;
	updated_at: string;
}

export interface WorkoutTemplateExerciseSetting {
	id: number;
	template_exercise_id: number;
	key: string;
	value: string;
}

export interface WorkoutTemplateExercise {
	id: number;
	template_id: number;
	position: number;
	exercise_type: string;
	notes?: string | null;
	per_side_weight?: boolean;
	split_weight?: boolean;
	settings?: WorkoutTemplateExerciseSetting[];
}

export interface WorkoutTemplateDetail {
	template: WorkoutTemplate;
	exercises: WorkoutTemplateExercise[];
}

export interface WorkoutTemplateExerciseInput {
	exercise_type: string;
	notes?: string | null;
	per_side_weight?: boolean;
	split_weight?: boolean;
	settings?: Array<Pick<WorkoutTemplateExerciseSetting, 'key' | 'value'>>;
}

export interface CreateWorkoutTemplateRequest {
	name: string;
	exercises: WorkoutTemplateExerciseInput[];
}

export interface UpdateWorkoutTemplateRequest {
	name: string;
	exercises: WorkoutTemplateExerciseInput[];
}

export interface DuplicateWorkoutTemplateRequest {
	name?: string | null;
}

export interface CreateWorkoutTemplateFromWorkoutRequest {
	name: string;
}

export interface StartWorkoutFromTemplateRequest {
	date: string;
	start_time: string;
	timezone_offset_minutes?: number | null;
}

export interface WorkoutStats {
	total_workouts: number;
	average_duration_minutes: number;
	feedback_distribution: {
		good: number;
		neutral: number;
		bad: number;
	};
	workout_frequency: {
		average_per_week: number;
		trend?: number; // Change from last 4 weeks
	};
	duration_trend?: number; // Change from last 4 weeks
	popular_hours: Array<{
		hour: string;
		count: number;
	}>;
	duration_distribution: Array<{
		range: string;
		count: number;
	}>;
	sessions_per_month?: Array<{
		month: string; // YYYY-MM
		count: number;
	}>;
	avg_exercise_duration_series?: Array<{
		start_time: string;
		end_time: string;
		duration_minutes: number;
		exercise_count: number;
		avg_minutes: number;
	}>;
	session_start_times?: string[];
	session_start_samples?: Array<{
		start_time: string;
		timezone_offset_minutes?: number | null;
	}>;
}

export interface VolumeStats {
	weekly_volume: Array<{
		week: string;
		total_volume: number;
		max_weight: number;
		total_reps: number;
		total_sets: number;
		max_estimated_1rm: number;
		set_schemes?: string[];
	}>;
	monthly_volume: Array<{
		month: string;
		total_volume: number;
		max_weight: number;
		total_reps: number;
		total_sets: number;
	}>;
	personal_records: {
		all_time_max_weight: number;
		max_volume: number;
		estimated_max_1rm: number;
		rep_prs?: Array<{
			reps: number;
			weight: number;
		}>;
	};
	timed_records?: TimedRecords | null;
}

export interface ExerciseProgress {
	exercise: Exercise;
	sets: Set[];
}

export interface TimedRecords {
	longest_set_seconds: number;
	best_session_duration_seconds: number;
	lifetime_duration_seconds: number;
	average_set_duration_seconds: number;
}

export interface ProgressPeriodComparison {
	workouts_delta: number;
	total_training_minutes_delta: number;
	exercises_delta: number;
	sets_delta: number;
	reps_delta: number;
	total_volume_delta: number;
	timed_sets_delta: number;
	total_timed_duration_seconds_delta: number;
	pr_count_delta: number;
	recent_best_count_delta: number;
}

export interface ProgressPeriodSummary {
	label: string;
	start_date: string;
	end_date: string;
	workouts: number;
	total_training_minutes: number;
	exercises: number;
	sets: number;
	reps: number;
	total_volume: number;
	timed_sets: number;
	total_timed_duration_seconds: number;
	pr_count: number;
	recent_best_count: number;
	comparison: ProgressPeriodComparison;
}

export type ProgressPrType =
	| 'max_weight'
	| 'estimated_1rm'
	| 'rep_pr'
	| 'timed_duration'
	| 'single_set_volume';

export interface RecentPr {
	exercise_type: string;
	pr_type: ProgressPrType;
	new_value: number;
	previous_value: number;
	date: string;
	set_id: number;
	set_details: {
		reps: number;
		weight: number;
		duration_seconds?: number | null;
	};
}

export interface ProgressOverview {
	last_7_days: ProgressPeriodSummary;
	last_30_days: ProgressPeriodSummary;
	recent_prs: RecentPr[];
	recent_bests: RecentPr[];
}
