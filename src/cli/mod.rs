pub mod commands;
pub mod display;

use std::collections::{HashMap, HashSet};

use comfy_table::{ContentArrangement, Table};
use rustyline::DefaultEditor;

use crate::data::GameData;
use crate::engine::balance::compute_island_balance;
use crate::engine::solver::{Engine, SolverSettings};
use crate::model::island::PopulationSettings;
use crate::model::*;

use commands::{parse_command, Command, DifficultyCommand};

/// CLI 互動 Session 狀態
struct Session {
    engine: Engine,
    game_data: std::sync::Arc<GameData>,
    difficulty: DifficultySettings,
    population: PopulationSettings,
    unlocked_research: HashSet<ResearchId>,
    recipe_preferences: HashMap<ResourceId, RecipeId>,
    last_chain: Option<results::ProductionChain>,
}

/// 啟動 CLI REPL
pub fn run(game_data: GameData) {
    println!("《工業隊長》生產計算機");
    println!(
        "已載入 {} 資源 / {} 建築 / {} 配方 / {} 研究",
        game_data.resources.len(),
        game_data.buildings.len(),
        game_data.recipes.len(),
        game_data.research.len(),
    );
    println!("輸入「幫助」查看指令列表");
    println!();

    let game_data_arc = std::sync::Arc::new(game_data);
    let engine = Engine::from_arc(game_data_arc.clone());

    let mut session = Session {
        engine,
        game_data: game_data_arc,
        difficulty: DifficultySettings::default(),
        population: PopulationSettings::default(),
        unlocked_research: HashSet::new(),
        recipe_preferences: HashMap::new(),
        last_chain: None,
    };

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("無法初始化終端: {e}");
            return;
        }
    };

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(&line);
                let cmd = parse_command(&line);
                if matches!(cmd, Command::Quit) {
                    println!("再見！");
                    break;
                }
                handle_command(&mut session, cmd);
            }
            Err(rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof) => {
                println!("再見！");
                break;
            }
            Err(e) => {
                eprintln!("讀取錯誤: {e}");
                break;
            }
        }
    }
}

fn handle_command(session: &mut Session, cmd: Command) {
    match cmd {
        Command::Calculate { resource_id, rate } => cmd_calculate(session, &resource_id, rate),
        Command::Balance => cmd_balance(session),
        Command::Gaps => cmd_gaps(session),
        Command::Difficulty(sub) => cmd_difficulty(session, sub),
        Command::Unlock { research_id } => cmd_unlock(session, &research_id),
        Command::Lock { research_id } => cmd_lock(session, &research_id),
        Command::UnlockedList => cmd_unlocked_list(session),
        Command::Prefer { resource_id, recipe_id } => cmd_prefer(session, &resource_id, &recipe_id),
        Command::ListResources => cmd_list_resources(session),
        Command::ListBuildings => cmd_list_buildings(session),
        Command::ListRecipes { building_id } => cmd_list_recipes(session, building_id.as_deref()),
        Command::ListResearch => cmd_list_research(session),
        Command::Population { count, housing_tier } => cmd_population(session, count, housing_tier),
        Command::Help => display::print_help(),
        Command::Quit => unreachable!(),
        Command::Unknown(msg) => {
            if !msg.is_empty() {
                println!("{msg}");
            }
        }
    }
}

fn cmd_calculate(session: &mut Session, resource_id: &str, rate: f64) {
    let settings = SolverSettings {
        difficulty: session.difficulty.clone(),
        unlocked_research: session.unlocked_research.clone(),
        recipe_preferences: session.recipe_preferences.clone(),
    };

    match session
        .engine
        .solve_chain(&ResourceId::new(resource_id), rate, &settings)
    {
        Ok(chain) => {
            display::print_chain(&chain);
            session.last_chain = Some(chain);
        }
        Err(e) => println!("求解失敗: {e}"),
    }
}

fn cmd_balance(session: &Session) {
    match &session.last_chain {
        Some(chain) => display::print_balance_sheet(&chain.balance_sheet),
        None => println!("尚無計算結果，請先使用「計算」指令"),
    }
}

