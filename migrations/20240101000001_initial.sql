-- Initial database schema for DJ Session Recorder

-- DJs table
CREATE TABLE djs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    registered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    weight REAL NOT NULL DEFAULT 1.0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    position_in_queue INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Sessions table
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    dj_id TEXT NOT NULL REFERENCES djs(id),
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at DATETIME,
    duration_minutes INTEGER,
    file_path TEXT,
    download_link TEXT,
    upload_status TEXT NOT NULL DEFAULT 'recording' CHECK (upload_status IN ('recording', 'processing', 'uploaded', 'failed')),
    session_type TEXT NOT NULL DEFAULT 'solo' CHECK (session_type IN ('solo', 'b2b', 'special')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Lottery draws table
CREATE TABLE lottery_draws (
    id TEXT PRIMARY KEY,
    winner_dj_id TEXT NOT NULL REFERENCES djs(id),
    drawn_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    algorithm_used TEXT NOT NULL,
    participants_data TEXT, -- JSON string of participants
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Guest requests table
CREATE TABLE guest_requests (
    id TEXT PRIMARY KEY,
    guest_name TEXT NOT NULL,
    guest_email TEXT NOT NULL,
    message TEXT,
    target_dj_id TEXT NOT NULL REFERENCES djs(id),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- System configuration table
CREATE TABLE system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better performance
CREATE INDEX idx_djs_active ON djs(is_active);
CREATE INDEX idx_djs_queue_position ON djs(position_in_queue);
CREATE INDEX idx_sessions_dj_id ON sessions(dj_id);
CREATE INDEX idx_sessions_active ON sessions(ended_at) WHERE ended_at IS NULL;
CREATE INDEX idx_lottery_draws_drawn_at ON lottery_draws(drawn_at);
CREATE INDEX idx_guest_requests_target_dj ON guest_requests(target_dj_id);
CREATE INDEX idx_guest_requests_status ON guest_requests(status);

-- Triggers for updated_at timestamps
CREATE TRIGGER update_djs_updated_at 
    AFTER UPDATE ON djs
    BEGIN
        UPDATE djs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER update_sessions_updated_at 
    AFTER UPDATE ON sessions
    BEGIN
        UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER update_guest_requests_updated_at 
    AFTER UPDATE ON guest_requests
    BEGIN
        UPDATE guest_requests SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER update_system_config_updated_at 
    AFTER UPDATE ON system_config
    BEGIN
        UPDATE system_config SET updated_at = CURRENT_TIMESTAMP WHERE key = NEW.key;
    END;