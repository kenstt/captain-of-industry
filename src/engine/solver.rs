use std::collections::HashSet;
use std::sync::Arc;

use crate::data::GameData;
use crate::error::AppError;
use crate::model::*;

/// 求解器設定
pub struct SolverSettings {
    pub difficulty: DifficultySettings,
    /// 已解鎖的研究節點
    pub unlocked_research: HashSet<ResearchId>,
    /// 使用者指定的配方偏好（資源 ID → 配方 ID）
    pub recipe_preferences: std::collections::HashMap<ResourceId, RecipeId>,
}

impl Default for SolverSettings {
    fn default() -> Self {
        Self {
            difficulty: DifficultySettings::default(),
            unlocked_research: HashSet::new(),
            recipe_preferences: std::collections::HashMap::new(),
        }
    }
}

/// 生產鏈求解引擎
pub struct Engine {
    game_data: Arc<GameData>,
}

impl Engine {
    pub fn new(game_data: GameData) -> Self {
        Self {
            game_data: Arc::new(game_data),
        }
    }

    pub fn from_arc(game_data: Arc<GameData>) -> Self {
        Self { game_data }
    }

    /// 取得遊戲資料的參考
    pub fn game_data(&self) -> &GameData {
        &self.game_data
    }

    /// 求解完整生產鏈
    ///
    /// 從目標資源和目標速率出發，遞迴計算所需的所有建築、原料和副產物。
    pub fn solve_chain(
        &self,
        target: &ResourceId,
        rate_per_min: f64,
        settings: &SolverSettings,
    ) -> Result<ProductionChain, AppError> {
        let mut nodes = Vec::new();
        let mut balance = BalanceSheet::new();
        let mut solving_stack = HashSet::new();

        self.solve_recursive(
            target,
            rate_per_min,
            settings,
            &mut nodes,
            &mut balance,
            &mut solving_stack,
        )?;

        Ok(ProductionChain {
            target_resource: target.clone(),
            target_rate_per_min: rate_per_min,
            nodes,
            balance_sheet: balance,
        })
    }

