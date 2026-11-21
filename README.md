# 屿 · Yu

> 电脑 UI 的一部分：一个会变形的胶囊、一个极简工作台、一个电脑数据的汇总中心。
> 全新实现，**基于浮屿（Fuyu）的产品理念与使用体验重新设计**；不使用其源代码。

## ⚠️ 致歉与致谢

屿的形态、命名与部分交互思路，来自我们此前深度使用的 **浮屿（Fuyu）**。这个项目最初只是给浮屿换皮，后来推倒重写；过程中给浮屿作者带来过困扰（同名进程、配置目录混用、界面被替换），在此**诚挚致歉**。

- 屿与浮屿**没有代码包含关系**，也不受其 AGPL 许可约束（详见 [NOTICE.md](NOTICE.md)）；
- 如果你喜欢这套形态，请先去支持浮屿：<https://github.com/TixXin/Fuyu>；
- 若浮屿作者认为任何命名/界面元素不妥，提 issue，我们立刻改。

## 这是什么

- **6 种形态**：待机 / 媒体 / 通知 / 专注 / 告警 / 职业，形状随状态变；
- **4 类呈现面**：胶囊（306×42）、面板（560）、独立小卡（通知 356 / 待办 300）、中心（620）；
- **17 个动画**，全部挂 motion token，`prefers-reduced-motion` 全量降级；
- **职业系统**：6 个包（学生 / 开发者 / 教师 / 创作者 / 办公 / 自由职业），学生包含课表、上下课、作业拆解、背诵 SM-2、错题速记、考试倒计时；
- **64 编排型**：`浮·自由·一瞥` 这类中文命名，程序用 `ARI-A0R0I0` 代号；
- **模块可拆编译**：一域一 crate，`cargo build -p yu-domain-sensor` 单独出库；
- **Obsidian / 系统状态 / Agent 状态** 三条联动；Go sidecar 提供本地 API 与 MCP。

## 仓库结构

```
yu/
├─ crates/
│  ├─ yu-abi/               稳定 ABI：trait + DTO + 错误码
│  ├─ yu-kernel/            注册表 / 总线 / 配置 / 状态 / 预算
│  ├─ yu-domain-island/     6 形态状态机 + 几何
│  ├─ yu-domain-todo/       待办（JSON 真源 + md 导出）
│  ├─ yu-domain-sensor/     系统指标（Linux /proc）
│  ├─ yu-domain-occupation/ 职业包 + 5 轴需求 + 评分 + 64 编排型
│  └─ yu-cli/               yu doctor / pack lint / archetype gen
├─ apps/desktop/            Tauri 2 宿主（本机编译）
├─ ui/                      可直接用浏览器打开的岛
├─ packs/occupations/       6 个职业包（TOML）
├─ packs/archetypes/        7 个样例（完整 64 个由 CLI 生成）
├─ sidecars/collector/      Go：采集 + 本地 API + MCP
└─ docs/                    架构 / 职业系统 / 路线
```

## 快速开始

### 只看界面（不需要编译）

```bash
# 任意静态服务器
python3 -m http.server 8080 --directory ui
# 打开 http://localhost:8080 ，按 1–6 切形态，底部切职业
# 深链：#panel / #notice / #todo / #center 可直接看对应层
```

### 只跑内核与职业系统

```bash
cargo test --workspace
cargo run -p yu-cli -- doctor
cargo run -p yu-cli -- pack lint packs/occupations
cargo run -p yu-cli -- archetype gen packs/archetypes
cargo run -p yu-cli -- demo
```

### 编桌面版（Windows，本机）

```powershell
# 需要 Rust + MSVC + WebView2
.\scripts\build.ps1          # 限 4 个 P 核、BelowNormal、CARGO_BUILD_JOBS=4
# 产物：apps\desktop\src-tauri\target\release\yu.exe
```

> 桌面宿主只做三件事：开透明窗口、加载 `ui/`、把窗口显示出来。业务逻辑都在 crates 里。

## 已知限制（v0.1）

- 桌面宿主尚未接 IPC：形态切换先在页面底部手动演示；
- Windows 的 CPU/内存采集由宿主注入（sidecar 目前只实现 Linux）；
- 通知/待办尚未接真实数据源（`yu todo import <md>` 可先导入）；
- 完整 64 个编排型用 `yu archetype gen` 生成，仓库只存 7 个样例；
- 界面里的 CPU/内存是演示数据，接宿主后由传感器推送。

## 许可

MIT，见 [LICENSE](LICENSE)。与浮屿的 AGPL 无关；若你复用浮屿代码，请遵守其许可。
