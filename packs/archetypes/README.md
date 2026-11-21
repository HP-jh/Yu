# 编排型（Archetype）

3 轴 × 4 档 = 64 个组合：

- A 注意深度：浮 / 明 / 沉 / 渊
- R 节奏：自由 / 短章 / 番茄 / 课时
- I 信息密度：一矇 / 一卡 / 数列 / 全图

命名：`明·课时·一矇`；代号：`ARI-A1R3I0`（程序用）。

完整 64 个由 CLI 生成，不提交生成物：

```bash
cargo run -p yu-cli -- archetype gen packs/archetypes
cargo run -p yu-cli -- archetype list
```

本目录只保留 7 个被职业包引用的样例。
