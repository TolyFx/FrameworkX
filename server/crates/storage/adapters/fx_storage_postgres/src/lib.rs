//! PostgreSQL 存储适配器。
//!
//! 职责:实现 fx_storage 的 `FileObjectStore` / `QuotaPolicy` 两个端口,
//!      对接 `file_objects` / `storage_quota` 表。
//! 边界:基础设施适配器,可依赖 sqlx;`owner` 列原样存不透明 `Scope`,存储层不解释其语义,
//!      因此 schema 不外键到任何用户表——保持与核心一致的「用户无关」。
//! 约束:`StorageObject` 在核心层无 sqlx 依赖,故此处用本地 `FromRow` 行结构做映射。

use async_trait::async_trait;
use fx_storage_service::{
    FileObjectStore, NewStorageObject, QuotaPolicy, QuotaSnapshot, Scope, StorageError,
    StorageObject,
};
use uuid::Uuid;

const DEFAULT_QUOTA: i64 = 104_857_600;

#[derive(sqlx::FromRow)]
struct FileObjectRow {
    id: i64,
    hash: String,
    storage_path: String,
    size: i64,
    mime_type: String,
    mime_category: String,
    width: Option<i32>,
    height: Option<i32>,
    duration_ms: Option<i64>,
    thumb_path: Option<String>,
    original_name: Option<String>,
    ref_count: i32,
    #[allow(dead_code)]
    owner: String,
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<FileObjectRow> for StorageObject {
    fn from(r: FileObjectRow) -> Self {
        StorageObject {
            id: r.id,
            hash: r.hash,
            storage_path: r.storage_path,
            size: r.size,
            mime_type: r.mime_type,
            mime_category: r.mime_category,
            width: r.width,
            height: r.height,
            duration_ms: r.duration_ms,
            thumb_path: r.thumb_path,
            original_name: r.original_name,
            ref_count: r.ref_count,
        }
    }
}

pub struct PgFileObjectStore {
    db: sqlx::PgPool,
}

impl PgFileObjectStore {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FileObjectStore for PgFileObjectStore {
    async fn find_by_hash(
        &self,
        scope: &Scope,
        hash: &str,
    ) -> Result<Option<StorageObject>, StorageError> {
        let owner: &str = scope;
        let row = sqlx::query_as::<_, FileObjectRow>(
            "SELECT * FROM file_objects WHERE owner = $1 AND hash = $2 AND state = 'ready'",
        )
        .bind(owner)
        .bind(hash)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(row.map(StorageObject::from))
    }

    async fn find_by_upload_id(
        &self,
        scope: &Scope,
        upload_id: Uuid,
    ) -> Result<Option<StorageObject>, StorageError> {
        let owner: &str = scope;
        let row = sqlx::query_as::<_, FileObjectRow>(
            "SELECT * FROM file_objects WHERE owner = $1 AND upload_id = $2 AND state = 'ready'",
        )
        .bind(owner)
        .bind(upload_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|error| StorageError::Store(error.to_string()))?;
        Ok(row.map(StorageObject::from))
    }

