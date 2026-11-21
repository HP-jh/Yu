//! 屿 · 微内核。
//!
//! 只依赖 yu-abi；禁止依赖任何 yu-domain-*、tauri、平台库。
//! 提供：注册表、事件总线、分层配置、状态存储、打扰预算。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use yu_abi::{Capability, IslandState, ModuleId, Taxon, YuError};

pub mod registry {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ModuleDesc {
        pub id: ModuleId,
        pub taxon: Taxon,
        pub capabilities: Vec<Capability>,
    }

    /// 模块注册表：运行时事实源。
    #[derive(Default)]
    pub struct Registry {
        items: BTreeMap<String, ModuleDesc>,
    }

    impl Registry {
        pub fn register(&mut self, desc: ModuleDesc) -> Result<(), YuError> {
            let key = desc.id.0.clone();
            if self.items.contains_key(&key) {
                return Err(YuError::RegistryDuplicate { id: key });
            }
            self.items.insert(key, desc);
            Ok(())
        }

        pub fn get(&self, id: &str) -> Option<&ModuleDesc> {
            self.items.get(id)
        }

        pub fn len(&self) -> usize {
            self.items.len()
        }

        pub fn is_empty(&self) -> bool {
            self.items.is_empty()
        }

        /// 以分类路径分组的树（供 `yu-cli doctor` 与设置页自省）。
        pub fn taxon_tree(&self) -> BTreeMap<String, Vec<String>> {
            let mut tree: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for d in self.items.values() {
                let branch = format!("{}/{}", d.taxon.phylum, d.taxon.class);
                tree.entry(branch).or_default().push(d.id.0.clone());
            }
            tree
        }
    }
}

pub mod bus {
    use super::*;

    type Handler = Box<dyn Fn(&str) + Send + Sync>;

    /// 唯一跨域通道：所有跨域消息必须经过这里。
    #[derive(Default)]
    pub struct EventBus {
        topics: HashMap<String, Vec<Handler>>,
    }

    impl EventBus {
        pub fn subscribe(&mut self, topic: &str, f: impl Fn(&str) + Send + Sync + 'static) {
            self.topics
                .entry(topic.to_string())
                .or_default()
                .push(Box::new(f));
        }

        pub fn publish(&self, topic: &str, payload: &str) -> usize {
            let mut n = 0;
            if let Some(handlers) = self.topics.get(topic) {
                for h in handlers {
                    h(payload);
                    n += 1;
                }
            }
            n
        }
    }
}

pub mod config {
    use super::*;

    /// 分层配置：内置默认 → 职业包 → 用户覆写 → 运行时。
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct Layered {
        layers: Vec<(String, toml::Value)>,
    }

    impl Layered {
        pub fn push_layer(&mut self, name: impl Into<String>, value: toml::Value) {
            self.layers.push((name.into(), value));
        }

        pub fn load_file(&mut self, name: impl Into<String>, path: &Path) -> Result<(), YuError> {
            let raw = std::fs::read_to_string(path).map_err(|e| YuError::Io {
                what: path.display().to_string(),
                detail: e.to_string(),
            })?;
            let value: toml::Value = toml::from_str(&raw).map_err(|e| YuError::Parse {
                what: path.display().to_string(),
                detail: e.to_string(),
            })?;
            self.push_layer(name, value);
            Ok(())
        }

        /// 获取扁平键，返回 (值, 来源层)。
        pub fn get(&self, dotted: &str) -> Option<(&toml::Value, &str)> {
            let mut found = None;
            for (name, layer) in &self.layers {
                if let Some(v) = dig(layer, dotted) {
                    found = Some((v, name.as_str()));
                }
            }
            found
        }

        pub fn provenance(&self) -> Vec<(String, String)> {
            self.layers
                .iter()
                .map(|(n, v)| (n.clone(), v.to_string().chars().take(48).collect()))
                .collect()
        }
    }

    fn dig<'a>(v: &'a toml::Value, dotted: &str) -> Option<&'a toml::Value> {
        let mut cur = v;
        for part in dotted.split('.') {
            cur = cur.get(part)?;
        }
        Some(cur)
    }
}

pub mod state {
    use super::*;

    /// 版本化状态：内存快照 + JSON 持久化（redb 在后续版本替换）。
    pub struct StateStore {
        path: Option<std::path::PathBuf>,
        island: Mutex<IslandState>,
    }

    impl StateStore {
        pub fn new(path: Option<std::path::PathBuf>) -> Self {
            Self {
                path,
                island: Mutex::new(IslandState::default()),
            }
        }

        pub fn snapshot(&self) -> IslandState {
            self.island.lock().unwrap().clone()
        }

        pub fn update<R>(&self, f: impl FnOnce(&mut IslandState) -> R) -> R {
            let mut s = self.island.lock().unwrap();
            s.revision += 1;
            f(&mut s)
        }

        pub fn save(&self) -> Result<(), YuError> {
            let Some(path) = &self.path else {
                return Ok(());
            };
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| YuError::Io {
                    what: dir.display().to_string(),
                    detail: e.to_string(),
                })?;
            }
            let snap = self.snapshot();
            let tmp = path.with_extension("tmp");
            std::fs::write(
                &tmp,
                serde_json::to_vec_pretty(&snap).map_err(|e| YuError::Parse {
                    what: "state".into(),
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
    }
}

pub mod policy {
    use super::*;

    /// 打扰预算：每小时最多几张卡，以及降级链。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Budget {
        pub cards_per_hour: u32,
        pub used: u32,
        pub degrade: String,
    }

    impl Default for Budget {
        fn default() -> Self {
            Self {
                cards_per_hour: 6,
                used: 0,
                degrade: "badge".into(),
            }
        }
    }

    impl Budget {
        pub fn can_pop(&self) -> bool {
            self.used < self.cards_per_hour
        }

        pub fn spend(&mut self) -> bool {
            if self.can_pop() {
                self.used += 1;
                true
            } else {
                false
            }
        }

        pub fn reset_hourly(&mut self) {
            self.used = 0;
        }
    }
}

/// 内核句柄：域模块拿到的是这些服务，而不是全局单例。
#[derive(Clone)]
pub struct Kernel {
    pub registry: Arc<Mutex<registry::Registry>>,
    pub bus: Arc<Mutex<bus::EventBus>>,
    pub state: Arc<state::StateStore>,
}
