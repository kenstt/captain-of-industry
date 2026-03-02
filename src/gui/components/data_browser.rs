use dioxus::prelude::*;

use crate::gui::state::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BrowserTab {
    Resources,
    Buildings,
    Recipes,
    Research,
}

#[component]
pub fn DataBrowser() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut active_tab = use_signal(|| BrowserTab::Resources);
    let mut search = use_signal(|| String::new());

    let filter = search.read().to_lowercase();

    rsx! {
        div { class: "panel data-browser-panel",
            h2 { "資料瀏覽 Data Browser" }

            div { class: "tab-bar sub-tabs",
                button {
                    class: if *active_tab.read() == BrowserTab::Resources { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(BrowserTab::Resources),
                    "資源 Resources"
                }
                button {
                    class: if *active_tab.read() == BrowserTab::Buildings { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(BrowserTab::Buildings),
                    "建築 Buildings"
                }
                button {
                    class: if *active_tab.read() == BrowserTab::Recipes { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(BrowserTab::Recipes),
                    "配方 Recipes"
                }
                button {
                    class: if *active_tab.read() == BrowserTab::Research { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(BrowserTab::Research),
                    "研究 Research"
                }
            }

            div { class: "form-row",
                input {
                    r#type: "text",
                    placeholder: "搜尋 Search...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
            }

            match *active_tab.read() {
                BrowserTab::Resources => {
                    let state = app_state.read();
                    let mut resources: Vec<_> = state.game_data.resources.values()
                        .filter(|r| {
                            filter.is_empty()
                                || r.id.0.to_lowercase().contains(&filter)
                                || r.name.to_lowercase().contains(&filter)
                                || r.name_en.to_lowercase().contains(&filter)
                        })
                        .collect();
                    resources.sort_by(|a, b| a.id.0.cmp(&b.id.0));

                    rsx! {
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "ID" }
                                    th { "名稱" }
                                    th { "English" }
                                    th { "類別" }
                                    th { "屬性" }
                                }
                            }
                            tbody {
                                for r in resources.iter() {
                                    {
                                        let mut flags = Vec::new();
                                        if r.is_primary { flags.push("原料"); }
                                        if r.is_waste { flags.push("廢棄物"); }
                                        if r.is_pollution { flags.push("污染"); }
                                        if r.is_virtual { flags.push("虛擬"); }
                                        rsx! {
                                            tr {
                                                td { "{r.id}" }
                                                td { "{r.name}" }
                                                td { "{r.name_en}" }
                                                td { "{r.category:?}" }
                                                td { "{flags.join(\", \")}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                BrowserTab::Buildings => {
                    let state = app_state.read();
                    let mut buildings: Vec<_> = state.game_data.buildings.values()
                        .filter(|b| {
                            filter.is_empty()
                                || b.id.0.to_lowercase().contains(&filter)
                                || b.name.to_lowercase().contains(&filter)
                                || b.name_en.to_lowercase().contains(&filter)
                        })
                        .collect();
                    buildings.sort_by(|a, b| a.id.0.cmp(&b.id.0));

                    rsx! {
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "ID" }
                                    th { "名稱" }
                                    th { "English" }
                                    th { "類別" }
                                    th { "工人" }
                                    th { "耗電 KW" }
                                    th { "研究需求" }
                                }
                            }
                            tbody {
                                for b in buildings.iter() {
                                    {
                                        let research = b.research_required.as_ref()
                                            .map(|r| r.0.as_str())
                                            .unwrap_or("—");
                                        rsx! {
                                            tr {
                                                td { "{b.id}" }
                                                td { "{b.name}" }
                                                td { "{b.name_en}" }
                                                td { "{b.category:?}" }
                                                td { "{b.workers}" }
                                                td { "{b.base_electricity_kw:.0}" }
                                                td { "{research}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                BrowserTab::Recipes => {
                    let state = app_state.read();
                    let mut recipes: Vec<_> = state.game_data.recipes.values()
                        .filter(|r| {
                            filter.is_empty()
                                || r.id.0.to_lowercase().contains(&filter)
                                || r.name.to_lowercase().contains(&filter)
                                || r.name_en.to_lowercase().contains(&filter)
                                || r.building_id.0.to_lowercase().contains(&filter)
                        })
                        .collect();
                    recipes.sort_by(|a, b| a.id.0.cmp(&b.id.0));

                    rsx! {
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "ID" }
                                    th { "名稱" }
                                    th { "建築" }
                                    th { "週期(s)" }
                                    th { "輸入" }
                                    th { "輸出" }
                                    th { "研究需求" }
                                }
                            }
                            tbody {
                                for r in recipes.iter() {
                                    {
                                        let inputs: String = r.inputs.iter()
                                            .map(|i| format!("{}×{}", i.resource_id, i.amount))
                                            .collect::<Vec<_>>()
                                            .join(", ");
                                        let outputs: String = r.outputs.iter()
                                            .map(|o| format!("{}×{}", o.resource_id, o.amount))
                                            .collect::<Vec<_>>()
                                            .join(", ");
                                        let research = r.research_required.as_ref()
                                            .map(|r| r.0.as_str())
                                            .unwrap_or("—");
                                        rsx! {
                                            tr {
                                                td { "{r.id}" }
                                                td { "{r.name}" }
                                                td { "{r.building_id}" }
                                                td { "{r.duration:.0}" }
                                                td { "{inputs}" }
                                                td { "{outputs}" }
                                                td { "{research}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                BrowserTab::Research => {
                    let state = app_state.read();
                    let unlocked = &state.unlocked_research;
                    let mut research: Vec<_> = state.game_data.research.values()
                        .filter(|r| {
                            filter.is_empty()
                                || r.id.0.to_lowercase().contains(&filter)
                                || r.name.to_lowercase().contains(&filter)
                                || r.name_en.to_lowercase().contains(&filter)
                        })
                        .collect();
                    research.sort_by(|a, b| a.id.0.cmp(&b.id.0));

                    rsx! {
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "ID" }
                                    th { "名稱" }
                                    th { "English" }
                                    th { "前置研究" }
                                    th { "狀態" }
                                }
                            }
                            tbody {
                                for r in research.iter() {
                                    {
                                        let prereqs = if r.prerequisites.is_empty() {
                                            "—".to_string()
                                        } else {
                                            r.prerequisites.iter()
                                                .map(|p| p.0.clone())
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        };
                                        let status = if unlocked.contains(&r.id) {
                                            "已解鎖"
                                        } else {
                                            "未解鎖"
                                        };
                                        let css = if unlocked.contains(&r.id) {
                                            "status-surplus"
                                        } else {
                                            "status-deficit"
                                        };
                                        rsx! {
                                            tr {
                                                td { "{r.id}" }
                                                td { "{r.name}" }
                                                td { "{r.name_en}" }
                                                td { "{prereqs}" }
                                                td { class: "{css}", "{status}" }
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
}
