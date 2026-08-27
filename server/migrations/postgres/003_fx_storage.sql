-- 存储基线表:文件对象按 owner 隔离,仅在同一归属内按 SHA-256 去重。
-- owner 为不透明归属标识,宿主约定其语义(如 "user:42"),存储层不解释、不外键到任何用户表。
-- 配额(quota)属于策略端口 QuotaPolicy,实现方式由宿主决定,不在此基线;见 004_user_storage.sql。
CREATE TABLE IF NOT EXISTS file_objects (
    id            BIGSERIAL    PRIMARY KEY,
    hash          CHAR(64)     NOT NULL,
    storage_path  VARCHAR(500) NOT NULL,
    size          BIGINT       NOT NULL CHECK (size >= 0),
    mime_type     VARCHAR(100) NOT NULL,
    mime_category VARCHAR(20)  NOT NULL,
    width         INT          CHECK (width IS NULL OR width > 0),
    height        INT          CHECK (height IS NULL OR height > 0),
    duration_ms   BIGINT       CHECK (duration_ms IS NULL OR duration_ms >= 0),
    thumb_path    VARCHAR(500),
    original_name VARCHAR(255),
    ref_count     INT          NOT NULL DEFAULT 1 CHECK (ref_count >= 0),
    owner         VARCHAR(64)  NOT NULL,
    upload_id     UUID,
    state         VARCHAR(16)  NOT NULL DEFAULT 'ready'
                                 CHECK (state IN ('ready', 'deleting')),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (owner, hash)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_file_objects_owner_upload_id
    ON file_objects(owner, upload_id)
    WHERE upload_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_file_objects_owner_state
    ON file_objects(owner, state, id DESC);
CREATE INDEX IF NOT EXISTS idx_file_objects_owner_state_category
    ON file_objects(owner, state, mime_category, id DESC);
