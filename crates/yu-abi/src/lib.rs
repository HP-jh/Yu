//! 屿 · 稳定 ABI。
//!
//! 只允许依赖 serde / serde_json；不含 IO、不含 unsafe。
//! 所有域模块只通过这里的 trait 与 DTO 与内核对话。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

pub const ABI_VERSION: &str = "0.1.0";

/// 模块标识，全局唯一。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub String);

impl ModuleId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 门纲目科属种。运行时事实源，路径即身份。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Taxon {
    pub phylum: String,
    pub class: String,
    pub order: String,
    pub family: String,
    pub genus: String,
    pub species: String,
}

impl Taxon {
    pub fn path(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}/{}",
            self.phylum, self.class, self.order, self.family, self.genus, self.species
        )
    }
}

/// 能力声明：内核在执行前校验。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    FsRead(String),
    FsWrite(String),
    NetHost(String),
    ProcExec,
    ProcSignal,
    Notify,
    Ui,
}

/// 形态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IslandMode {
    Idle,
    Media,
    Notice,
    Focus,
    Alert,
    Occupation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Calm,
    Attention,
    Urgent,
}

/// 胶囊槽位（有序）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleSlots {
    pub left: Vec<WidgetRef>,
    pub mid: Vec<WidgetRef>,
    pub right: Vec<WidgetRef>,
}

impl Default for CapsuleSlots {
    fn default() -> Self {
        Self {
            left: vec![WidgetRef::new("clock")],
            mid: vec![],
            right: vec![WidgetRef::new("battery")],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetRef {
    pub id: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl WidgetRef {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            args: vec![],
        }
    }
}

/// 前端只读的形态真源。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IslandState {
    pub mode: IslandMode,
    pub occupation: String,
    pub capsule: CapsuleSlots,
    pub severity: Severity,
    pub expanded: bool,
    pub revision: u64,
}

impl Default for IslandState {
    fn default() -> Self {
        Self {
            mode: IslandMode::Idle,
            occupation: "student".into(),
            capsule: CapsuleSlots::default(),
            severity: Severity::Calm,
            expanded: false,
            revision: 1,
        }
    }
}

/// 五轴需求，0–3。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct NeedVector {
    pub attention: f32,
    pub rhythm: f32,
    pub info: f32,
    pub social: f32,
    pub growth: f32,
}

/// 待办来源。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Origin {
    Local,
    Obsidian { path: String, block_hash: String },
    Plugin { id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub done: bool,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub origin: Option<Origin>,
    #[serde(default)]
    pub updated_at: i64,
}

impl TodoItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: format!("yu-{:x}", now_ms()),
            title: title.into(),
            done: false,
            project: None,
            due: None,
            origin: Some(Origin::Local),
            updated_at: now_ms() as i64,
        }
    }
}

/// 系统指标。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    CpuPercent,
    MemPercent,
    MemUsedMb,
    DiskPercent,
    NetRxKbps,
    NetTxKbps,
    BatteryPercent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub kind: MetricKind,
    pub value: f32,
    pub at_ms: u64,
}

/// 通知等级与打扰预算联动。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeLevel {
    Badge,
    Card,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeAction {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notice {
    pub id: String,
    pub level: NoticeLevel,
    pub source: String,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub actions: Vec<NoticeAction>,
    #[serde(default)]
    pub ttl_ms: u32,
}

/// 动作请求/结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionArgs {
    pub name: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutcome {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub module: ModuleId,
    pub capability: Capability,
}

/// 错误码：可诊断、可本地化。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum YuError {
    AbiVersionMismatch { expected: String, actual: String },
    RegistryDuplicate { id: String },
    PolicyDenied { module: String, capability: String },
    PlatformUnsupported { what: String },
    Parse { what: String, detail: String },
    Conflict { what: String, detail: String },
    Io { what: String, detail: String },
}

impl YuError {
    pub fn code(&self) -> &'static str {
        match self {
            YuError::AbiVersionMismatch { .. } => "yu.err.abi.version_mismatch",
            YuError::RegistryDuplicate { .. } => "yu.err.kernel.registry_dup",
            YuError::PolicyDenied { .. } => "yu.err.policy.denied",
            YuError::PlatformUnsupported { .. } => "yu.err.platform.unsupported",
            YuError::Parse { .. } => "yu.err.parse",
            YuError::Conflict { .. } => "yu.err.conflict",
            YuError::Io { .. } => "yu.err.io",
        }
    }
}

impl fmt::Display for YuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.code(), self)
    }
}

impl std::error::Error for YuError {}

/// 模块上下文：只给内核提供的服务句柄，不给自由。
#[derive(Clone, Copy, Debug)]
pub struct ModuleCtx {
    pub abi_version: &'static str,
}

pub trait Module: Send + Sync + 'static {
    fn id(&self) -> ModuleId;
    fn taxon(&self) -> Taxon;
    fn capabilities(&self) -> &'static [Capability];
    fn boot(&self, ctx: &ModuleCtx) -> Result<(), YuError>;
    fn shutdown(&self) {}
}

pub trait Source<T>: Module {
    fn poll(&self) -> Result<Vec<T>, YuError>;
}

pub trait Sink<T>: Module {
    fn apply(&self, items: &[T]) -> Result<usize, YuError>;
}

pub trait Action: Module {
    fn invoke(&self, args: &ActionArgs) -> Result<ActionOutcome, YuError>;
}

pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
