use std::sync::Arc;

use dioxus::prelude::*;

use crate::engine::solver::SolverSettings;
use crate::model::*;
use crate::gui::state::AppState;

#[component]
pub fn Calculator() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let mut selected_resource = use_signal(|| String::new());
    let mut rate_input = use_signal(|| "12".to_string());
    let mut search_filter = use_signal(|| String::new());
    let mut error_msg = use_signal(|| Option::<String>::None);

    let resource_list: Vec<(String, String, String)> = {
        let state = app_state.read();
        let filter = search_filter.read().to_lowercase();
        let mut list: Vec<_> = state.game_data.resources.values()
            .filter(|r| {
                if filter.is_empty() {
                    return true;
                }
                r.id.0.to_lowercase().contains(&filter)
                    || r.name.to_lowercase().contains(&filter)
                    || r.name_en.to_lowercase().contains(&filter)
            })
            .map(|r| (r.id.0.clone(), r.name.clone(), r.name_en.clone()))
            .collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    };

    let on_calculate = move |_| {
        let resource_id_str = selected_resource.read().clone();
        if resource_id_str.is_empty() {
            error_msg.set(Some("請選擇資源".into()));
            return;
        }
        let rate: f64 = match rate_input.read().parse() {
            Ok(v) if v > 0.0 => v,
            _ => {
                error_msg.set(Some("請輸入有效的正數速率".into()));
                return;
            }
        };

        let state = app_state.read();
        let settings = SolverSettings {
            difficulty: state.difficulty.clone(),
            unlocked_research: state.unlocked_research.clone(),
            recipe_preferences: state.recipe_preferences.clone(),
        };

        match state.engine.solve_chain(&ResourceId::new(&resource_id_str), rate, &settings) {
            Ok(chain) => {
                drop(state);
                error_msg.set(None);
                app_state.write().last_chain = Some(Arc::new(chain));
            }
            Err(e) => {
                error_msg.set(Some(format!("求解失敗: {e}")));
            }
        }
    };

    let chain = app_state.read().last_chain.clone();

    rsx! {
        div { class: "panel calculator-panel",
            h2 { "生產計算 Production Calculator" }

            div { class: "form-row",
                label { "搜尋資源 Filter:" }
                input {
                    r#type: "text",
                    placeholder: "輸入資源 ID 或名稱...",
                    value: "{search_filter}",
                    oninput: move |e| search_filter.set(e.value()),
                }
            }

            div { class: "form-row",
                label { "目標資源 Target:" }
                select {
                    value: "{selected_resource}",
                    onchange: move |e| selected_resource.set(e.value()),
                    option { value: "", "-- 選擇資源 --" }
                    for (id, name, name_en) in resource_list.iter() {
                        option {
                            value: "{id}",
                            selected: *id == *selected_resource.read(),
                            "{name} ({name_en}) [{id}]"
                        }
                    }
                }
            }

            div { class: "form-row",
                label { "速率 Rate (/min):" }
                input {
                    r#type: "number",
                    value: "{rate_input}",
                    oninput: move |e| rate_input.set(e.value()),
                    min: "0.01",
                    step: "0.1",
                }
            }

            button {
                class: "btn-primary",
                onclick: on_calculate,
                "計算 Calculate"
            }

            if let Some(err) = &*error_msg.read() {
                div { class: "error-msg", "{err}" }
            }

            if let Some(chain) = &chain {
                div { class: "results",
                    h3 { "生產鏈 Production Chain — {chain.target_resource} @ {chain.target_rate_per_min:.2}/min" }

                    table { class: "data-table",
                        thead {
                            tr {
                                th { "建築 Building" }
                                th { "配方 Recipe" }
                                th { "需求台數" }
                                th { "實際台數" }
                                th { "耗電 KW" }
                                th { "工人" }
                            }
                        }
                        tbody {
                            for node in chain.nodes.iter() {
                                tr {
                                    td { "{node.building_name}" }
                                    td { "{node.recipe_name}" }
                                    td { "{node.machines_needed:.2}" }
                                    td { "{node.machines_actual}" }
                                    td { "{node.electricity_kw:.1}" }
                                    td { "{node.workers}" }
                                }
                            }
                        }
                    }

                    {
                        let total_workers: u32 = chain.nodes.iter().map(|n| n.workers).sum();
                        let total_electricity: f64 = chain.nodes.iter().map(|n| n.electricity_kw).sum();
                        rsx! {
                            div { class: "summary",
                                span { "總工人: {total_workers}" }
                                span { " | 總耗電: {total_electricity:.1} KW" }
                            }
                        }
                    }

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
                        "匯入平衡表 Import to Balance"
                    }
                }
            }
        }
    }
}
