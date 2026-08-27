CREATE TABLE accounts (
    id         BIGSERIAL    PRIMARY KEY,
    status     SMALLINT     NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_accounts_status ON accounts(status);

CREATE TABLE user_profiles (
    account_id BIGINT       PRIMARY KEY REFERENCES accounts(id),
    nickname   VARCHAR(50)  NOT NULL,
    avatar     VARCHAR(500),
    signature  VARCHAR(100) DEFAULT '',
    bio        VARCHAR(200),
    gender     SMALLINT     DEFAULT 0,
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_profiles_nickname ON user_profiles(nickname);

CREATE TABLE auth_credentials (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id BIGINT       NOT NULL REFERENCES accounts(id),
    auth_type  VARCHAR(20)  NOT NULL,
    identifier VARCHAR(100) NOT NULL,
    credential VARCHAR(255),
    verified   BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    UNIQUE(auth_type, identifier)
);

CREATE INDEX idx_auth_credentials_account
    ON auth_credentials(account_id);

CREATE TABLE login_logs (
    id          BIGSERIAL    PRIMARY KEY,
    account_id  BIGINT       NOT NULL REFERENCES accounts(id),
    login_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    ip          VARCHAR(45),
    platform    VARCHAR(20),
    device_name VARCHAR(100),
    device_id   VARCHAR(100),
    app_version VARCHAR(20)
);

CREATE INDEX idx_login_logs_account ON login_logs(account_id);
CREATE INDEX idx_login_logs_time ON login_logs(login_at DESC);

CREATE TABLE IF NOT EXISTS verify_codes (
    id          BIGSERIAL PRIMARY KEY,
    identifier  VARCHAR(255) NOT NULL,
    channel     VARCHAR(10) NOT NULL,
    scene       VARCHAR(20) NOT NULL DEFAULT 'login',
    code        VARCHAR(6) NOT NULL,
    status      SMALLINT NOT NULL DEFAULT 0,
    expires_at  TIMESTAMPTZ NOT NULL,
    request_ip  VARCHAR(45),
    sender      VARCHAR(255),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_verify_codes_lookup
    ON verify_codes(identifier, channel, scene, status, created_at DESC);

CREATE TABLE IF NOT EXISTS scan_sessions (
    token       VARCHAR(36) PRIMARY KEY,
    status      SMALLINT NOT NULL DEFAULT 0,
    user_id     BIGINT REFERENCES accounts(id),
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
