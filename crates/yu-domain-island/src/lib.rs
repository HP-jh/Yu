//! 屿 · 形态引擎。
//!
//! 六种形态 + 几何 + 过渡参数。纯计算，不碰窗口、不碰 DOM。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use yu_abi::{IslandMode, Severity, YuError};

/// 逻辑尺寸（px）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub w: f32,
    pub h: f32,
    pub radius: f32,
}

impl Size {
    pub const fn new(w: f32, h: f32, radius: f32) -> Self {
        Self { w, h, radius }
    }
}

/// 每个形态的身份：尺寸 + 视觉标记。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub mode: IslandMode,
    pub collapsed: Size,
    pub expanded: Size,
    pub severity: Severity,
}

pub fn shape_for(mode: IslandMode) -> Shape {
    let (w, h, r) = match mode {
        IslandMode::Idle => (306.0, 42.0, 21.0),
        IslandMode::Media => (360.0, 42.0, 21.0),
        IslandMode::Notice => (340.0, 46.0, 23.0),
        IslandMode::Focus => (300.0, 42.0, 21.0),
        IslandMode::Alert => (320.0, 44.0, 22.0),
        IslandMode::Occupation => (340.0, 46.0, 23.0),
    };
    Shape {
        mode,
        collapsed: Size::new(w, h, r),
        expanded: Size::new(560.0, 306.0, 32.0),
        severity: if matches!(mode, IslandMode::Alert) {
            Severity::Urgent
        } else if matches!(mode, IslandMode::Notice) {
            Severity::Attention
        } else {
            Severity::Calm
        },
    }
}

/// 过渡：开关对称、不用 linear、装饰不无限循环。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub from: IslandMode,
    pub to: IslandMode,
    pub duration_ms: u32,
    pub curve: &'static str,
    pub reduce_motion_ms: u32,
}

pub const SPRING: &str = "cubic-bezier(0.22,1,0.36,1)";
pub const EASE_OUT: &str = "cubic-bezier(0.16,1,0.3,1)";

pub fn transition(from: IslandMode, to: IslandMode, reduce_motion: bool) -> Transition {
    let base = match (from, to) {
        (IslandMode::Idle, IslandMode::Notice) => 240,
        (IslandMode::Idle, IslandMode::Media) => 260,
        (IslandMode::Idle, IslandMode::Focus) => 320,
        (IslandMode::Idle, IslandMode::Alert) => 200,
        (IslandMode::Idle, IslandMode::Occupation) => 520,
        _ => 420,
    };
    Transition {
        from,
        to,
        duration_ms: if reduce_motion { 160 } else { base },
        curve: if reduce_motion { EASE_OUT } else { SPRING },
        reduce_motion_ms: 160,
    }
}

/// 舞台窗口尺寸：展开时留 24px 余量给回弹，收起等动画结束再缩。
pub fn stage_size(shape: Shape, expanded: bool) -> Size {
    if expanded {
        Size::new(
            shape.expanded.w + 48.0,
            shape.expanded.h + 48.0,
            shape.expanded.radius,
        )
    } else {
        Size::new(
            shape.collapsed.w + 64.0,
            shape.collapsed.h + 48.0,
            shape.collapsed.radius,
        )
    }
}

/// 从状态推导形态：优先级 Alert > Focus > Notice > Media > Occupation > Idle。
pub fn resolve_mode(
    cpu_high: bool,
    focusing: bool,
    has_notice: bool,
    media_playing: bool,
    switching_occupation: bool,
) -> IslandMode {
    if cpu_high {
        IslandMode::Alert
    } else if focusing {
        IslandMode::Focus
    } else if has_notice {
        IslandMode::Notice
    } else if media_playing {
        IslandMode::Media
    } else if switching_occupation {
        IslandMode::Occupation
    } else {
        IslandMode::Idle
    }
}

pub struct IslandService;

impl IslandService {
    pub fn validate(shape: Shape) -> Result<Shape, YuError> {
        if shape.collapsed.w < 240.0 || shape.collapsed.h < 36.0 {
            return Err(YuError::Parse {
                what: "island.shape".into(),
                detail: "尺寸小于最小可用值".into(),
            });
        }
        Ok(shape)
    }
}
