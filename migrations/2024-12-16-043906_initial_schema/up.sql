-- Your SQL goes here

CREATE TABLE
    users (
        id SERIAL PRIMARY KEY,
        username VARCHAR(64) NOT NULL UNIQUE,
        pw_hash TEXT NOT NULL,
        two_fa_secret TEXT DEFAULT NULL,
        created_at TIMESTAMP NOT NULL DEFAULT NOW (),
        -- Lockout policy
        invalid_login_attempts INTEGER NOT NULL DEFAULT 0 CHECK (invalid_login_attempts >= 0),
        lock_duration_s INTEGER NOT NULL DEFAULT 60,
        lock_duration_factor INTEGER NOT NULL DEFAULT 2,
        lock_duration_cap INTEGER NOT NULL DEFAULT 3600,
        locked_until TIMESTAMP DEFAULT NULL
    );

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    session_token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);