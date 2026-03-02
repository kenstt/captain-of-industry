use comfy_table::{Cell, Color, ContentArrangement, Table};

use crate::model::resource::ResourceCategory;
use crate::model::results::{BalanceSheet, ProductionChain};

/// 顯示生產鏈結果
pub fn print_chain(chain: &ProductionChain) {
    println!();
    println!(
        "═══ 生產鏈: {} — 目標 {:.1}/分 ═══",
        chain.target_resource, chain.target_rate_per_min
    );
    println!();

    // 生產節點表格
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["建築", "配方", "需要台數", "實際台數", "工人"]);

    for node in &chain.nodes {
        table.add_row(vec![
            Cell::new(&node.building_name),
            Cell::new(&node.recipe_name),
            Cell::new(format!("{:.2}", node.machines_needed)),
            Cell::new(node.machines_actual.to_string()),
            Cell::new(node.workers.to_string()),
        ]);
    }
    println!("{table}");

    // 總工人數
    let total_workers: u32 = chain.nodes.iter().map(|n| n.workers).sum();
    println!("總工人數: {total_workers}");
    println!();

    // 平衡表
    print_balance_sheet(&chain.balance_sheet);
}

/// 顯示資源平衡表
pub fn print_balance_sheet(balance: &BalanceSheet) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["類別", "資源", "產出/分", "消耗/分", "淨值", "狀態"]);

    for (_id, entry) in balance.sorted_entries() {
        let net = entry.net_per_min();
        let status = if entry.is_raw_input {
            "需供給"
        } else if entry.category == ResourceCategory::Pollution {
            "環境值"
        } else if net.abs() < f64::EPSILON {
            "平衡"
        } else if net > 0.0 {
            "盈餘"
        } else {
            "赤字"
        };

        let status_color = match status {
            "赤字" | "需供給" => Color::Red,
            "盈餘" => Color::Green,
            "平衡" => Color::White,
            "環境值" => Color::Yellow,
            _ => Color::White,
        };

        let category_label = category_name(&entry.category);

        table.add_row(vec![
            Cell::new(category_label),
            Cell::new(&entry.resource_name),
            Cell::new(format_rate(entry.produced_per_min)),
            Cell::new(format_rate(entry.consumed_per_min)),
            Cell::new(format_net(net)),
            Cell::new(status).fg(status_color),
        ]);
    }

    println!("── 資源平衡表 ──");
    println!("{table}");
}

/// 顯示說明
pub fn print_help() {
    println!(
        r#"
《工業隊長》生產計算機 — 指令列表

  計算 <resource_id> <rate>   計算完整生產鏈
  平衡                         顯示當前資源平衡表
  缺口                         分析赤字並建議建築

  難度                         顯示當前難度設定
  難度 <key> <value>           設定難度（maintenance/fuel/food/goods <倍率>）
  難度 conveyor|storage on|off 開關傳送帶/倉儲耗電

  解鎖 <research_id>           標記研究為已解鎖
  鎖定 <research_id>           標記研究為未解鎖
  解鎖列表                     顯示已解鎖研究

  偏好 <resource_id> <recipe>  指定配方偏好

  資源列表                     列出所有資源
  建築列表                     列出所有建築
  配方列表 [building_id]       列出配方
  研究列表                     列出研究節點

  幫助                         顯示此說明
  離開                         結束程式

支援中/英文指令（如 calculate = 計算, balance = 平衡）
"#
    );
}

fn format_rate(rate: f64) -> String {
    if rate.abs() < f64::EPSILON {
        "—".to_string()
    } else {
        format!("{:.2}", rate)
    }
}

fn format_net(net: f64) -> String {
    if net.abs() < f64::EPSILON {
        "0".to_string()
    } else if net > 0.0 {
        format!("+{:.2}", net)
    } else {
        format!("{:.2}", net)
    }
}

fn category_name(category: &ResourceCategory) -> &'static str {
    match category {
        ResourceCategory::RawMaterial => "原料",
        ResourceCategory::Intermediate => "中間",
        ResourceCategory::FinalProduct => "成品",
        ResourceCategory::MoltenMaterial => "熔融",
        ResourceCategory::Food => "食物",
        ResourceCategory::Fuel => "燃料",
        ResourceCategory::Electricity => "電力",
        ResourceCategory::Computing => "算力",
        ResourceCategory::Unity => "凝聚力",
        ResourceCategory::Maintenance => "維護",
        ResourceCategory::Service => "服務",
        ResourceCategory::Housing => "住宅",
        ResourceCategory::Waste => "廢棄物",
        ResourceCategory::Pollution => "污染",
    }
}
