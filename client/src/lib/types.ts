export type FeedbackEmoji = '😊' | '😐' | '😞';

export interface Workout {
    id?: number;
    date: string;  // ISO date string
    start_time: string;  // ISO date string
    end_time: string;  // ISO date string
    notes?: string;
    feedback?: FeedbackEmoji;
}

export interface Exercise {
    id?: number;
    workout_id: number;
    exercise_type: string;
    start_time: string;  // ISO date string
    end_time: string;  // ISO date string
    notes?: string;
}

export interface CreateWorkoutRequest {
    date: string;  // ISO date string
    start_time: string;  // ISO date string
    notes?: string;
}

export interface UpdateWorkoutRequest {
    end_time: string;  // ISO date string
    notes?: string;
    feedback?: FeedbackEmoji;
}

export interface CreateExerciseRequest {
    exercise_type: string;
    start_time: string;  // ISO date string
    notes?: string;
}

export interface UpdateExerciseRequest {
    end_time: string;  // ISO date string
    notes?: string;
}

export interface Set {
    id?: number;
    exercise_id: number;
    reps: number;
    weight: number;
    notes?: string;
}

export interface CreateSetRequest {
    reps: number;
    weight: number;
    notes?: string;
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
}

export interface VolumeStats {
    weekly_volume: Array<{
        week: string;
        total_volume: number;
        max_weight: number;
        total_reps: number;
        sessions: number;
    }>;
    personal_records: Array<{
        reps: number;
        weight: number;
        achieved_at: string;
    }>;
}

export interface ExerciseProgress {
    exercise: Exercise;
    sets: Set[];
} 