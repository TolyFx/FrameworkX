CREATE TABLE IF NOT EXISTS scan_sessions (
    token       VARCHAR(36) PRIMARY KEY,
    status      SMALLINT NOT NULL DEFAULT 0,
    user_id     BIGINT REFERENCES accounts(id),
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
