use crate::process::KillReport;
use crate::profile::Profile;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PROTOCOL_VERSION: u16 = 2;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthContext {
    InteractiveUser,
    RunnerService,
    ScheduledTask,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub protocol_version: u16,
    pub client_id: String,
    pub request_id: String,
    pub timestamp_unix_ms: u64,
    pub auth_context: AuthContext,
    pub payload: T,
}

impl<T> Envelope<T> {
    pub fn new(client_id: impl Into<String>, auth_context: AuthContext, payload: T) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            client_id: client_id.into(),
            request_id: next_request_id("edge"),
            timestamp_unix_ms: now_unix_ms(),
            auth_context,
            payload,
        }
    }

    pub fn with_request_id(
        client_id: impl Into<String>,
        auth_context: AuthContext,
        request_id: impl Into<String>,
        payload: T,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            client_id: client_id.into(),
            request_id: request_id.into(),
            timestamp_unix_ms: now_unix_ms(),
            auth_context,
            payload,
        }
    }

    pub fn respond<U>(
        &self,
        client_id: impl Into<String>,
        auth_context: AuthContext,
        payload: U,
    ) -> Envelope<U> {
        Envelope {
            protocol_version: self.protocol_version,
            client_id: client_id.into(),
            request_id: self.request_id.clone(),
            timestamp_unix_ms: now_unix_ms(),
            auth_context,
            payload,
        }
    }
}

pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

pub fn next_request_id(prefix: &str) -> String {
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}-{}", prefix, now_unix_ms(), counter)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CleanupKind {
    RecycleBin,
    BrowserCache,
}

impl CleanupKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CleanupKind::RecycleBin => "recycle-bin",
            CleanupKind::BrowserCache => "browser-cache",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingsToRunnerCommand {
    ActivateProfile {
        profile: Profile,
        game_session_id: Option<String>,
    },
    OpenFlyout,
    OpenSettings,
    RequestOptimization {
        profile: Profile,
        game_session_id: Option<String>,
    },
    RequestCleanup {
        cleanup_kind: CleanupKind,
    },
    PreviewImpact {
        profile: Profile,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EngineState {
    Starting,
    Ready,
    Degraded,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationResult {
    pub request_id: String,
    pub success: bool,
    pub summary: String,
    pub killed: Vec<String>,
    pub failed: Vec<String>,
    pub not_found: Vec<String>,
    pub skipped: Vec<String>,
}

impl OperationResult {
    pub fn from_kill_report(request_id: String, report: KillReport) -> Self {
        let success = report.failed.is_empty();
        let summary = format!(
            "killed={} failed={} not_found={} skipped={}",
            report.killed.len(),
            report.failed.len(),
            report.not_found.len(),
            report.blocklist_skipped.len()
        );

        Self {
            request_id,
            success,
            summary,
            killed: report.killed,
            failed: report.failed,
            not_found: report.not_found,
            skipped: report.blocklist_skipped,
        }
    }

    pub fn error(request_id: String, summary: impl Into<String>) -> Self {
        Self {
            request_id,
            success: false,
            summary: summary.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunnerToSettingsEvent {
    EngineState(EngineState),
    OptimizationResult(OperationResult),
    CleanupResult(OperationResult),
    UserActionRequired { reason: String },
    Ack { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunnerToEngineCommand {
    ApplyProfile { profile: Profile },
    KillProcesses { processes: Vec<String> },
    RunCleanup { cleanup_kind: CleanupKind },
    GetCapabilities,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineToRunnerEvent {
    Ack {
        message: String,
    },
    Progress {
        message: String,
    },
    Result(OperationResult),
    Error {
        code: String,
        recoverable: bool,
        message: String,
    },
    Capabilities {
        cleanup_kinds: Vec<CleanupKind>,
        supports_process_kill: bool,
    },
    Pong,
}

#[derive(Debug, Default)]
pub struct IdempotencyCache {
    seen: HashSet<String>,
}

impl IdempotencyCache {
    pub fn check_and_insert(&mut self, request_id: &str) -> bool {
        self.seen.insert(request_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ids_are_unique_enough() {
        let a = next_request_id("test");
        let b = next_request_id("test");
        assert_ne!(a, b);
    }

    #[test]
    fn idempotency_cache_detects_duplicates() {
        let mut cache = IdempotencyCache::default();
        assert!(cache.check_and_insert("req-1"));
        assert!(!cache.check_and_insert("req-1"));
    }
}
