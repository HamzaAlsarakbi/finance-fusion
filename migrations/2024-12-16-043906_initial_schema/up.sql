-- Your SQL goes here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL UNIQUE,
    pw_hash TEXT NOT NULL,
    two_fa_secret TEXT DEFAULT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW (),
    is_dev_mode BOOLEAN NOT NULL DEFAULT FALSE,
    -- Lockout policy
    invalid_login_attempts INTEGER NOT NULL DEFAULT 0 CHECK (invalid_login_attempts >= 0),
    lock_duration_s INTEGER NOT NULL DEFAULT 60,
    lock_duration_factor INTEGER NOT NULL DEFAULT 2,
    lock_duration_cap_s INTEGER NOT NULL DEFAULT 3600,
    locked_until TIMESTAMP DEFAULT NULL
);

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE plans (
    name VARCHAR(64) PRIMARY KEY NOT NULL,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_modified TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    type VARCHAR(64) NOT NULL DEFAULT 'info',
    plan_name VARCHAR(64) NOT NULL REFERENCES plans(name) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    status VARCHAR(64) NOT NULL DEFAULT 'unread'
);

CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    icon TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    plan_name VARCHAR(64) NOT NULL REFERENCES plans(name) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    balance DECIMAL(10, 2) NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL,
    savings_type VARCHAR(64) DEFAULT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE currencies (
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code VARCHAR(3) PRIMARY KEY NOT NULL,
    name VARCHAR(64) NOT NULL
);

CREATE TABLE budgets (
    id SERIAL PRIMARY KEY,
    plan_name VARCHAR(64) NOT NULL REFERENCES plans(name) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    amount DECIMAL(10, 2) NOT NULL,
    interval VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL REFERENCES currencies(code),
    start_date DATE NOT NULL,
    end_date DATE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    plan_name VARCHAR(64) NOT NULL REFERENCES plans(name) ON DELETE CASCADE,
    type VARCHAR(64) NOT NULL,
    from_account INT REFERENCES accounts(id) ON DELETE CASCADE,
    to_account INT REFERENCES accounts(id) ON DELETE CASCADE,
    amount DECIMAL(10, 2) NOT NULL,
    currency VARCHAR(3) NOT NULL REFERENCES currencies(code),
    statement TEXT,
    is_cancelled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE automations (
    id SERIAL PRIMARY KEY,
    plan_name VARCHAR(64) NOT NULL REFERENCES plans(name) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    type VARCHAR(64) NOT NULL,
    from_account INT REFERENCES accounts(id) ON DELETE CASCADE,
    to_account INT REFERENCES accounts(id) ON DELETE CASCADE,
    amount DECIMAL(10, 2) NOT NULL,
    currency VARCHAR(3) NOT NULL REFERENCES currencies(code),
    statement TEXT,
    frequency VARCHAR(64) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    is_paused BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE account_tags (
    account_id INT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    tag_id INT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (account_id, tag_id)
);

CREATE TABLE transaction_tags (
    transaction_id INT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    tag_id INT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (transaction_id, tag_id)
);
