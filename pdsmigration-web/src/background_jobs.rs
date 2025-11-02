use crate::errors::ApiError;
use pdsmigration_common::ExportBlobsRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use utoipa::ToSchema;
use uuid::Uuid;

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    ExportBlobs,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Success,
    Error,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct JobProgress {
    #[schema(example = 0)]
    pub processed: u64,
    #[schema(example = 100)]
    pub total: Option<u64>,
    #[schema(example = 0)]
    pub percent: Option<u8>,
    #[schema(example = "starting")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobRecord {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    pub kind: JobKind,
    pub status: JobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub result: Option<JsonValue>,
    #[schema(value_type = u64, example = 1700000000)]
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = u64, example = 1700000001)]
    pub started_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = u64, example = 1700000100)]
    pub finished_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobProgress>,
}

struct RunningJob {
    handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct JobManager {
    state: Arc<RwLock<JobState>>,
}

#[derive(Default)]
struct JobState {
    records: HashMap<Uuid, JobRecord>,
    running: HashMap<Uuid, RunningJob>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(JobState::default())),
        }
    }

    pub async fn list(&self) -> Vec<JobRecord> {
        let st = self.state.read().await;
        st.records.values().cloned().collect()
    }

    pub async fn get(&self, id: Uuid) -> Option<JobRecord> {
        let st = self.state.read().await;
        st.records.get(&id).cloned()
    }

    pub async fn cancel(&self, id: Uuid) -> bool {
        let mut st = self.state.write().await;
        if let Some(running) = st.running.remove(&id) {
            running.handle.abort();
            if let Some(rec) = st.records.get_mut(&id) {
                rec.status = JobStatus::Canceled;
                rec.finished_at = Some(now_millis());
            }
            true
        } else {
            false
        }
    }

    pub async fn spawn_export_blobs(&self, request: ExportBlobsRequest) -> Result<Uuid, ApiError> {
        let id = Uuid::new_v4();
        let rec = JobRecord {
            id: id.to_string(),
            kind: JobKind::ExportBlobs,
            status: JobStatus::Queued,
            error: None,
            result: None,
            created_at: now_millis(),
            started_at: None,
            finished_at: None,
            progress: Some(JobProgress {
                processed: 0,
                total: None,
                percent: None,
                message: Some("queued".to_string()),
            }),
        };

        {
            let mut st = self.state.write().await;
            st.records.insert(id, rec);
        }

        let state = self.state.clone();
        let handle = tokio::spawn(async move {
            {
                let mut st = state.write().await;
                if let Some(r) = st.records.get_mut(&id) {
                    r.status = JobStatus::Running;
                    r.started_at = Some(now_millis());
                    if let Some(p) = r.progress.as_mut() {
                        p.message = Some("running".into());
                    }
                }
            }

            let result = pdsmigration_common::export_blobs_api(request).await;

            match result {
                Ok(res) => {
                    let mut st = state.write().await;
                    if let Some(r) = st.records.get_mut(&id) {
                        r.status = JobStatus::Success;
                        r.result = Some(serde_json::to_value(res).unwrap_or(JsonValue::Null));
                        r.finished_at = Some(now_millis());
                        if let Some(p) = r.progress.as_mut() {
                            p.message = Some("completed".into());
                            p.percent = Some(100);
                        }
                    }
                    st.running.remove(&id);
                }
                Err(e) => {
                    let mut st = state.write().await;
                    if let Some(r) = st.records.get_mut(&id) {
                        r.status = JobStatus::Error;
                        r.error = Some(format!("{}", e));
                        r.finished_at = Some(now_millis());
                        if let Some(p) = r.progress.as_mut() {
                            p.message = Some("error".into());
                        }
                    }
                    st.running.remove(&id);
                }
            }
        });

        {
            let mut st = self.state.write().await;
            st.running.insert(id, RunningJob { handle });
        }

        Ok(id)
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}
