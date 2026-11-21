# 架构（v0.1）

## 四层（对齐 TarsGatewayNT 的 abi → kernel → kingdom → cmd）

```
cmd       apps/desktop（Tauri 2）、yu-cli
kingdom   yu-domain-island / todo / sensor / occupation（后续 media/obsidian/agents/plugins）
kernel    Registry / EventBus / Config / State / Policy
abi       trait + DTO + 错误码（无 IO、无 unsafe）
```

依赖铁律（后续用 `xtask deps` + CI 强制）：

1. `yu-abi` 只依赖 serde；
2. `yu-kernel` 只依赖 `yu-abi`，禁止依赖任何域与 tauri；
3. 域 crate 只依赖 abi + kernel，域之间不互相 import（跨域走 EventBus）；
4. `unsafe` 只允许出现在平台层（当前为 0 处）。

## 为什么一域一 crate

- `cargo build -p yu-domain-sensor` 可以只出这一块，测试也只跑这一块；
- 错误耦合会立刻暴露：域 A 想用域 B 的类型就必须经过 `yu-abi` 或总线；
- 发布时可以按 `core / standard / full` 组合裁剪（见 `yu resolve --tier`）。

## 状态与数据

- `IslandState` 是唯一形态真源，前端只读快照 + revision；
- 待办 `todos.json` 为在线真源，Markdown 只做导出与 Obsidian 互通；
- 写文件一律 tmp → rename，避免半截文件；
- 迁移浮屿：只读 `~/.fuyu`，写入 `~/.yu`。

## 形态引擎

6 形态（idle / media / notice / focus / alert / occupation），每个形态有独立的尺寸、圆角、强调色与允许的槽位；过渡用 `cubic-bezier(0.22,1,0.36,1)` 420ms，减少动画时 160ms、无回弹。

窗口策略：展开先撑窗（+48px 余量），收起等动画结束再缩窗。

## 后续（v0.2+）

- `yu-platform`：Win32 前台进程、输入空闲、SMTC、壁纸采样；
- `yu-plugins`：WASM（wasmtime + WIT），能力校验 + 签名；
- `yu-updater`：SHA256SUMS + 原子替换 + 回滚；
- Gatekeeper：高危动作两阶段确认（token 绑定参数哈希）。
