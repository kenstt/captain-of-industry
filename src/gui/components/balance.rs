use dioxus::prelude::*;

use crate::engine::balance::compute_island_balance;
use crate::gui::state::AppState;
use crate::model::resource::ResourceCategory;
use crate::model::*;

#[component]
pub fn Balance() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    // ── 新增建築表單狀態 ──
    let mut show_add_form = use_signal(|| false);
    let mut add_building = use_signal(|| String::new());
    let mut add_recipe = use_signal(|| String::new());
    let mut add_count = use_signal(|| "1".to_string());
    let mut building_filter = use_signal(|| String::new());

    // ── 人口展開狀態 ──
    let mut show_pop_advanced = use_signal(|| false);

    // ── 計算平衡表 ──
    let state = app_state.read();
    let island_balance = compute_island_balance(
        &state.island_entries,
        &state.population,
        &state.game_data,
        &state.difficulty,
    );
    let sorted = island_balance.balance_sheet.sorted_entries();

    // ── 建築列表（用於下拉選單）──
    let building_list: Vec<(String, String, String)> = {
        let filter = building_filter.read().to_lowercase();
        let mut list: Vec<_> = state
            .game_data
            .buildings
            .values()
            .filter(|b| {
                if filter.is_empty() {
                    return true;
                }
                b.id.0.to_lowercase().contains(&filter)
                    || b.name.to_lowercase().contains(&filter)
                    || b.name_en.to_lowercase().contains(&filter)
            })
            .map(|b| (b.id.0.clone(), b.name.clone(), b.name_en.clone()))
            .collect();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        list
    };

    // ── 選中建築的可用配方 ──
    let recipe_list: Vec<(String, String, String)> = {
        let bid_str = add_building.read().clone();
        if bid_str.is_empty() {
            Vec::new()
        } else {
            let bid = BuildingId::new(&bid_str);
            match state.game_data.buildings.get(&bid) {
                Some(building) => building
                    .available_recipes
                    .iter()
                    .filter_map(|rid| {
                        state
                            .game_data
                            .recipes
                            .get(rid)
                            .map(|r| (r.id.0.clone(), r.name.clone(), r.name_en.clone()))
                    })
                    .collect(),
                None => Vec::new(),
            }
        }
    };

    // ── 條目列表資料 ──
    let entry_rows: Vec<(u32, String, String, f64, f64, u32)> = state
        .island_entries
        .iter()
        .map(|e| {
            let bname = state
                .game_data
                .buildings
                .get(&e.building_id)
                .map(|b| b.name.clone())
                .unwrap_or_else(|| e.building_id.0.clone());
            let rname = state
                .game_data
                .recipes
                .get(&e.recipe_id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| e.recipe_id.0.clone());
            let building = state.game_data.buildings.get(&e.building_id);
            let recipe = state.game_data.recipes.get(&e.recipe_id);
            let elec = building
                .map(|b| {
                    b.base_electricity_kw
                        * recipe.map_or(1.0, |r| r.electricity_multiplier)
                        * e.count
                })
                .unwrap_or(0.0);
            let workers = building.map_or(0, |b| b.workers * (e.count.ceil() as u32));
            (e.id, bname, rname, e.count, elec, workers)
        })
        .collect();

    // ── 人口食物選項資料 ──
    let food_cat_options: Vec<(String, String, Vec<(String, String)>)> = {
        let pop_data = &state.game_data.population_data;
        let mut cats: Vec<_> = pop_data
            .food_categories
            .iter()
            .map(|(key, cat)| {
                let items: Vec<_> = cat
                    .items
                    .keys()
                    .map(|k| (k.clone(), k.clone()))
                    .collect();
                (key.clone(), cat.name.clone(), items)
            })
            .collect();
        cats.sort_by(|a, b| a.0.cmp(&b.0));
        cats
    };

    let pop_count = state.population.population;
    let housing_tier = state.population.housing_tier;
    let food_choices = state.population.food_choices.clone();

    // ── 平衡表行資料 ──
    let balance_rows: Vec<_> = sorted
        .iter()
        .map(|(id, entry)| {
            let net = entry.net_per_min();
            let (status, css_class) = if entry.is_raw_input {
                ("原料需求".to_string(), "status-raw")
            } else if net.abs() < f64::EPSILON {
                ("平衡".to_string(), "status-balanced")
            } else if net > 0.0 {
                ("盈餘".to_string(), "status-surplus")
            } else {
                ("赤字".to_string(), "status-deficit")
            };
            let cat_class = category_css_class(&entry.category);
            let name = format!("{} ({})", entry.resource_name, id);
            let category = format!("{:?}", entry.category);
            (
                name,
                category,
                cat_class,
                entry.produced_per_min,
                entry.consumed_per_min,
                net,
                status,
                css_class,
            )
        })
        .collect();

    let has_chain = state.last_chain.is_some();

    drop(state);

    rsx! {
        div { class: "panel balance-panel",
            h2 { "全島平衡表 Island Balance Sheet" }

            // ── A. 工具列 ──
            div { class: "toolbar",
                button {
                    class: "btn-primary",
                    onclick: move |_| {
                        let current = *show_add_form.read();
                        show_add_form.set(!current);
                    },
                    if *show_add_form.read() { "取消 Cancel" } else { "新增建築 Add Building" }
                }
                if has_chain {
                    button {
                        class: "btn-primary btn-import",
                        onclick: move |_| {
                            let state = app_state.read();
                            if let Some(chain) = &state.last_chain {
                                let nodes: Vec<_> = chain.nodes.iter().map(|n| {
                                    (n.building_id.clone(), n.recipe_id.clone(), n.machines_needed)
                                }).collect();
                                drop(state);
                                let mut state = app_state.write();
                                for (bid, rid, count) in nodes {
                                    state.add_island_entry(bid, rid, count);
                                }
                            }
                        },
                        "從計算匯入 Import from Calc"
                    }
                }
            }

            // ── 新增表單 ──
            if *show_add_form.read() {
                div { class: "add-form",
                    div { class: "form-row",
                        label { "搜尋建築 Filter:" }
                        input {
                            r#type: "text",
                            placeholder: "輸入建築名稱...",
                            value: "{building_filter}",
                            oninput: move |e| building_filter.set(e.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "建築 Building:" }
                        select {
                            value: "{add_building}",
                            onchange: move |e| {
                                add_building.set(e.value());
                                add_recipe.set(String::new());
                            },
                            option { value: "", "-- 選擇建築 --" }
                            for (id, name, name_en) in building_list.iter() {
                                option {
                                    value: "{id}",
                                    "{name} ({name_en})"
                                }
                            }
                        }
                    }
                    div { class: "form-row",
                        label { "配方 Recipe:" }
                        select {
                            value: "{add_recipe}",
                            onchange: move |e| add_recipe.set(e.value()),
                            option { value: "", "-- 選擇配方 --" }
                            for (id, name, name_en) in recipe_list.iter() {
                                option {
                                    value: "{id}",
                                    "{name} ({name_en})"
                                }
                            }
                        }
                    }
                    div { class: "form-row",
                        label { "數量 Count:" }
                        input {
                            r#type: "number",
                            value: "{add_count}",
                            oninput: move |e| add_count.set(e.value()),
                            min: "0.01",
                            step: "0.5",
                        }
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let bid = add_building.read().clone();
                            let rid = add_recipe.read().clone();
                            let cnt: f64 = add_count.read().parse().unwrap_or(1.0);
                            if !bid.is_empty() && !rid.is_empty() && cnt > 0.0 {
                                app_state.write().add_island_entry(
                                    BuildingId::new(&bid),
                                    RecipeId::new(&rid),
                                    cnt,
                                );
                                add_building.set(String::new());
                                add_recipe.set(String::new());
                                add_count.set("1".to_string());
                                building_filter.set(String::new());
                                show_add_form.set(false);
                            }
                        },
                        "確認 Add"
                    }
                }
            }

            // ── B. 條目列表 ──
            if !entry_rows.is_empty() {
                h3 { "建築條目 Building Entries" }
                table { class: "data-table",
                    thead {
                        tr {
                            th { "建築 Building" }
                            th { "配方 Recipe" }
                            th { "數量 Count" }
                            th { "耗電 KW" }
                            th { "工人 Workers" }
                            th { "操作" }
                        }
                    }
                    tbody {
                        for (eid, bname, rname, count, elec, workers) in entry_rows.iter() {
                            {
                                let eid = *eid;
                                let count_val = *count;
                                rsx! {
                                    tr {
                                        td { "{bname}" }
                                        td { "{rname}" }
                                        td {
                                            input {
                                                r#type: "number",
                                                class: "inline-input",
                                                value: "{count_val}",
                                                min: "0.01",
                                                step: "0.5",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<f64>() {
                                                        if v > 0.0 {
                                                            let mut state = app_state.write();
                                                            if let Some(entry) = state.island_entries.iter_mut().find(|x| x.id == eid) {
                                                                entry.count = v;
                                                            }
                                                        }
                                                    }
                                                },
                                            }
                                        }
                                        td { "{elec:.1}" }
                                        td { "{workers}" }
                                        td {
                                            button {
                                                class: "btn-small btn-danger",
                                                onclick: move |_| {
                                                    app_state.write().remove_island_entry(eid);
                                                },
                                                "刪除"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── C. 人口設定 ──
            h3 { "人口設定 Population" }
            div { class: "form-row",
                label { "人口數 Population:" }
                input {
                    r#type: "number",
                    value: "{pop_count}",
                    min: "0",
                    step: "10",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<u32>() {
                            app_state.write().population.population = v;
                        }
                    },
                }
            }
            div { class: "form-row",
                label { "住宅等級 Housing Tier:" }
                select {
                    value: "{housing_tier}",
                    onchange: move |e| {
                        if let Ok(v) = e.value().parse::<u32>() {
                            app_state.write().population.housing_tier = v;
                        }
                    },
                    option { value: "1", "I" }
                    option { value: "2", "II" }
                    option { value: "3", "III" }
                    option { value: "4", "IV" }
                }
            }

            button {
                class: "btn-small",
                style: "margin-bottom: 8px;",
                onclick: move |_| {
                    let current = *show_pop_advanced.read();
                    show_pop_advanced.set(!current);
                },
                if *show_pop_advanced.read() { "收起食物設定 ▲" } else { "食物設定 ▼" }
            }

            if *show_pop_advanced.read() {
                div { class: "food-settings",
                    for (cat_key, cat_name, items) in food_cat_options.iter() {
                        {
                            let cat_key_c = cat_key.clone();
                            let cat_key_c2 = cat_key.clone();
                            let choice = food_choices.iter().find(|c| c.category_key == *cat_key);
                            let is_enabled = choice.map_or(false, |c| c.enabled);
                            let selected_food = choice.map_or(String::new(), |c| c.food_id.clone());
                            rsx! {
                                div { class: "form-row",
                                    label {
                                        input {
                                            r#type: "checkbox",
                                            checked: is_enabled,
                                            onchange: move |e: Event<FormData>| {
                                                let mut state = app_state.write();
                                                if let Some(c) = state.population.food_choices.iter_mut().find(|c| c.category_key == cat_key_c) {
                                                    c.enabled = e.checked();
                                                }
                                            },
                                        }
                                        " {cat_name}"
                                    }
                                    select {
                                        value: "{selected_food}",
                                        onchange: move |e: Event<FormData>| {
                                            let mut state = app_state.write();
                                            if let Some(c) = state.population.food_choices.iter_mut().find(|c| c.category_key == cat_key_c2) {
                                                c.food_id = e.value();
                                            }
                                        },
                                        for (food_id, food_label) in items.iter() {
                                            option {
                                                value: "{food_id}",
                                                "{food_label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── D. 平衡表總覽 ──
            h3 { "資源平衡 Resource Balance" }

            div { class: "summary-bar",
                span { class: "summary-item", "工人: {island_balance.total_workers}" }
                span { class: "summary-item", "耗電: {island_balance.total_electricity_kw:.1} KW" }
                span { class: "summary-item", "維護: {island_balance.total_maintenance:.2}/min" }
            }

            if balance_rows.is_empty() {
                p { class: "empty-hint", "尚無條目，請新增建築或匯入計算結果。" }
            } else {
                table { class: "data-table balance-table",
                    thead {
                        tr {
                            th { "資源 Resource" }
                            th { "類別 Category" }
                            th { "產出/min" }
                            th { "消耗/min" }
                            th { "淨值 Net" }
                            th { "狀態 Status" }
                        }
                    }
                    tbody {
                        for (name, category, cat_class, produced, consumed, net, status, css_class) in balance_rows.iter() {
                            tr { class: "{css_class}",
                                td { "{name}" }
                                td { class: "cat {cat_class}", "{category}" }
                                td { "{produced:.2}" }
                                td { "{consumed:.2}" }
                                td { class: "{css_class}", "{net:.2}" }
                                td { class: "{css_class}", "{status}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn category_css_class(cat: &ResourceCategory) -> &'static str {
    match cat {
        ResourceCategory::RawMaterial => "cat-raw",
        ResourceCategory::Intermediate => "cat-intermediate",
        ResourceCategory::FinalProduct => "cat-final",
        ResourceCategory::Waste => "cat-waste",
        ResourceCategory::Pollution => "cat-pollution",
        ResourceCategory::Electricity => "cat-electricity",
        ResourceCategory::Maintenance => "cat-maintenance",
        _ => "cat-other",
    }
}
