use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::data::GameData;
use crate::engine::solver::Engine;
use crate::model::*;

/// GUI 共享應用狀態
#[derive(Clone)]
pub struct AppState {
    pub game_data: Arc<GameData>,
    pub engine: Arc<Engine>,
    pub difficulty: DifficultySettings,
    pub unlocked_research: HashSet<ResearchId>,
    pub recipe_preferences: HashMap<ResourceId, RecipeId>,
    pub last_chain: Option<Arc<ProductionChain>>,
    pub active_tab: Tab,
    pub island_entries: Vec<IslandEntry>,
    pub population: PopulationSettings,
    pub next_entry_id: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Calculator,
    Balance,
    Settings,
    DataBrowser,
}

impl AppState {
    pub fn new(game_data: GameData) -> Self {
        let game_data = Arc::new(game_data);
        let engine = Arc::new(Engine::from_arc(game_data.clone()));
        let mut population = PopulationSettings::default();
        population.needs = PopulationSettings::build_default_needs(&game_data.population_data);
        Self {
            game_data,
            engine,
            difficulty: DifficultySettings::default(),
            unlocked_research: HashSet::new(),
            recipe_preferences: HashMap::new(),
            last_chain: None,
            active_tab: Tab::Calculator,
            island_entries: Vec::new(),
            population,
            next_entry_id: 1,
        }
    }

    /// 新增一筆島嶼帳本條目，回傳分配的 ID
    pub fn add_island_entry(&mut self, building_id: BuildingId, recipe_id: RecipeId, count: f64) -> u32 {
        let id = self.next_entry_id;
        self.next_entry_id += 1;
        self.island_entries.push(IslandEntry {
            id,
            building_id,
            recipe_id,
            count,
        });
        id
    }

    /// 刪除指定 ID 的條目
    pub fn remove_island_entry(&mut self, entry_id: u32) {
        self.island_entries.retain(|e| e.id != entry_id);
    }
}
