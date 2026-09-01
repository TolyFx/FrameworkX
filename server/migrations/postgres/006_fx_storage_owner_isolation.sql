-- 将早期全局 hash 去重的 file_objects 升级为 owner 内去重。
-- 本迁移同时兼容：
-- 1. 已执行旧版 003（VARCHAR(40)、hash 全局唯一）的历史数据库；
-- 2. 已执行新版 003（CHAR(64)、owner + hash 唯一）的数据库。

ALTER TABLE file_objects
    ADD COLUMN IF NOT EXISTS upload_id UUID,
    ADD COLUMN IF NOT EXISTS state VARCHAR(16) NOT NULL DEFAULT 'ready';

ALTER TABLE file_objects
    ALTER COLUMN hash TYPE VARCHAR(64) USING BTRIM(hash),
    ALTER COLUMN size SET NOT NULL,
    ALTER COLUMN ref_count SET DEFAULT 1,
    ALTER COLUMN ref_count SET NOT NULL,
    ALTER COLUMN state SET DEFAULT 'ready',
    ALTER COLUMN state SET NOT NULL;

ALTER TABLE file_objects
    DROP CONSTRAINT IF EXISTS file_objects_hash_key;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'file_objects'::regclass
          AND conname = 'file_objects_owner_hash_key'
    ) THEN
        ALTER TABLE file_objects
            ADD CONSTRAINT file_objects_owner_hash_key UNIQUE (owner, hash);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'file_objects'::regclass
          AND conname = 'file_objects_size_check'
    ) THEN
        ALTER TABLE file_objects
            ADD CONSTRAINT file_objects_size_check CHECK (size >= 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'file_objects'::regclass
          AND conname = 'file_objects_width_check'
    ) THEN
        ALTER TABLE file_objects
            ADD CONSTRAINT file_objects_width_check CHECK (width IS NULL OR width > 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'file_objects'::regclass
          AND conname = 'file_objects_height_check'
    ) THEN
        ALTER TABLE file_objects
            ADD CONSTRAINT file_objects_height_check CHECK (height IS NULL OR height > 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'file_objects'::regclass
          AND conname = 'file_objects_duration_ms_check'
    ) THEN
        ALTER TABLE file_objects
            ADD CONSTRAINT file_objects_duration_ms_check
            CHECK (duration_ms IS NULL OR duration_ms >= 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'file_objects'::regclass
          AND conname = 'file_objects_ref_count_check'
    ) THEN
        ALTER TABLE file_objects
            ADD CONSTRAINT file_objects_ref_count_check CHECK (ref_count >= 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'file_objects'::regclass
          AND conname = 'file_objects_state_check'
    ) THEN
        ALTER TABLE file_objects
            ADD CONSTRAINT file_objects_state_check
            CHECK (state IN ('ready', 'deleting'));
    END IF;
END $$;

DROP INDEX IF EXISTS idx_file_objects_hash;
DROP INDEX IF EXISTS idx_file_objects_owner;

CREATE UNIQUE INDEX IF NOT EXISTS uq_file_objects_owner_upload_id
    ON file_objects(owner, upload_id)
    WHERE upload_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_file_objects_owner_state
    ON file_objects(owner, state, id DESC);
CREATE INDEX IF NOT EXISTS idx_file_objects_owner_state_category
    ON file_objects(owner, state, mime_category, id DESC);
