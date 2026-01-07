export type FeedbackEmoji = '😊' | '😐' | '😞';

export interface Workout {
	id?: number;
	date: string; // ISO date string
	start_time: string; // ISO date string
	end_time: string; // ISO date string
	notes?: string | null;
	feedback?: FeedbackEmoji | null;
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
}

export interface UpdateWorkoutRequest {
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
	notes?: string | null;
}

export interface CreateSetRequest {
	reps: number;
	weight: number;
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
}

export interface ExerciseProgress {
	exercise: Exercise;
	sets: Set[];
}