fn cmd_gaps(session: &Session) {
    let Some(chain) = &session.last_chain else {
        println!("尚無計算結果，請先使用「計算」指令");
        return;
    };

    let deficits = chain.balance_sheet.deficits();
    if deficits.is_empty() {
        println!("無赤字，所有資源平衡或盈餘！");
        return;
    }

    println!("── 資源赤字 ──");
    for (id, entry) in &deficits {
        let net = entry.net_per_min();
        if entry.is_raw_input {
            println!("  {} ({}): 需供給 {:.2}/分", entry.resource_name, id, net.abs());
        } else {
            println!("  {} ({}): 赤字 {:.2}/分", entry.resource_name, id, net.abs());
        }
    }
    println!();
    println!("（缺口建議功能將在後續版本實作）");
}

fn cmd_difficulty(session: &mut Session, sub: DifficultyCommand) {
    match sub {
        DifficultyCommand::Show => {
            println!("當前難度設定:");
            println!("  維護倍率 (maintenance): {:.2}", session.difficulty.maintenance_multiplier);
            println!("  燃油倍率 (fuel):        {:.2}", session.difficulty.fuel_multiplier);
            println!("  食物倍率 (food):        {:.2}", session.difficulty.food_consumption_multiplier);
            println!("  商品服務 (goods):       {:.2}", session.difficulty.goods_services_multiplier);
            println!(
                "  傳送帶耗電 (conveyor):  {}",
                if session.difficulty.conveyor_power_enabled { "開" } else { "關" }
            );
            println!(
                "  倉儲耗電 (storage):     {}",
                if session.difficulty.storage_power_enabled { "開" } else { "關" }
            );
        }
        DifficultyCommand::Set { key, value } => {
            match key.as_str() {
                "maintenance" | "維護" => {
                    if let Ok(v) = value.parse::<f64>() {
                        session.difficulty.maintenance_multiplier = v;
                        println!("維護倍率已設為 {v:.2}");
                    } else {
                        println!("無效數值: {value}");
                    }
                }
                "fuel" | "燃油" => {
                    if let Ok(v) = value.parse::<f64>() {
                        session.difficulty.fuel_multiplier = v;
                        println!("燃油倍率已設為 {v:.2}");
                    } else {
                        println!("無效數值: {value}");
                    }
                }
                "food" | "食物" => {
                    if let Ok(v) = value.parse::<f64>() {
                        session.difficulty.food_consumption_multiplier = v;
                        println!("食物倍率已設為 {v:.2}");
                    } else {
                        println!("無效數值: {value}");
                    }
                }
                "goods" | "商品" => {
                    if let Ok(v) = value.parse::<f64>() {
                        session.difficulty.goods_services_multiplier = v;
                        println!("商品服務倍率已設為 {v:.2}");
                    } else {
                        println!("無效數值: {value}");
                    }
                }
                "conveyor" | "傳送帶" => {
                    match value.as_str() {
                        "on" | "開" => { session.difficulty.conveyor_power_enabled = true; println!("傳送帶耗電: 開"); }
                        "off" | "關" => { session.difficulty.conveyor_power_enabled = false; println!("傳送帶耗電: 關"); }
                        _ => println!("用法: 難度 conveyor on|off"),
                    }
                }
                "storage" | "倉儲" => {
                    match value.as_str() {
                        "on" | "開" => { session.difficulty.storage_power_enabled = true; println!("倉儲耗電: 開"); }
                        "off" | "關" => { session.difficulty.storage_power_enabled = false; println!("倉儲耗電: 關"); }
                        _ => println!("用法: 難度 storage on|off"),
                    }
                }
                _ => println!("未知設定: {key}（可用: maintenance, fuel, food, goods, conveyor, storage）"),
            }
        }
    }
}

fn cmd_unlock(session: &mut Session, research_id: &str) {
    let id = ResearchId::new(research_id);
    if session.game_data.research.contains_key(&id) {
        session.unlocked_research.insert(id);
        println!("已解鎖: {research_id}");
    } else {
        println!("找不到研究: {research_id}");
        println!("使用「研究列表」查看所有可用研究");
    }
}

fn cmd_lock(session: &mut Session, research_id: &str) {
    let id = ResearchId::new(research_id);
    if session.unlocked_research.remove(&id) {
        println!("已鎖定: {research_id}");
    } else {
        println!("{research_id} 未在已解鎖列表中");
    }
}

