CREATE TABLE interview_stages (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL,
    stage_number INTEGER NOT NULL,
    name TEXT,
    status TEXT NOT NULL DEFAULT 'SCHEDULED',
    scheduled_date TEXT NOT NULL,
    notes TEXT,
    created TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs (id) ON DELETE CASCADE,
    UNIQUE (job_id, stage_number)
);
CREATE INDEX idx_interview_stages_job_id ON interview_stages (job_id);
