//! 屿 · 职业系统与编排。
//!
//! 职业 = 数据包（TOML），编排 = 信号 → 需求 → 评分 → 合成。
//! 代码只做引擎，不做 if-else 业务。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use yu_abi::{NeedVector, YuError};

/// 五轴权重。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Weights {
    pub attention: f32,
    pub rhythm: f32,
    pub info: f32,
    pub social: f32,
    pub growth: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            attention: 0.35,
            rhythm: 0.25,
            info: 0.20,
            social: 0.10,
            growth: 0.10,
        }
    }
}

/// 客观信号（全部本地可得）。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Signals {
    /// 0 = 空闲, 1 = 上课/考试, 2 = 深度专注
    pub phase: u8,
    pub fullscreen: bool,
    pub idle_secs: u32,
    pub battery_low: bool,
    pub todo_pressure: f32,
    pub focusing: bool,
    pub media_playing: bool,
    pub agent_active: bool,
    pub exam_in_days: Option<u32>,
}

impl Signals {
    /// 信号 → 五轴需求（0–3，带 0.25 滞回）。
    pub fn need(&self) -> NeedVector {
        let mut attention: f32 = if self.focusing { 2.0 } else { 0.5 };
        if self.fullscreen || self.phase >= 1 {
            attention = 3.0;
        }
        if self.idle_secs > 120 && attention > 0.0 {
            attention -= 1.0;
        }

        let rhythm = match self.phase {
            0 => 1.0,
            1 => 3.0,
            _ => 2.0,
        };

        let mut info = 1.0 + self.todo_pressure * 1.5;
        if self.media_playing {
            info += 0.5;
        }
        if self.battery_low {
            info = info.min(1.0);
        }

        let social = if self.fullscreen { 0.0 } else { 1.0 };

        let growth = if self.exam_in_days.map(|d| d <= 14).unwrap_or(false) {
            3.0
        } else if self.todo_pressure > 0.5 {
            2.0
        } else {
            1.0
        };

        NeedVector {
            attention: quantize(attention),
            rhythm: quantize(rhythm),
            info: quantize(info),
            social: quantize(social),
            growth: quantize(growth),
        }
    }
}

fn quantize(v: f32) -> f32 {
    let clamped = v.clamp(0.0, 3.0);
    (clamped * 4.0).round() / 4.0
}

/// 职业包：只允许操作编排词汇，不允许写逻辑。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub archetype: String,
    #[serde(default)]
    pub enabled_modules: Vec<String>,
    #[serde(default)]
    pub capsule: Capsule,
    #[serde(default)]
    pub policy: Policy,
    #[serde(default)]
    pub focus: Focus,
    #[serde(default)]
    pub process: ProcessTreat,
    #[serde(default)]
    pub obsidian: Obsidian,
    #[serde(default, rename = "ritual")]
    pub rituals: Vec<Ritual>,
    #[serde(default)]
    pub templates: Templates,
    #[serde(default)]
    pub need_weights: Weights,
}