fn cmd_unlocked_list(session: &Session) {
    if session.unlocked_research.is_empty() {
        println!("尚未解鎖任何研究（所有配方皆可用）");
        return;
    }

    println!("已解鎖研究:");
    let mut list: Vec<_> = session.unlocked_research.iter().collect();
    list.sort_by(|a, b| a.0.cmp(&b.0));
    for id in list {
        let name = session
            .game_data
            .research
            .get(id)
            .map(|r| r.name.as_str())
            .unwrap_or("(未知)");
        println!("  ✓ {} — {}", id, name);
    }
}

fn cmd_prefer(session: &mut Session, resource_id: &str, recipe_id: &str) {
    session.recipe_preferences.insert(
        ResourceId::new(resource_id),
        RecipeId::new(recipe_id),
    );
    println!("已設定偏好: {resource_id} → {recipe_id}");
}

fn cmd_list_resources(session: &Session) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["ID", "名稱", "English", "類別", "屬性"]);

    let mut resources: Vec<_> = session.game_data.resources.values().collect();
    resources.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    for r in resources {
        let mut flags = Vec::new();
        if r.is_primary { flags.push("原料"); }
        if r.is_waste { flags.push("廢棄物"); }
        if r.is_pollution { flags.push("污染"); }
        if r.is_virtual { flags.push("虛擬"); }

        table.add_row(vec![
            &r.id.0,
            &r.name,
            &r.name_en,
            &format!("{:?}", r.category),
            &flags.join(", "),
        ]);
    }
    println!("{table}");
}

fn cmd_list_buildings(session: &Session) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["ID", "名稱", "類別", "工人", "耗電KW", "研究需求"]);

    let mut buildings: Vec<_> = session.game_data.buildings.values().collect();
    buildings.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    for b in buildings {
        let research = b
            .research_required
            .as_ref()
            .map(|r| r.0.as_str())
            .unwrap_or("—");

        table.add_row(vec![
            &b.id.0,
            &b.name,
            &format!("{:?}", b.category),
            &b.workers.to_string(),
            &format!("{:.0}", b.base_electricity_kw),
            research,
        ]);
    }
    println!("{table}");
}

fn cmd_list_recipes(session: &Session, building_filter: Option<&str>) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["ID", "名稱", "建築", "週期(秒)", "輸入", "輸出", "研究需求"]);

    let mut recipes: Vec<_> = session.game_data.recipes.values().collect();
    recipes.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    for r in recipes {
        if let Some(filter) = building_filter {
            if r.building_id.0 != filter {
                continue;
            }
        }

        let inputs: Vec<String> = r
            .inputs
            .iter()
            .map(|i| format!("{}×{}", i.resource_id, i.amount))
            .collect();
        let outputs: Vec<String> = r
            .outputs
            .iter()
            .map(|o| format!("{}×{}", o.resource_id, o.amount))
            .collect();

        let research = r
            .research_required
            .as_ref()
            .map(|r| r.0.as_str())
            .unwrap_or("—");

        table.add_row(vec![
            &r.id.0,
            &r.name,
            &r.building_id.0,
            &format!("{:.0}", r.duration),
            &inputs.join(", "),
            &outputs.join(", "),
            research,
        ]);
    }
    println!("{table}");
}

fn cmd_population(session: &mut Session, count: u32, housing_tier: Option<u32>) {
    if let Some(tier) = housing_tier {
        if !(1..=4).contains(&tier) {
            println!("住宅等級須為 1-4，使用預設等級 {}", session.population.housing_tier);
        } else {
            session.population.housing_tier = tier;
        }
    }
    session.population.population = count;

    let result = compute_island_balance(
        &[],
        &session.population,
        &session.game_data,
        &session.difficulty,
    );

    println!();
    println!(
        "═══ 人口需求: {} 人 / 住宅等級 {} ═══",
        count, session.population.housing_tier
    );
    println!();
    display::print_balance_sheet(&result.balance_sheet);
}

fn cmd_list_research(session: &Session) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["ID", "名稱", "English", "前置研究", "狀態"]);

    let mut research: Vec<_> = session.game_data.research.values().collect();
    research.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    for r in research {
        let prereqs: Vec<String> = r.prerequisites.iter().map(|p| p.0.clone()).collect();
        let status = if session.unlocked_research.contains(&r.id) {
            "✓ 已解鎖"
        } else {
            "✗ 未解鎖"
        };

        table.add_row(vec![
            &r.id.0,
            &r.name,
            &r.name_en,
            &if prereqs.is_empty() { "—".into() } else { prereqs.join(", ") },
            status,
        ]);
    }
    println!("{table}");
}
