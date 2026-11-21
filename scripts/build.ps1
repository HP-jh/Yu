# 屿 · 本机编译（Windows）
# 限 CPU：4 个 P 核 + BelowNormal + CARGO_BUILD_JOBS=4
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

$env:CARGO_BUILD_JOBS = '4'
$tauri = Join-Path $root 'apps\desktop\src-tauri'

Write-Host "== 1/3 校验内核 ==" -ForegroundColor Cyan
cargo test --workspace
cargo run -q -p yu-cli -- doctor

Write-Host "== 2/3 编译桌面宿主（限核）==" -ForegroundColor Cyan
Push-Location $tauri
# start /affinity 0x3C3 = 逻辑核 0,1,6,7,8,9（P 核子集），子进程继承
cmd /c "start /affinity 3C3 /b /wait cargo build --release --features tauri/custom-protocol"
Pop-Location
# 若页面空白（前端未嵌入）：cargo install tauri-cli --version ^2 ; cargo tauri build --no-bundle

Write-Host "== 3/3 完成 ==" -ForegroundColor Green
$exe = Join-Path $tauri 'target\release\yu.exe'
if (Test-Path $exe) { Write-Host "产物：$exe" } else { Write-Host "未找到 yu.exe，请检查 cargo 输出" -ForegroundColor Yellow }
