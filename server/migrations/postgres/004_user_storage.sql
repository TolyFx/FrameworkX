-- 用户配额层:宿主 QuotaPolicy 实现(UserQuotaPolicy)所用的配额表 + accounts 绑定。
-- 配额是策略端口,实现方式由宿主决定;本表只是 simple server 选择的「按 owner 配额」实现,
-- 不属于可复用存储基线(003)。换宿主可换配额方案,003 不受影响。
-- owner 约定 'user:' || accounts.id,须与 server/src/storage/routes.rs 的 scope 编码保持一致。

-- 归属配额表(owner 不透明,默认 100MB)
CREATE TABLE IF NOT EXISTS storage_quota (
    owner       VARCHAR(64) PRIMARY KEY,
    used_bytes  BIGINT       NOT NULL DEFAULT 0,
    quota_bytes BIGINT       NOT NULL DEFAULT 104857600,
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- 为已有账号补建配额记录(quota_bytes 走表默认 100MB)
INSERT INTO storage_quota (owner, used_bytes, updated_at)
SELECT 'user:' || a.id::text, 0, NOW()
FROM accounts a
WHERE NOT EXISTS (
    SELECT 1 FROM storage_quota q WHERE q.owner = ('user:' || a.id::text)
)
ON CONFLICT (owner) DO NOTHING;
