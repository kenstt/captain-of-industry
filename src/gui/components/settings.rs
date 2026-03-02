use dioxus::prelude::*;

use crate::model::*;
use crate::gui::state::AppState;

#[component]
pub fn Settings() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    // 讀取當前值
    let difficulty = app_state.read().difficulty.clone();
    let unlocked = app_state.read().unlocked_research.clone();

    // 研究列表
    let research_list: Vec<(String, String, String, bool)> = {
        let state = app_state.read();
        let mut list: Vec<_> = state.game_data.research.values()
            .map(|r| {
                let is_unlocked = unlocked.contains(&r.id);
                (r.id.0.clone(), r.name.clone(), r.name_en.clone(), is_unlocked)
            })
            .collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    };

    // 配方偏好列表
    let preferences: Vec<(String, String)> = {
        let state = app_state.read();
        let mut prefs: Vec<_> = state.recipe_preferences.iter()
            .map(|(r, p)| (r.0.clone(), p.0.clone()))
            .collect();
        prefs.sort_by(|a, b| a.0.cmp(&b.0));
        prefs
    };

    rsx! {
        div { class: "panel settings-panel",
            h2 { "設定 Settings" }

            // ── 難度設定 ──
            div { class: "settings-section",
                h3 { "難度倍率 Difficulty Multipliers" }

                div { class: "form-row",
                    label { "維護 Maintenance:" }
                    input {
                        r#type: "number",
                        value: "{difficulty.maintenance_multiplier}",
                        min: "0",
                        step: "0.1",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                app_state.write().difficulty.maintenance_multiplier = v;
                            }
                        },
                    }
                }

                div { class: "form-row",
                    label { "燃油 Fuel:" }
                    input {
                        r#type: "number",
                        value: "{difficulty.fuel_multiplier}",
                        min: "0",
                        step: "0.1",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                app_state.write().difficulty.fuel_multiplier = v;
                            }
                        },
                    }
                }

                div { class: "form-row",
                    label { "食物 Food:" }
                    input {
                        r#type: "number",
                        value: "{difficulty.food_consumption_multiplier}",
                        min: "0",
                        step: "0.1",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                app_state.write().difficulty.food_consumption_multiplier = v;
                            }
                        },
                    }
                }

                div { class: "form-row",
                    label { "商品服務 Goods:" }
                    input {
                        r#type: "number",
                        value: "{difficulty.goods_services_multiplier}",
                        min: "0",
                        step: "0.1",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                app_state.write().difficulty.goods_services_multiplier = v;
                            }
                        },
                    }
                }

                div { class: "form-row",
                    label { "傳送帶耗電 Conveyor Power:" }
                    input {
                        r#type: "checkbox",
                        checked: difficulty.conveyor_power_enabled,
                        onchange: move |e| {
                            app_state.write().difficulty.conveyor_power_enabled = e.checked();
                        },
                    }
                }

                div { class: "form-row",
                    label { "倉儲耗電 Storage Power:" }
                    input {
                        r#type: "checkbox",
                        checked: difficulty.storage_power_enabled,
                        onchange: move |e| {
                            app_state.write().difficulty.storage_power_enabled = e.checked();
                        },
                    }
                }
            }

            // ── 研究解鎖 ──
            div { class: "settings-section",
                h3 { "研究解鎖 Research Unlocks" }
                p { class: "hint", "未勾選任何研究時，所有配方皆可用。" }

                div { class: "research-list",
                    for (id, name, name_en, is_unlocked) in research_list.iter() {
                        div { class: "research-item",
                            label {
                                input {
                                    r#type: "checkbox",
                                    checked: *is_unlocked,
                                    onchange: {
                                        let id = id.clone();
                                        move |e: Event<FormData>| {
                                            let rid = ResearchId::new(&id);
                                            if e.checked() {
                                                app_state.write().unlocked_research.insert(rid);
                                            } else {
                                                app_state.write().unlocked_research.remove(&rid);
                                            }
                                        }
                                    },
                                }
                                " {name} ({name_en}) [{id}]"
                            }
                        }
                    }
                }
            }

            // ── 配方偏好 ──
            div { class: "settings-section",
                h3 { "配方偏好 Recipe Preferences" }
                if preferences.is_empty() {
                    p { class: "hint", "尚未設定任何偏好。" }
                } else {
                    table { class: "data-table",
                        thead {
                            tr {
                                th { "資源 Resource" }
                                th { "偏好配方 Preferred Recipe" }
                                th { "操作" }
                            }
                        }
                        tbody {
                            for (res_id, recipe_id) in preferences.iter() {
                                tr {
                                    td { "{res_id}" }
                                    td { "{recipe_id}" }
                                    td {
                                        button {
                                            class: "btn-small btn-danger",
                                            onclick: {
                                                let res_id = res_id.clone();
                                                move |_| {
                                                    app_state.write().recipe_preferences
                                                        .remove(&ResourceId::new(&res_id));
                                                }
                                            },
                                            "移除"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
