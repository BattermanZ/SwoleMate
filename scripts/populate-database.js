const sqlite3 = require('sqlite3').verbose();
const path = require('path');
const { format } = require('date-fns');

// Exercise definitions with starting weights and progression
const exercises = [
    { name: 'Bench Press', startWeight: 60, progression: 1.25 },
    { name: 'Squat', startWeight: 70, progression: 2.5 },
    { name: 'Deadlift', startWeight: 80, progression: 2.5 },
    { name: 'Overhead Press', startWeight: 40, progression: 1 },
    { name: 'Barbell Row', startWeight: 50, progression: 1.25 },
    { name: 'Pull-ups', startWeight: 0, progression: 0 }, // Bodyweight
    { name: 'Dips', startWeight: 0, progression: 2.5 }, // Start bodyweight, then add weight
    { name: 'Romanian Deadlift', startWeight: 60, progression: 2 },
    { name: 'Incline Bench Press', startWeight: 50, progression: 1.25 },
    { name: 'Front Squat', startWeight: 50, progression: 2 }
];

// Function to generate random number between min and max
const random = (min, max) => Math.floor(Math.random() * (max - min + 1)) + min;

// Function to generate workout dates between start and end date
function generateWorkoutDates(startDate, endDate, totalWorkouts) {
    const dates = [];
    const daysBetween = (endDate - startDate) / (1000 * 60 * 60 * 24);
    const averageDaysBetween = daysBetween / totalWorkouts;
    
    let currentDate = new Date(startDate);
    for (let i = 0; i < totalWorkouts; i++) {
        // Add some randomness to the days between workouts (1-3 days variation)
        const daysToAdd = averageDaysBetween + random(-1, 1);
        currentDate = new Date(currentDate.getTime() + (daysToAdd * 24 * 60 * 60 * 1000));
        if (currentDate <= endDate) {
            dates.push(new Date(currentDate));
        }
    }
    return dates;
}

// Function to generate a workout session
function generateWorkout(db, date, sessionNumber) {
    return new Promise((resolve, reject) => {
        const startTime = new Date(date);
        startTime.setHours(random(7, 20), random(0, 59), 0);
        const endTime = new Date(startTime.getTime() + random(45, 90) * 60 * 1000);
        
        const workoutSql = `
            INSERT INTO workouts (date, start_time, end_time, notes, feedback)
            VALUES (?, ?, ?, ?, ?)
        `;
        
        const feedbacks = ['😊', '😐', '😞'];
        const feedback = feedbacks[random(0, 2)];
        const notes = random(1, 10) > 8 ? 'Great session!' : null; // 20% chance of having notes

        // Format dates in RFC3339/ISO8601 format with UTC timezone
        const formattedDate = date.toISOString();
        const formattedStartTime = startTime.toISOString();
        const formattedEndTime = endTime.toISOString();

        db.run(workoutSql, [
            formattedDate,
            formattedStartTime,
            formattedEndTime,
            notes,
            feedback
        ], async function(err) {
            if (err) {
                reject(err);
                return;
            }

            const workoutId = this.lastID;
            const numExercises = random(5, 7);
            
            // Select random exercises for this workout
            const shuffled = [...exercises].sort(() => 0.5 - Math.random());
            const selectedExercises = shuffled.slice(0, numExercises);
            
            try {
                // Generate exercises and their sets sequentially
                for (let i = 0; i < selectedExercises.length; i++) {
                    await generateExercise(db, workoutId, selectedExercises[i], startTime, i, sessionNumber);
                }
                resolve(workoutId);
            } catch (error) {
                reject(error);
            }
        });
    });
}

