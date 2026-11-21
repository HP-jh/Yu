//! 屿 · 命令行：诊断、职业包校验、编排型生成、模块档位解析、待办迁移。
//!
//! 所有命令都不依赖 UI，可在纯终端跑。

use std::path::{Path, PathBuf};

use yu_domain_occupation as occ;
use yu_domain_todo as todo;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(|s| s.as_str()) {
        Some("doctor") => doctor(),
        Some("pack") => pack(&args[1..]),
        Some("archetype") => archetype(&args[1..]),
        Some("resolve") => resolve(&args[1..]),
        Some("todo") => todo_cmd(&args[1..]),
        Some("demo") => demo(),
        _ => {
            help();
            0
        }
    };
    std::process::exit(code);
}

fn help() {
    println!("屿 v0.1 — 内核与职业系统自检");
    println!("  yu doctor                 环境与数据自检");
    println!("  yu pack lint <dir>        校验职业包");
    println!("  yu archetype gen <dir>    生成 64 个编排型");
    println!("  yu resolve --tier standard 解析模块档位");
    println!("  yu todo import <md>        从 Markdown 导入待办");
    println!("  yu demo                    打印一组需求向量与切换建议");
}

fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("YU_HOME") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".yu")
}

fn doctor() -> i32 {
    let dir = data_dir();
    println!("yu.abi      {}", yu_abi::ABI_VERSION);
    println!("data dir    {}", dir.display());
    let shape = yu_domain_island::shape_for(yu_abi::IslandMode::Idle);
    println!(
        "island      idle {}x{} r{}",
        shape.collapsed.w, shape.collapsed.h, shape.collapsed.radius
    );
    let s = yu_domain_sensor::Sensor::default();
    match s.sample() {
        ms if ms.is_empty() => println!("sensor      unsupported（Windows 由宿主注入）"),
        ms => {
            for m in ms {
                println!("sensor      {:?}={:.1}", m.kind, m.value);
            }
        }
    }
    let store = todo::Store::load(&dir.join("todos.json")).unwrap_or_default();
    println!(
        "todos       {} 条，未完成 {}",
        store.todos.len(),
        store.pending()
    );
    let packs_dir = Path::new("packs/occupations");
    match occ::load_dir(packs_dir) {
        Ok(p) => println!(
            "packs       {} 个（{}）",
            p.len(),
            if p.is_empty() { "空" } else { "ok" }
        ),
        Err(e) => println!("packs       {e}"),
    }
    println!("status      ok");
    0
}

fn pack(args: &[String]) -> i32 {
    let dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("packs/occupations"));
    match occ::load_dir(&dir) {
        Ok(packs) => {
            for p in &packs {
                println!("{:<12} {:<10} archetype={}", p.id, p.name, p.archetype);
            }
            println!("lint ok: {} packs", packs.len());
            0
        }
        Err(e) => {
            eprintln!("lint failed: {e}");
            1
        }
    }
}

fn archetype(args: &[String]) -> i32 {
    match args.first().map(|s| s.as_str()) {
        Some("gen") => {
            let dir = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("packs/archetypes"));
            match occ::generate_archetypes(&dir) {
                Ok(n) => {
                    println!("generated {n} archetypes -> {}", dir.display());
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            }
        }
        Some("list") => {
            for a in occ::all_archetypes() {
                println!("{}  {}", a.code, a.name);
            }
            0
        }
        _ => {
            println!("yu archetype gen <dir> | list");
            0
        }
    }
}

fn resolve(args: &[String]) -> i32 {
    let mut tier = "standard".to_string();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--tier" {
            if let Some(v) = args.get(i + 1) {
                tier = v.clone();
            }
        }
        i += 1;
    }
    let catalog = yu_yaml();
    let enabled = match tier.as_str() {
        "core" => vec!["island", "sensor", "todo"],
        "standard" => vec!["island", "sensor", "todo", "media", "obsidian"],
        "full" => vec![
            "island", "sensor", "todo", "media", "obsidian", "agents", "plugins",
        ],
        other => {
            eprintln!("unknown tier: {other}（core|standard|full）");
            return 1;
        }
    };
    println!("tier = {tier}");
    for m in enabled {
        let deps = catalog
            .iter()
            .find(|(id, _, _)| *id == m)
            .map(|(_, _, d)| *d)
            .unwrap_or(&[]);
        println!("  {m:<10} crate={m}  deps={deps:?}");
    }
    0
}

type Catalog = Vec<(&'static str, &'static str, &'static [&'static str])>;

fn yu_yaml() -> Catalog {
    vec![
        ("island", "yu-domain-island", &[]),
        ("sensor", "yu-domain-sensor", &[]),
        ("todo", "yu-domain-todo", &[]),
        ("media", "yu-domain-media", &["sensor"]),
        ("obsidian", "yu-domain-obsidian", &["todo"]),
        ("agents", "yu-domain-agents", &[]),
        ("plugins", "yu-plugins", &["island"]),
    ]
}

fn todo_cmd(args: &[String]) -> i32 {
    match args.first().map(|s| s.as_str()) {
        Some("import") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: yu todo import <file.md>");
                return 1;
            };
            let mut store = todo::Store::load(&data_dir().join("todos.json")).unwrap_or_default();
            match store.import_markdown(Path::new(path)) {
                Ok(n) => {
                    if let Err(e) = store.save(&data_dir().join("todos.json")) {
                        eprintln!("save failed: {e}");
                        return 1;
                    }
                    println!("imported {n} todos");
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            }
        }
        Some("export") => {
            let store = todo::Store::load(&data_dir().join("todos.json")).unwrap_or_default();
            println!("{}", store.export_block());
            0
        }
        _ => {
            println!("yu todo import <md> | export");
            0
        }
    }
}

fn demo() -> i32 {
    let packs = occ::load_dir(Path::new("packs/occupations")).unwrap_or_default();
    let signals = occ::Signals {
        phase: 1,
        fullscreen: false,
        idle_secs: 10,
        battery_low: false,
        todo_pressure: 0.6,
        focusing: false,
        media_playing: false,
        agent_active: false,
        exam_in_days: Some(9),
    };
    let need = signals.need();
    println!(
        "need a{:.2} r{:.2} i{:.2} s{:.2} g{:.2}",
        need.attention, need.rhythm, need.info, need.social, need.growth
    );
    if packs.is_empty() {
        println!("（先运行 yu pack lint packs/occupations）");
        return 0;
    }
    if let Some(d) = occ::decide("student", &packs, need) {
        println!(
            "best={} score={:.2} switch={} reason={}",
            d.best, d.best_score, d.switch, d.reason
        );
    }
    0
}
