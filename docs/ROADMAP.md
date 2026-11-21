# 路线（v0.1 → v1.0）

## v0.1（本仓库）
内核骨架、6 形态 UI、6 职业包、64 编排型、CLI、Go 采集、文档与 CI。

## v0.2
- `yu-platform`：Windows 前台进程 / 输入空闲 / SMTC / 壁纸主色；
- 桌面宿主接通 IPC：Rust 推 `IslandState`，页面不再手动切形态；
- Obsidian 文件桥：只写 `%% yu %%` 块 + 三方合并 + 冲突卡；
- 托盘、全局热键、单实例、开机自启（注册表值 `Yu`）。

## v0.3
- WASM 插件（WIT：ui.slot / data.source / action.exec）+ 2 个示例；
- Gatekeeper 两阶段确认；
- `yu-updater`：SHA256SUMS、原子替换、回滚。

## v1.0
- 性能达标：待机 CPU ≤1.5%、内存 ≤180MB、冷启 ≤1.2s、动画 ≥55fps；
- 安装包（NSIS）、签名、自动更新；
- 学生包课表可导入 ICS，家长可见开关上线。
