# 本机编译（Windows）

## 0. 前置

- Rust stable + MSVC 工具链（`rustup default stable-x86_64-pc-windows-msvc`）
- WebView2 Runtime（Win11 自带；Win10 需装 Evergreen）
- Tauri CLI：`cargo install tauri-cli --version "^2" --locked`

## 1. 一条命令

```powershell
git clone https://github.com/HP-jh/Yu.git
cd Yu
.\scripts\build.ps1
```

产物：`apps\desktop\src-tauri\target\release\yu.exe`

## 2. 不想限核（最快）

```powershell
cd apps\desktop\src-tauri
cargo tauri build --no-bundle
```

`--no-bundle` 只出 exe，不打包安装器，省 1–2 分钟；前端 `ui/` 会被嵌进二进制。

## 3. 限 CPU 的三种做法

| 目标 | 做法 |
|---|---|
| 只用性能核（Intel 12 代+） | `cmd /c "start /affinity 0x3C3 /b /wait cargo build --release"`（0–1、6–9 号逻辑核；按自己 CPU 调整） |
| 只用能效核（把编译赶到 E 核，前台更顺） | 把 affinity 掩码改成 E 核位，例如 `0xFC`（2–7 号逻辑核） |
| 整体降优先级 | `Start-Process -FilePath cargo -ArgumentList "build --release" -Priority BelowNormal -Wait` |

进程优先级只影响调度权重，affinity 才决定落在哪类核；两者可叠加。

## 4. 让编译更快的开关

```powershell
$env:CARGO_BUILD_JOBS = "4"          # 并行度=核数，别超过物理 P 核
$env:RUSTFLAGS = "-C debuginfo=0"    # release 不需要调试信息
$env:CARGO_PROFILE_RELEASE_LTO = "thin"
$env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "16"
```

不要用 `lto = "fat"`：链接阶段单线程，反而更慢。

## 5. 「没有出现岛」排查顺序

1. **exe 起来了吗**：任务管理器看 `yu.exe` 是否在；没有就是启动即崩，用 `cargo run` 看 panic。
2. **窗口透明 + 无边框**：`tauri.conf.json` 里 `transparent: true`、`decorations: false`；缺 WebView2 会白屏/黑屏。
3. **先 show 再改几何**：`main.rs` 已改成 setup 里先 `show()`、`on_page_load` 再 `show()`，避免几何失败导致永不显示。
4. **`visible: false` 的坑**：如果你的机器上仍不出现，把 `visible` 改 `true` 再编一次，确认是显示时序还是渲染问题。
5. **页面空白但窗口在**：说明前端没嵌进去，用 `cargo tauri build --no-bundle` 重编，别用裸 `cargo build`。
6. **多屏 / DPI**：`width/height` 是逻辑像素，缩放 150% 时视觉会偏小，属正常。

## 6. 跑起来后

```powershell
# 命令行自检（不依赖 UI）
cargo run -p yu-cli -- doctor
cargo run -p yu-cli -- demo
```

首次运行会在 `%USERPROFILE%\.yu` 建目录；与浮屿的 `~/.fuyu` 完全隔离。