    /// 遞迴求解單一資源的生產需求
    fn solve_recursive(
        &self,
        target: &ResourceId,
        rate_per_min: f64,
        settings: &SolverSettings,
        nodes: &mut Vec<ProductionNode>,
        balance: &mut BalanceSheet,
        solving_stack: &mut HashSet<ResourceId>,
    ) -> Result<(), AppError> {
        let resource = self
            .game_data
            .resources
            .get(target)
            .ok_or_else(|| AppError::ResourceNotFound(target.clone()))?;

        let meta = EntryMeta {
            name: resource.name.clone(),
            name_en: resource.name_en.clone(),
            category: resource.category.clone(),
        };

        // 原料：僅記錄需求，不遞迴
        if resource.is_primary {
            balance.add_consumption(target, rate_per_min, meta);
            balance.mark_raw_input(target);
            return Ok(());
        }

        // 最終污染：僅列出數值
        if resource.is_pollution {
            balance.add_production(target, rate_per_min, meta);
            return Ok(());
        }

        // 循環偵測
        if solving_stack.contains(target) {
            // 記錄為回饋迴路缺口，不繼續遞迴
            balance.add_consumption(target, rate_per_min, meta);
            return Ok(());
        }
        solving_stack.insert(target.clone());

        // 找出可用配方（建築和配方都需已解鎖）
        let recipe = self.find_best_recipe(target, settings)?;
        let building = self
            .game_data
            .buildings
            .get(&recipe.building_id)
            .ok_or_else(|| AppError::BuildingNotFound(recipe.building_id.clone()))?;

        // 計算此配方對目標資源的每分鐘單機產量
        let target_output_per_cycle = recipe
            .outputs
            .iter()
            .find(|o| o.resource_id == *target)
            .map(|o| o.amount)
            .unwrap_or(0.0);

        let cycles_per_min = 60.0 / recipe.duration;
        let single_machine_rate = target_output_per_cycle * cycles_per_min;
        let machines_needed = rate_per_min / single_machine_rate;
        let machines_actual = machines_needed.ceil() as u32;

        // 計算所有輸入/輸出的每分鐘速率
        let inputs_per_min: Vec<Ingredient> = recipe
            .inputs
            .iter()
            .map(|i| Ingredient {
                resource_id: i.resource_id.clone(),
                amount: i.amount * cycles_per_min * machines_needed,
            })
            .collect();

        let outputs_per_min: Vec<Ingredient> = recipe
            .outputs
            .iter()
            .map(|o| Ingredient {
                resource_id: o.resource_id.clone(),
                amount: o.amount * cycles_per_min * machines_needed,
            })
            .collect();

        // 建築耗電（含配方倍率）
        let electricity_kw = building.base_electricity_kw
            * recipe.electricity_multiplier
            * machines_needed;

        // 算力需求
        let computing_tflops = building.computing_tflops * machines_needed;

        // 凝聚力消耗（每月→每分鐘，遊戲 1 月 = 1 分鐘 @1x）
        let unity_per_min = building.unity_consumption_per_month * machines_needed;

        // 維護消耗
        let maintenance = building.maintenance.as_ref().map(|m| {
            let amount = m.amount_per_month
                * machines_needed
                * settings.difficulty.maintenance_multiplier;
            (m.tier.clone(), amount)
        });

        // 工人數
        let workers = building.workers * machines_actual;

        // 建立節點
        let node = ProductionNode {
            recipe_id: recipe.id.clone(),
            building_id: building.id.clone(),
            building_name: building.name.clone(),
            recipe_name: recipe.name.clone(),
            machines_needed,
            machines_actual,
            inputs_per_min: inputs_per_min.clone(),
            outputs_per_min: outputs_per_min.clone(),
            electricity_kw,
            maintenance: maintenance.clone(),
            computing_tflops,
            unity_per_min,
            workers,
        };
        nodes.push(node);

        // 更新平衡表：所有輸出
        for output in &outputs_per_min {
            let out_resource = self.game_data.resources.get(&output.resource_id);
            let out_meta = EntryMeta {
                name: out_resource.map_or_else(
                    || output.resource_id.0.clone(),
                    |r| r.name.clone(),
                ),
                name_en: out_resource.map_or_else(
                    || output.resource_id.0.clone(),
                    |r| r.name_en.clone(),
                ),
                category: out_resource.map_or(
                    ResourceCategory::Intermediate,
                    |r| r.category.clone(),
                ),
            };
            balance.add_production(&output.resource_id, output.amount, out_meta);
        }

        // 更新平衡表：電力消耗
        if electricity_kw > 0.0 {
            balance.add_consumption(
                &ResourceId::new("electricity"),
                electricity_kw,
                EntryMeta {
                    name: "電力".into(),
                    name_en: "Electricity".into(),
                    category: ResourceCategory::Electricity,
                },
            );
        }

        // 更新平衡表：算力消耗
        if computing_tflops > 0.0 {
            balance.add_consumption(
                &ResourceId::new("computing"),
                computing_tflops,
                EntryMeta {
                    name: "算力".into(),
                    name_en: "Computing".into(),
                    category: ResourceCategory::Computing,
                },
            );
        }

        // 更新平衡表：凝聚力消耗
        if unity_per_min > 0.0 {
            balance.add_consumption(
                &ResourceId::new("unity"),
                unity_per_min,
                EntryMeta {
                    name: "凝聚力".into(),
                    name_en: "Unity".into(),
                    category: ResourceCategory::Unity,
                },
            );
        }

        // 更新平衡表：維護消耗
        if let Some((ref tier, amount)) = maintenance {
            let maint_id = match tier {
                MaintenanceTier::Maintenance1 => ResourceId::new("maintenance_1"),
                MaintenanceTier::Maintenance2 => ResourceId::new("maintenance_2"),
                MaintenanceTier::Maintenance3 => ResourceId::new("maintenance_3"),
            };
            let tier_label = match tier {
                MaintenanceTier::Maintenance1 => ("維護 I", "Maintenance I"),
                MaintenanceTier::Maintenance2 => ("維護 II", "Maintenance II"),
                MaintenanceTier::Maintenance3 => ("維護 III", "Maintenance III"),
            };
            balance.add_consumption(
                &maint_id,
                amount,
                EntryMeta {
                    name: tier_label.0.into(),
                    name_en: tier_label.1.into(),
                    category: ResourceCategory::Maintenance,
                },
            );
        }

        // 遞迴求解每個輸入
        for input in &inputs_per_min {
            // 先記錄消耗
            let in_resource = self.game_data.resources.get(&input.resource_id);
            let in_meta = EntryMeta {
                name: in_resource
                    .map_or_else(|| input.resource_id.0.clone(), |r| r.name.clone()),
                name_en: in_resource
                    .map_or_else(|| input.resource_id.0.clone(), |r| r.name_en.clone()),
                category: in_resource
                    .map_or(ResourceCategory::Intermediate, |r| r.category.clone()),
            };
            balance.add_consumption(&input.resource_id, input.amount, in_meta);

            // 遞迴求解輸入來源
            self.solve_recursive(
                &input.resource_id,
                input.amount,
                settings,
                nodes,
                balance,
                solving_stack,
            )?;
        }

        solving_stack.remove(target);
        Ok(())
    }