    async fn insert(
        &self,
        obj: NewStorageObject,
        scope: &Scope,
    ) -> Result<StorageObject, StorageError> {
        let owner: &str = scope;
        let row = sqlx::query_as::<_, FileObjectRow>(
            "INSERT INTO file_objects \
             (hash, storage_path, size, mime_type, mime_category, width, height, duration_ms, thumb_path, original_name, owner, upload_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *",
        )
        .bind(obj.hash.as_str())
        .bind(obj.storage_path.as_str())
        .bind(obj.size)
        .bind(obj.mime_type.as_str())
        .bind(obj.mime_category.as_str())
        .bind(obj.width)
        .bind(obj.height)
        .bind(obj.duration_ms)
        .bind(obj.thumb_path.as_deref())
        .bind(obj.original_name.as_deref())
        .bind(owner)
        .bind(obj.upload_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(row.into())
    }

    async fn get(&self, id: i64, scope: &Scope) -> Result<Option<StorageObject>, StorageError> {
        let owner: &str = scope;
        let row = sqlx::query_as::<_, FileObjectRow>(
            "SELECT * FROM file_objects WHERE id = $1 AND owner = $2 AND state = 'ready'",
        )
        .bind(id)
        .bind(owner)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(row.map(StorageObject::from))
    }

    async fn list(
        &self,
        scope: &Scope,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StorageObject>, StorageError> {
        let owner: &str = scope;
        let rows = if let Some(category) = category {
            sqlx::query_as::<_, FileObjectRow>(
                "SELECT * FROM file_objects \
                 WHERE owner = $1 AND state = 'ready' AND mime_category = $2 \
                 ORDER BY id DESC LIMIT $3 OFFSET $4",
            )
            .bind(owner)
            .bind(category)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await
        } else {
            sqlx::query_as::<_, FileObjectRow>(
                "SELECT * FROM file_objects \
                 WHERE owner = $1 AND state = 'ready' \
                 ORDER BY id DESC LIMIT $2 OFFSET $3",
            )
            .bind(owner)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await
        }
        .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(rows.into_iter().map(StorageObject::from).collect())
    }

    async fn count(&self, scope: &Scope, category: Option<&str>) -> Result<i64, StorageError> {
        let owner: &str = scope;
        let total: (i64,) = if let Some(category) = category {
            sqlx::query_as(
                "SELECT COUNT(*)::BIGINT FROM file_objects WHERE owner = $1 AND state = 'ready' AND mime_category = $2",
            )
            .bind(owner)
            .bind(category)
            .fetch_one(&self.db)
            .await
        } else {
            sqlx::query_as("SELECT COUNT(*)::BIGINT FROM file_objects WHERE owner = $1 AND state = 'ready'")
                .bind(owner)
                .fetch_one(&self.db)
                .await
        }
        .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(total.0)
    }

    async fn delete(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM file_objects WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(())
    }

    async fn inc_ref(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("UPDATE file_objects SET ref_count = ref_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(())
    }

    async fn dec_ref(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("UPDATE file_objects SET ref_count = GREATEST(ref_count - 1, 0) WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(())
    }
}

pub struct PgQuotaPolicy {
    db: sqlx::PgPool,
    base_quota: i64,
}

impl PgQuotaPolicy {
    pub fn new(db: sqlx::PgPool, base_quota: i64) -> Self {
        Self { db, base_quota }
    }

    async fn load(&self, scope: &Scope) -> Result<QuotaSnapshot, StorageError> {
        let owner: &str = scope;
        let row: Option<(i64, i64)> =
            sqlx::query_as("SELECT used_bytes, quota_bytes FROM storage_quota WHERE owner = $1")
                .bind(owner)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| StorageError::Store(e.to_string()))?;
        Ok(match row {
            Some((u, q)) => QuotaSnapshot {
                used_bytes: u,
                quota_bytes: q,
            },
            None => QuotaSnapshot {
                used_bytes: 0,
                quota_bytes: self.base_quota,
            },
        })
    }
}

#[async_trait]
impl QuotaPolicy for PgQuotaPolicy {
    async fn check(&self, scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError> {
        let snap = self.load(scope).await?;
        if snap.used_bytes + size > snap.quota_bytes {
            return Err(StorageError::QuotaExceeded {
                used_bytes: snap.used_bytes,
                quota_bytes: snap.quota_bytes,
            });
        }
        Ok(snap)
    }

    async fn consume(&self, scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError> {
        let owner: &str = scope;
        // 先确保额度行存在，再使用条件 UPDATE 原子保留额度。
        sqlx::query(
            "INSERT INTO storage_quota (owner, used_bytes, quota_bytes, updated_at) \
             VALUES ($1, 0, $2, NOW()) ON CONFLICT (owner) DO NOTHING",
        )
        .bind(owner)
        .bind(self.base_quota)
        .execute(&self.db)
        .await
        .map_err(|e| StorageError::Store(e.to_string()))?;
        let row: Option<(i64, i64)> = sqlx::query_as(
            "UPDATE storage_quota \
             SET used_bytes = used_bytes + $2, updated_at = NOW() \
             WHERE owner = $1 AND used_bytes + $2 <= quota_bytes \
             RETURNING used_bytes, quota_bytes",
        )
        .bind(owner)
        .bind(size)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| StorageError::Store(e.to_string()))?;
        match row {
            Some((used_bytes, quota_bytes)) => Ok(QuotaSnapshot {
                used_bytes,
                quota_bytes,
            }),
            None => {
                let snapshot: QuotaSnapshot = self.load(scope).await?;
                Err(StorageError::QuotaExceeded {
                    used_bytes: snapshot.used_bytes,
                    quota_bytes: snapshot.quota_bytes,
                })
            }
        }
    }

    async fn release(&self, scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError> {
        let owner: &str = scope;
        sqlx::query(
            "UPDATE storage_quota \
             SET used_bytes = GREATEST(used_bytes - $2, 0), updated_at = NOW() \
             WHERE owner = $1",
        )
        .bind(owner)
        .bind(size)
        .execute(&self.db)
        .await
        .map_err(|e| StorageError::Store(e.to_string()))?;
        self.load(scope).await
    }

    async fn current(&self, scope: &Scope) -> Result<QuotaSnapshot, StorageError> {
        self.load(scope).await
    }
}

/// 全局默认配额常量,供宿主装配时引用
pub fn default_quota() -> i64 {
    DEFAULT_QUOTA
}
