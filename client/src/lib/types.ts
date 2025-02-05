export interface Workout {
    id?: number;
    date: string;  // ISO date string
    notes?: string;
}

export interface Exercise {
    id?: number;
    workout_id: number;
    exercise_type: string;
    notes?: string;
}

export interface Set {
    id?: number;
    exercise_id: number;
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