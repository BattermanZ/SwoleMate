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