/// CLI 指令解析
#[derive(Debug)]
pub enum Command {
    /// 計算 <resource_id> <rate>
    Calculate { resource_id: String, rate: f64 },
    /// 平衡
    Balance,
    /// 缺口
    Gaps,
    /// 難度 [show | maintenance|fuel|food|goods <value> | conveyor|storage on|off]
    Difficulty(DifficultyCommand),
    /// 解鎖 <research_id>
    Unlock { research_id: String },
    /// 鎖定 <research_id>
    Lock { research_id: String },
    /// 解鎖列表
    UnlockedList,
    /// 偏好 <resource_id> <recipe_id>
    Prefer { resource_id: String, recipe_id: String },
    /// 資源列表
    ListResources,
    /// 建築列表
    ListBuildings,
    /// 配方列表 [building_id]
    ListRecipes { building_id: Option<String> },
    /// 研究列表
    ListResearch,
    /// 人口 <count> [housing_tier]
    Population { count: u32, housing_tier: Option<u32> },
    /// 幫助
    Help,
    /// 離開
    Quit,
    /// 無法辨識
    Unknown(String),
}

#[derive(Debug)]
pub enum DifficultyCommand {
    Show,
    Set { key: String, value: String },
}

/// 解析使用者輸入的指令
pub fn parse_command(input: &str) -> Command {
    let input = input.trim();
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return Command::Unknown(String::new());
    }

    match parts[0] {
        "計算" | "calculate" | "calc" => {
            if parts.len() >= 3 {
                if let Ok(rate) = parts[2].parse::<f64>() {
                    return Command::Calculate {
                        resource_id: parts[1].to_string(),
                        rate,
                    };
                }
            }
            Command::Unknown("用法: 計算 <resource_id> <rate>".into())
        }
        "平衡" | "balance" => Command::Balance,
        "缺口" | "gaps" => Command::Gaps,
        "難度" | "difficulty" => {
            if parts.len() >= 3 {
                Command::Difficulty(DifficultyCommand::Set {
                    key: parts[1].to_string(),
                    value: parts[2].to_string(),
                })
            } else {
                Command::Difficulty(DifficultyCommand::Show)
            }
        }
        "解鎖" | "unlock" => {
            if parts.len() >= 2 {
                Command::Unlock {
                    research_id: parts[1].to_string(),
                }
            } else {
                Command::Unknown("用法: 解鎖 <research_id>".into())
            }
        }
        "鎖定" | "lock" => {
            if parts.len() >= 2 {
                Command::Lock {
                    research_id: parts[1].to_string(),
                }
            } else {
                Command::Unknown("用法: 鎖定 <research_id>".into())
            }
        }
        "解鎖列表" | "unlocked" => Command::UnlockedList,
        "偏好" | "prefer" => {
            if parts.len() >= 3 {
                Command::Prefer {
                    resource_id: parts[1].to_string(),
                    recipe_id: parts[2].to_string(),
                }
            } else {
                Command::Unknown("用法: 偏好 <resource_id> <recipe_id>".into())
            }
        }
        "資源列表" | "resources" => Command::ListResources,
        "建築列表" | "buildings" => Command::ListBuildings,
        "配方列表" | "recipes" => Command::ListRecipes {
            building_id: parts.get(1).map(|s| s.to_string()),
        },
        "研究列表" | "research" => Command::ListResearch,
        "人口" | "population" | "pop" => {
            if parts.len() >= 2 {
                if let Ok(count) = parts[1].parse::<u32>() {
                    let housing_tier = parts.get(2).and_then(|s| s.parse::<u32>().ok());
                    return Command::Population { count, housing_tier };
                }
            }
            Command::Unknown("用法: 人口 <人數> [住宅等級]".into())
        }
        "幫助" | "help" | "?" => Command::Help,
        "離開" | "quit" | "exit" | "q" => Command::Quit,
        _ => Command::Unknown(format!("未知指令: {}", parts[0])),
    }
}
