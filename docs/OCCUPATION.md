# 职业系统

职业不是标签，是**情境预设**；编排不是 if-else，是「信号 → 需求 → 评分 → 合成」。

## 9 个概念

| 概念 | 在本仓库的落点 |
|---|---|
| Signal 信号 | `Signals`（phase/fullscreen/idle/battery/todo/focus/media/agent/exam） |
| Need 需求向量 | `Signals::need()` → 5 轴 0–3，带 0.25 量化 |
| Trait 编排功能 | 16 个（感知 8 + 编排 8），用于「职业画像」解释 |
| Policy 策略字段 | `Pack::policy`（安静时段、预算、降级链） |
| Ritual 节奏 | `Pack::rituals`（上课/课间/午休/自习/夜间/假期） |
| Slot 槽位 | `Pack::capsule`（左/中/右） |
| Template 模板 | `Pack::templates.items`（如作业四步拆解） |
| Guard 红线 | `RitualGuard`（notices/motion/media_cover） |
| Archetype 编排型 | 64 个，`浮/明/沉/渊 × 自由/短章/番茄/课时 × 一瞥/一卡/数列/全图` |

## 评分

```
score(pack) = Σ w_axis * need_axis * pack_weight_axis / 3
switch = best >= 0.55 && (best - current) >= 0.12
```

最短驻留 120s、防抖 3s、手动锁定优先；`yu demo` 会打印一次判定与原因。

## 学生包（重点）

- 6 种节奏：上课 / 课间 / 午休 / 自习 / 夜间 / 周末假期；
- 8 项能力：课表驱动、上下课提示、分科作业模板、四步拆解、背诵 SM-2、阅读打卡、错题速记、考试倒计时 + 护眼；
- Guard：上课与考试「零弹卡、零流光、无封面」；夜间不弹学习提醒；
- 进程待遇：上课时给网课提权、非前台降 EcoQoS、保护输入法与屿自身；
- 家长可见：默认关闭，开启只共享完成率与专注时长（不含标题与内容）。

## 新增一个职业

1. 复制 `packs/occupations/student/`；
2. 改 `id / name / archetype / 槽位 / 策略 / 模板`；
3. `cargo run -p yu-cli -- pack lint packs/occupations`。

不需要改 Rust 代码。