fn default_version() -> String {
    "1.0.0".into()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capsule {
    #[serde(default)]
    pub left: Vec<String>,
    #[serde(default)]
    pub mid: Vec<String>,
    #[serde(default)]
    pub right: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    #[serde(default)]
    pub quiet_when_foreground: Vec<String>,
    #[serde(default)]
    pub quiet_hours: Vec<String>,
    #[serde(default = "default_budget")]
    pub interruption_budget: u32,
    #[serde(default = "default_degrade")]
    pub degrade_to: String,
}

fn default_budget() -> u32 {
    6
}
fn default_degrade() -> String {
    "badge".into()
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            quiet_when_foreground: vec![],
            quiet_hours: vec![],
            interruption_budget: default_budget(),
            degrade_to: default_degrade(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Focus {
    #[serde(default = "d25")]
    pub work: u32,
    #[serde(default = "d5")]
    pub rest: u32,
    #[serde(default = "d4")]
    pub long_break_every: u32,
}

fn d25() -> u32 {
    25
}
fn d5() -> u32 {
    5
}
fn d4() -> u32 {
    4
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            work: d25(),
            rest: d5(),
            long_break_every: d4(),
        }
    }
}

/// 进程待遇：借 TSGMemOptimizer 的 Profile 结构，语义改为「给谁让路」。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessTreat {
    #[serde(default)]
    pub boost: Vec<ProcRule>,
    #[serde(default)]
    pub protect: Vec<String>,
    #[serde(default)]
    pub eco: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcRule {
    pub name: String,
    #[serde(default)]
    pub delta: i8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Obsidian {
    #[serde(default)]
    pub daily_folder: String,
    #[serde(default)]
    pub task_tag: String,
    #[serde(default)]
    pub write_block: String,
    #[serde(default)]
    pub inbox: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ritual {
    pub id: String,
    #[serde(default)]
    pub match_time: String,
    #[serde(default)]
    pub match_app: Vec<String>,
    #[serde(default)]
    pub gap_min: u32,
    #[serde(default)]
    pub guard: RitualGuard,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RitualGuard {
    #[serde(default)]
    pub notices: String,
    #[serde(default)]
    pub motion: String,
    #[serde(default)]
    pub media_cover: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Templates {
    #[serde(default)]
    pub todo: String,
    #[serde(default)]
    pub review: String,
    #[serde(default)]
    pub items: Vec<String>,
}

impl Pack {
    pub fn load(path: &Path) -> Result<Self, YuError> {
        let raw = std::fs::read_to_string(path).map_err(|e| YuError::Io {
            what: path.display().to_string(),
            detail: e.to_string(),
        })?;
        let pack: Pack = toml::from_str(&raw).map_err(|e| YuError::Parse {
            what: path.display().to_string(),
            detail: e.to_string(),
        })?;
        pack.validate()?;
        Ok(pack)
    }

    pub fn validate(&self) -> Result<(), YuError> {
        if self.id.is_empty() {
            return Err(YuError::Parse {
                what: "pack.id".into(),
                detail: "id 不能为空".into(),
            });
        }
        if !self.archetype.is_empty() && !archetype_exists(&self.archetype) {
            return Err(YuError::Parse {
                what: "pack.archetype".into(),
                detail: format!("未知编排型：{}", self.archetype),
            });
        }
        Ok(())
    }

    pub fn fit(&self, need: NeedVector) -> f32 {
        let w = self.need_weights;
        // 需求越接近 3，且包的权重越高，得分越高；上限 1.0。
        let raw = (need.attention * w.attention
            + need.rhythm * w.rhythm
            + need.info * w.info
            + need.social * w.social
            + need.growth * w.growth)
            / 3.0;
        raw.clamp(0.0, 1.0)
    }
}

pub fn load_dir(dir: &Path) -> Result<Vec<Pack>, YuError> {
    let mut out = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| YuError::Io {
        what: dir.display().to_string(),
        detail: e.to_string(),
    })?;
    for entry in entries.flatten() {
        let path = entry.path().join("pack.toml");
        if path.exists() {
            out.push(Pack::load(&path)?);
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// 评分 + 切换判定（阈值 0.55、差值 0.12、最短驻留由调用方计时）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub current: String,
    pub best: String,
    pub best_score: f32,
    pub switch: bool,
    pub reason: String,
}

pub fn decide(current: &str, packs: &[Pack], need: NeedVector) -> Option<Decision> {
    let scored: Vec<(&Pack, f32)> = packs.iter().map(|p| (p, p.fit(need))).collect();
    let (best, best_score) = scored
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))?;
    let cur_score = scored
        .iter()
        .find(|(p, _)| p.id == current)
        .map(|(_, s)| *s)
        .unwrap_or(0.0);
    let switch = *best_score >= 0.55 && (best_score - cur_score) >= 0.12;
    Some(Decision {
        current: current.to_string(),
        best: best.id.clone(),
        best_score: *best_score,
        switch,
        reason: format!(
            "need(a{:.1} r{:.1} i{:.1} s{:.1} g{:.1})",
            need.attention, need.rhythm, need.info, need.social, need.growth
        ),
    })
}

// ───────────────────────── 64 编排型 ─────────────────────────

pub const AXIS_A: [&str; 4] = ["浮", "明", "沉", "渊"];
pub const AXIS_R: [&str; 4] = ["自由", "短章", "番茄", "课时"];
pub const AXIS_I: [&str; 4] = ["一瞥", "一卡", "数列", "全图"];

pub fn archetype_code(a: usize, r: usize, i: usize) -> String {
    format!("ARI-A{a}R{r}I{i}")
}

pub fn archetype_name(a: usize, r: usize, i: usize) -> String {
    format!("{}·{}·{}", AXIS_A[a], AXIS_R[r], AXIS_I[i])
}

pub fn archetype_exists(code: &str) -> bool {
    all_archetypes().iter().any(|x| x.code == code)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archetype {
    pub code: String,
    pub name: String,
    pub a: usize,
    pub r: usize,
    pub i: usize,
}

pub fn all_archetypes() -> Vec<Archetype> {
    let mut v = Vec::with_capacity(64);
    for a in 0..4 {
        for r in 0..4 {
            for i in 0..4 {
                v.push(Archetype {
                    code: archetype_code(a, r, i),
                    name: archetype_name(a, r, i),
                    a,
                    r,
                    i,
                });
            }
        }
    }
    v
}

/// 生成 64 个 TOML 到目录（可被职业包引用）。
pub fn generate_archetypes(dir: &Path) -> Result<usize, YuError> {
    std::fs::create_dir_all(dir).map_err(|e| YuError::Io {
        what: dir.display().to_string(),
        detail: e.to_string(),
    })?;
    let mut n = 0;
    for arch in all_archetypes() {
        let body = format!(
            "code = \"{}\"\nname = \"{}\"\nattention = {}\nrhythm = {}\ninfo = {}\n",
            arch.code, arch.name, arch.a, arch.r, arch.i
        );
        let file: PathBuf = dir.join(format!("{}.toml", arch.code.to_lowercase()));
        std::fs::write(&file, body).map_err(|e| YuError::Io {
            what: file.display().to_string(),
            detail: e.to_string(),
        })?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_four_names_are_unique() {
        let all = all_archetypes();
        assert_eq!(all.len(), 64);
        let mut names: Vec<_> = all.iter().map(|a| a.name.clone()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 64);
    }

    #[test]
    fn deep_focus_need_increases_attention() {
        let s = Signals {
            phase: 1,
            fullscreen: true,
            ..Default::default()
        };
        assert!(s.need().attention >= 3.0);
    }
}
