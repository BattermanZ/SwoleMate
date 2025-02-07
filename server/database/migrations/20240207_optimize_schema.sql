-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Begin transaction to ensure all changes are atomic
BEGIN TRANSACTION;

-- Create temporary tables with new schema
CREATE TABLE workouts_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATETIME NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    notes TEXT,
    feedback TEXT CHECK(feedback IN ('😊', '😐', '😞') OR feedback IS NULL)
);

CREATE TABLE exercises_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_id INTEGER NOT NULL,
    exercise_type TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    notes TEXT,
    FOREIGN KEY (workout_id) 
        REFERENCES workouts(id) 
        ON DELETE CASCADE
);

CREATE TABLE sets_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exercise_id INTEGER NOT NULL,
    reps INTEGER NOT NULL,
    weight REAL NOT NULL,
    notes TEXT,
    FOREIGN KEY (exercise_id) 
        REFERENCES exercises(id) 
        ON DELETE CASCADE
);

-- Copy existing data
INSERT INTO workouts_new 
SELECT id, date, start_time, end_time, notes, feedback 
FROM workouts;

INSERT INTO exercises_new 
SELECT id, workout_id, exercise_type, start_time, end_time, notes 
FROM exercises;

INSERT INTO sets_new 
SELECT id, exercise_id, reps, weight, notes 
FROM sets;

-- Drop old tables
DROP TABLE sets;
DROP TABLE exercises;
DROP TABLE workouts;

-- Rename new tables to original names
ALTER TABLE workouts_new RENAME TO workouts;
ALTER TABLE exercises_new RENAME TO exercises;
ALTER TABLE sets_new RENAME TO sets;

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_workouts_date ON workouts(date);
CREATE INDEX IF NOT EXISTS idx_exercises_workout_id ON exercises(workout_id);
CREATE INDEX IF NOT EXISTS idx_exercises_type ON exercises(exercise_type);
CREATE INDEX IF NOT EXISTS idx_sets_exercise_id ON sets(exercise_id);

-- Verify data integrity
SELECT CASE 
    WHEN (SELECT COUNT(*) FROM workouts) = (SELECT COUNT(*) FROM workouts_new) AND
         (SELECT COUNT(*) FROM exercises) = (SELECT COUNT(*) FROM exercises_new) AND
         (SELECT COUNT(*) FROM sets) = (SELECT COUNT(*) FROM sets_new)
    THEN 'Data migration successful'
    ELSE RAISE(ROLLBACK, 'Data migration failed')
END;

-- Commit transaction
COMMIT; 