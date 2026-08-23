//! 提供验证码服务集成测试使用的内存存储与发送适配器。

use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use fx_auth_code::{
    ActiveVerificationCode, NewVerificationCode, VerificationCodeSender, VerificationCodeStore,
};
use fx_auth_core::AuthContext;
use fx_core::AppResult;

#[derive(Default)]
pub struct MemoryStore {
    records: Mutex<Vec<(i64, NewVerificationCode, bool)>>,
}

#[async_trait]
impl VerificationCodeStore for MemoryStore {
    async fn latest_issued_at(
        &self,
        identifier: &str,
        _request_ip: Option<&str>,
    ) -> AppResult<Option<DateTime<Utc>>> {
        Ok(self
            .records
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|(_, record, _)| record.identifier == identifier)
            .map(|(_, record, _)| record.expires_at - Duration::minutes(5)))
    }

    async fn save(&self, code: NewVerificationCode) -> AppResult<()> {
        let mut records = self.records.lock().unwrap();
        let id = records.len() as i64 + 1;
        records.push((id, code, false));
        Ok(())
    }

    async fn find_active(
        &self,
        identifier: &str,
        channel: &str,
        scene: &str,
        now: DateTime<Utc>,
    ) -> AppResult<Option<ActiveVerificationCode>> {
        Ok(self
            .records
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|(_, record, used)| {
                !used
                    && record.identifier == identifier
                    && record.channel == channel
                    && record.scene == scene
                    && record.expires_at > now
            })
            .map(|(id, record, _)| ActiveVerificationCode {
                id: *id,
                code: record.code.clone(),
            }))
    }

    async fn mark_used(&self, id: i64) -> AppResult<()> {
        if let Some((_, _, used)) = self
            .records
            .lock()
            .unwrap()
            .iter_mut()
            .find(|(record_id, _, _)| *record_id == id)
        {
            *used = true;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct MemorySender {
    sent: Mutex<Vec<(String, String, String)>>,
}

impl MemorySender {
    pub fn sent_count(&self) -> usize {
        self.sent.lock().unwrap().len()
    }
}

#[async_trait]
impl VerificationCodeSender for MemorySender {
    fn name(&self, _channel: &str) -> Option<String> {
        Some("memory".into())
    }

    async fn send(
        &self,
        channel: &str,
        identifier: &str,
        code: &str,
        _ctx: &AuthContext,
    ) -> AppResult<()> {
        self.sent
            .lock()
            .unwrap()
            .push((channel.into(), identifier.into(), code.into()));
        Ok(())
    }
}