    /// 找出目標資源的最佳可用配方
    ///
    /// 優先順序：使用者偏好 > is_default > 字母序
    /// 前提：建築和配方的 research_required 都必須已解鎖
    fn find_best_recipe(
        &self,
        target: &ResourceId,
        settings: &SolverSettings,
    ) -> Result<Recipe, AppError> {
        // 收集所有能產出此資源的配方
        let mut candidates: Vec<&Recipe> = self
            .game_data
            .recipes
            .values()
            .filter(|r| r.outputs.iter().any(|o| o.resource_id == *target))
            .filter(|r| self.is_recipe_available(r, settings))
            .collect();

        if candidates.is_empty() {
            return Err(AppError::NoAvailableRecipe(target.clone()));
        }

        // 使用者偏好優先
        if let Some(preferred_id) = settings.recipe_preferences.get(target) {
            if let Some(preferred) = candidates.iter().find(|r| r.id == *preferred_id) {
                return Ok((*preferred).clone());
            }
        }

        // is_default 優先，再按 id 字母序穩定排序
        candidates.sort_by(|a, b| {
            b.is_default
                .cmp(&a.is_default)
                .then_with(|| a.id.0.cmp(&b.id.0))
        });

        Ok(candidates[0].clone())
    }

    /// 檢查配方是否可用（建築和配方的研究都已解鎖）
    pub fn is_recipe_available(&self, recipe: &Recipe, settings: &SolverSettings) -> bool {
        // 如果沒有設定任何解鎖研究，視為全部解鎖（方便測試）
        if settings.unlocked_research.is_empty() {
            return true;
        }

        // 檢查配方本身的研究
        if let Some(ref req) = recipe.research_required {
            if !settings.unlocked_research.contains(req) {
                return false;
            }
        }

        // 檢查建築的研究
        if let Some(building) = self.game_data.buildings.get(&recipe.building_id) {
            if let Some(ref req) = building.research_required {
                if !settings.unlocked_research.contains(req) {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_game_data;
    use std::path::PathBuf;

    fn load_test_data() -> GameData {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        load_game_data(&data_dir).expect("載入測試資料")
    }

    #[test]
    fn test_solve_iron_plate_chain() {
        let engine = Engine::new(load_test_data());

        // 僅解鎖 BF1 相關研究，確保使用高爐而非高爐 II
        let settings = SolverSettings {
            unlocked_research: HashSet::from([ResearchId::new("iron_smelting_scrap")]),
            ..Default::default()
        };

        // 目標：每分鐘 12 鐵板
        let chain = engine
            .solve_chain(&ResourceId::new("iron_plate"), 12.0, &settings)
            .expect("應成功求解");

        // 應至少有鑄造機和高爐兩個節點
        assert!(chain.nodes.len() >= 2, "生產鏈至少需要 2 個節點");

        // 鑄造鐵板：12/分 = 1 台金屬鑄造機（12 鐵水 → 12 鐵板 / 60秒）
        let caster = chain
            .nodes
            .iter()
            .find(|n| n.building_id == BuildingId::new("metal_caster"))
            .expect("應有金屬鑄造機");
        assert!(
            (caster.machines_needed - 1.0).abs() < 0.01,
            "鑄造機應需 1 台，實際: {}",
            caster.machines_needed
        );

        // 高爐：12 鐵水/分 → 0.5 台（bf_molten_iron_scrap: 24 鐵水 / 60秒 = 24/分/台）
        let furnace = chain
            .nodes
            .iter()
            .find(|n| n.building_id == BuildingId::new("blast_furnace"))
            .expect("應有高爐");
        assert!(
            (furnace.machines_needed - 0.5).abs() < 0.01,
            "高爐應需 0.5 台，實際: {}",
            furnace.machines_needed
        );

        // 平衡表應有鐵板產出
        let iron_plate_entry = chain
            .balance_sheet
            .entries
            .get(&ResourceId::new("iron_plate"))
            .expect("平衡表應有鐵板");
        assert!(
            (iron_plate_entry.produced_per_min - 12.0).abs() < 0.01,
            "鐵板產出應為 12/分"
        );
    }

    #[test]
    fn test_research_filter() {
        let engine = Engine::new(load_test_data());

        // 只解鎖鐵冶煉（廢料），不解鎖銅冶煉
        let settings = SolverSettings {
            unlocked_research: HashSet::from([ResearchId::new("iron_smelting_scrap")]),
            ..Default::default()
        };

        // 鐵板應可求解（鐵廢料配方已解鎖）
        let result = engine.solve_chain(&ResourceId::new("iron_plate"), 12.0, &settings);
        assert!(result.is_ok(), "鐵板應可求解");

        // 銅板應失敗（銅冶煉未解鎖）
        let result = engine.solve_chain(&ResourceId::new("copper_plate"), 12.0, &settings);
        assert!(result.is_err(), "銅板應因研究未解鎖而失敗");
    }

    #[test]
    fn test_waste_in_balance_sheet() {
        let engine = Engine::new(load_test_data());
        let settings = SolverSettings {
            unlocked_research: HashSet::from([ResearchId::new("iron_smelting_scrap")]),
            ..Default::default()
        };

        let chain = engine
            .solve_chain(&ResourceId::new("iron_plate"), 12.0, &settings)
            .expect("應成功求解");

        // 平衡表應有廢氣產出
        let exhaust = chain
            .balance_sheet
            .entries
            .get(&ResourceId::new("exhaust"));
        assert!(exhaust.is_some(), "平衡表應有廢氣");
        assert!(
            exhaust.unwrap().produced_per_min > 0.0,
            "廢氣產出應 > 0"
        );
    }

    #[test]
    fn test_difficulty_affects_maintenance() {
        let engine = Engine::new(load_test_data());

        // 使用高爐 II（有維護消耗）
        let all_research = HashSet::from([
            ResearchId::new("iron_smelting_scrap"),
            ResearchId::new("iron_smelting_ore"),
            ResearchId::new("copper_smelting"),
            ResearchId::new("advanced_smelting"),
        ]);

        // 標準難度
        let normal = SolverSettings {
            unlocked_research: all_research.clone(),
            ..Default::default()
        };

        // 高難度（維護 ×2）
        let hard = SolverSettings {
            difficulty: DifficultySettings {
                maintenance_multiplier: 2.0,
                ..Default::default()
            },
            unlocked_research: all_research.clone(),
            ..Default::default()
        };

        let chain_normal = engine
            .solve_chain(&ResourceId::new("molten_iron"), 16.0, &normal)
            .expect("標準難度應成功");
        let chain_hard = engine
            .solve_chain(&ResourceId::new("molten_iron"), 16.0, &hard)
            .expect("高難度應成功");

        let maint_normal = chain_normal
            .balance_sheet
            .entries
            .get(&ResourceId::new("maintenance_1"))
            .map(|e| e.consumed_per_min)
            .unwrap_or(0.0);

        let maint_hard = chain_hard
            .balance_sheet
            .entries
            .get(&ResourceId::new("maintenance_1"))
            .map(|e| e.consumed_per_min)
            .unwrap_or(0.0);

        assert!(
            (maint_hard - maint_normal * 2.0).abs() < 0.01,
            "高難度維護應為標準的 2 倍: normal={maint_normal}, hard={maint_hard}"
        );
    }

    #[test]
    fn test_total_workers() {
        let engine = Engine::new(load_test_data());
        let settings = SolverSettings {
            unlocked_research: HashSet::from([ResearchId::new("iron_smelting_scrap")]),
            ..Default::default()
        };

        let chain = engine
            .solve_chain(&ResourceId::new("iron_plate"), 12.0, &settings)
            .expect("應成功求解");

        let total_workers: u32 = chain.nodes.iter().map(|n| n.workers).sum();
        // 鑄造機 1 台 × 4 工人 + 高爐 1 台(ceil 0.5) × 8 工人 = 12
        assert_eq!(total_workers, 12, "總工人數應為 12");
    }
}
