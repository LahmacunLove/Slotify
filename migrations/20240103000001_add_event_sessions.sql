-- Create event_sessions table to manage the overall event flow
CREATE TABLE IF NOT EXISTS event_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    started_at DATETIME NOT NULL,
    ended_at DATETIME,
    slot_duration_minutes INTEGER NOT NULL DEFAULT 60,
    late_arrival_cutoff_hours INTEGER NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT true,
    current_dj_id TEXT,
    current_slot_started_at DATETIME,
    next_draw_at DATETIME,
    FOREIGN KEY (current_dj_id) REFERENCES djs(id)
);

-- Create index for finding active event session quickly
CREATE INDEX IF NOT EXISTS idx_event_sessions_is_active ON event_sessions(is_active);
