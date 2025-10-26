-- Add Session-Recorder integration columns to sessions table

ALTER TABLE sessions ADD COLUMN recorder_session_id TEXT;
ALTER TABLE sessions ADD COLUMN recorder_id TEXT;
ALTER TABLE sessions ADD COLUMN recorder_ogg_url TEXT;
ALTER TABLE sessions ADD COLUMN recorder_flac_url TEXT;
ALTER TABLE sessions ADD COLUMN recorder_waveform_url TEXT;

-- Add indexes for better performance
CREATE INDEX idx_sessions_recorder_session_id ON sessions(recorder_session_id);
CREATE INDEX idx_sessions_recorder_id ON sessions(recorder_id);