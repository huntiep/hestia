CREATE TABLE IF NOT EXISTS reminders (
    id INTEGER PRIMARY KEY,
    owner INTEGER REFERENCES users (id) ON DELETE CASCADE,
    recurrence INTEGER NOT NULL,
    reason TEXT,
    date DATE NOT NULL
);
