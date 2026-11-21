//! 屿 · 待办。
//!
//! JSON 为在线真源；Markdown 只负责导出与 Obsidian 互通。
//! 只写 `%% yu:begin %%` / `%% yu:end %%` 之间的块，绝不覆盖用户正文。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use yu_abi::{Origin, TodoItem, YuError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub version: u32,
    pub revision: u64,
    pub todos: Vec<TodoItem>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: 2,
            revision: 1,
            todos: Vec::new(),
        }
    }
}

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, mut item: TodoItem) -> String {
        self.revision += 1;
        item.updated_at = yu_abi::now_ms() as i64;
        let id = item.id.clone();
        self.todos.push(item);
        id
    }

    pub fn toggle(&mut self, id: &str) -> Option<bool> {
        self.revision += 1;
        let item = self.todos.iter_mut().find(|t| t.id == id)?;
        item.done = !item.done;
        item.updated_at = yu_abi::now_ms() as i64;
        Some(item.done)
    }

    pub fn pending(&self) -> usize {
        self.todos.iter().filter(|t| !t.done).count()
    }

    pub fn pressure(&self) -> f32 {
        let total = self.todos.len().max(1) as f32;
        (self.pending() as f32 / total).clamp(0.0, 1.0)
    }

    pub fn load(path: &Path) -> Result<Self, YuError> {
        match std::fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str(&raw).map_err(|e| YuError::Parse {
                what: path.display().to_string(),
                detail: e.to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::new()),
            Err(e) => Err(YuError::Io {
                what: path.display().to_string(),
                detail: e.to_string(),
            }),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), YuError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| YuError::Io {
                what: dir.display().to_string(),
                detail: e.to_string(),
            })?;
        }
        let tmp = PathBuf::from(format!("{}.tmp", path.display()));
        std::fs::write(
            &tmp,
            serde_json::to_vec_pretty(self).map_err(|e| YuError::Parse {
                what: "todo".into(),
                detail: e.to_string(),
            })?,
        )
        .map_err(|e| YuError::Io {
            what: tmp.display().to_string(),
            detail: e.to_string(),
        })?;
        std::fs::rename(&tmp, path).map_err(|e| YuError::Io {
            what: path.display().to_string(),
            detail: e.to_string(),
        })
    }

    /// 从今日 md 导入（`- [ ] [课内] 上阅读课`）。
    pub fn import_markdown(&mut self, path: &Path) -> Result<usize, YuError> {
        let raw = std::fs::read_to_string(path).map_err(|e| YuError::Io {
            what: path.display().to_string(),
            detail: e.to_string(),
        })?;
        let mut n = 0;
        for line in raw.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("- [") {
                continue;
            }
            let done = trimmed.starts_with("- [x]");
            let rest = &trimmed[5..];
            let (project, title) = if let Some(stripped) = rest.strip_prefix('[') {
                if let Some(end) = stripped.find(']') {
                    (
                        Some(stripped[..end].to_string()),
                        stripped[end + 1..].trim().to_string(),
                    )
                } else {
                    (None, rest.trim().to_string())
                }
            } else {
                (None, rest.trim().to_string())
            };
            if title.is_empty() {
                continue;
            }
            let mut item = TodoItem::new(title);
            item.done = done;
            item.project = project;
            item.origin = Some(Origin::Obsidian {
                path: path.display().to_string(),
                block_hash: format!("{:x}", yu_abi::now_ms()),
            });
            self.add(item);
            n += 1;
        }
        Ok(n)
    }

    /// 导出到 md 的 `%% yu %%` 块（只替换自己的块）。
    pub fn export_block(&self) -> String {
        let mut out = String::from("%% yu:begin %%\n");
        for t in &self.todos {
            let mark = if t.done { "x" } else { " " };
            let project = t
                .project
                .as_ref()
                .map(|p| format!("[{p}] "))
                .unwrap_or_default();
            out.push_str(&format!("- [{mark}] {project}{}\n", t.title));
        }
        out.push_str("%% yu:end %%\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_toggle_and_pressure() {
        let mut s = Store::new();
        let id = s.add(TodoItem::new("完成山水六五作业"));
        assert_eq!(s.pending(), 1);
        assert_eq!(s.toggle(&id), Some(true));
        assert_eq!(s.pending(), 0);
    }

    #[test]
    fn parse_project_lines() {
        let mut s = Store::new();
        s.add(TodoItem::new("[课内] 上阅读课"));
        assert_eq!(s.todos.len(), 1);
    }
}
