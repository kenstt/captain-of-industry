use dioxus::prelude::*;

use crate::gui::state::{AppState, Tab};
use crate::gui::components::calculator::Calculator;
use crate::gui::components::balance::Balance;
use crate::gui::components::settings::Settings;
use crate::gui::components::data_browser::DataBrowser;

const CSS: &str = include_str!("styles.css");

#[component]
pub fn App() -> Element {
    // 從 LaunchBuilder 注入的 AppState 建立 Signal，供所有子元件共享
    let initial = use_context::<AppState>();
    let mut app_state = use_context_provider(|| Signal::new(initial));
    let active_tab = app_state.read().active_tab;

    rsx! {
        style { "{CSS}" }

        div { class: "app",
            header { class: "app-header",
                h1 { "工業隊長 生產計算機" }
                span { class: "subtitle", "Captain of Industry — Production Calculator" }
            }

            nav { class: "tab-bar",
                button {
                    class: if active_tab == Tab::Calculator { "tab active" } else { "tab" },
                    onclick: move |_| app_state.write().active_tab = Tab::Calculator,
                    "計算 Calc"
                }
                button {
                    class: if active_tab == Tab::Balance { "tab active" } else { "tab" },
                    onclick: move |_| app_state.write().active_tab = Tab::Balance,
                    "平衡表 Balance"
                }
                button {
                    class: if active_tab == Tab::Settings { "tab active" } else { "tab" },
                    onclick: move |_| app_state.write().active_tab = Tab::Settings,
                    "設定 Settings"
                }
                button {
                    class: if active_tab == Tab::DataBrowser { "tab active" } else { "tab" },
                    onclick: move |_| app_state.write().active_tab = Tab::DataBrowser,
                    "資料 Data"
                }
            }

            main { class: "content",
                if active_tab == Tab::Calculator {
                    Calculator {}
                } else if active_tab == Tab::Balance {
                    Balance {}
                } else if active_tab == Tab::Settings {
                    Settings {}
                } else {
                    DataBrowser {}
                }
            }
        }
    }
}