// Function to generate an exercise with sets
function generateExercise(db, workoutId, exercise, startTime, exerciseIndex, sessionNumber) {
    return new Promise((resolve, reject) => {
        const exerciseStartTime = new Date(startTime.getTime() + exerciseIndex * 15 * 60 * 1000);
        const exerciseEndTime = new Date(exerciseStartTime.getTime() + random(10, 20) * 60 * 1000);
        
        const exerciseSql = `
            INSERT INTO exercises (workout_id, exercise_type, start_time, end_time, notes)
            VALUES (?, ?, ?, ?, ?)
        `;
        
        // Format dates in RFC3339/ISO8601 format with UTC timezone
        const formattedStartTime = exerciseStartTime.toISOString();
        const formattedEndTime = exerciseEndTime.toISOString();
        
        db.run(exerciseSql, [
            workoutId,
            exercise.name,
            formattedStartTime,
            formattedEndTime,
            null
        ], async function(err) {
            if (err) {
                reject(err);
                return;
            }

            const exerciseId = this.lastID;
            const numSets = random(3, 5); // Randomize number of sets
            
            try {
                // Calculate weight progression
                const baseWeight = exercise.startWeight + (exercise.progression * Math.floor(sessionNumber / 2));
                
                // Generate sets sequentially
                for (let setNum = 1; setNum <= numSets; setNum++) {
                    const weight = baseWeight + random(-2, 2); // Small weight variation
                    const reps = random(8, 12);
                    
                    await new Promise((resolve, reject) => {
                        const setSql = `
                            INSERT INTO sets (exercise_id, reps, weight, notes)
                            VALUES (?, ?, ?, ?)
                        `;
                        
                        db.run(setSql, [exerciseId, reps, weight, null], (err) => {
                            if (err) reject(err);
                            else resolve();
                        });
                    });
                }
                resolve();
            } catch (error) {
                reject(error);
            }
        });
    });
}

// Main function to populate the database
async function populateDatabase() {
    const dbPath = path.join(__dirname, '..', 'server', 'database', 'swolemate.db');
    console.log('Using database at:', dbPath);
    
    const db = new sqlite3.Database(dbPath);
    
    // Initialize schema if needed
    const schemaPath = path.join(__dirname, '..', 'server', 'database', 'migrations', '20240207_initial_schema.sql');
    const schema = require('fs').readFileSync(schemaPath, 'utf8');
    
    try {
        // Run schema in a transaction
        await new Promise((resolve, reject) => {
            db.exec(schema, (err) => {
                if (err) reject(err);
                else resolve();
            });
        });
        
        const startDate = new Date('2024-11-01');
        const endDate = new Date('2025-01-31');
        const totalWorkouts = 30;
        
        const workoutDates = generateWorkoutDates(startDate, endDate, totalWorkouts);
        
        console.log(`Generating ${totalWorkouts} workouts from ${format(startDate, 'yyyy-MM-dd')} to ${format(endDate, 'yyyy-MM-dd')}`);
        
        // Generate workouts sequentially
        for (let i = 0; i < workoutDates.length; i++) {
            const workoutId = await generateWorkout(db, workoutDates[i], i);
            console.log(`Created workout #${workoutId} on ${format(workoutDates[i], 'yyyy-MM-dd')}`);
        }
        
        // Get final statistics
        const stats = await Promise.all([
            new Promise((resolve, reject) => {
                db.get('SELECT COUNT(*) as count FROM workouts', [], (err, row) => {
                    if (err) reject(err);
                    else resolve(row.count);
                });
            }),
            new Promise((resolve, reject) => {
                db.get('SELECT COUNT(*) as count FROM exercises', [], (err, row) => {
                    if (err) reject(err);
                    else resolve(row.count);
                });
            }),
            new Promise((resolve, reject) => {
                db.get('SELECT COUNT(*) as count FROM sets', [], (err, row) => {
                    if (err) reject(err);
                    else resolve(row.count);
                });
            })
        ]);
        
        console.log('\nDatabase Statistics:');
        console.log(`Total Workouts: ${stats[0]}`);
        console.log(`Total Exercises: ${stats[1]}`);
        console.log(`Total Sets: ${stats[2]}`);
        
    } catch (error) {
        console.error('Error populating database:', error);
    } finally {
        db.close();
    }
}

// Run the population script
populateDatabase(); 