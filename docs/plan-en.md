# Captain of Industry Production Calculator — Design Plan

## Background

The existing Rust project has basic single-recipe calculation logic (target output → machine count + input/output rates). It needs to be expanded into a full recursive production chain calculator with resource balance sheets, gap analysis with building suggestions, population demands, difficulty weights, edict effects, and JSON-driven game data.

Reference: https://wiki.coigame.com/

---

## 1. Module Structure

```
src/
  main.rs                  -- Load data + start CLI
  lib.rs                   -- Public API re-exports
  model/
    mod.rs                 -- re-exports
    ids.rs                 -- ResourceId, BuildingId, RecipeId, VehicleId, ResearchId
    resource.rs            -- Resource, ResourceCategory
    building.rs            -- Building, MaintenanceCost, Footprint, BuildingCategory
    recipe.rs              -- Recipe, Ingredient (with electricity multiplier)
    vehicle.rs             -- Vehicle
    research.rs            -- Research
    difficulty.rs          -- DifficultySettings
    edict.rs               -- Edict, EdictEffect (policy system)
    results.rs             -- ProductionNode, ProductionChain, BalanceSheet, GapSuggestion
    cargo_ship.rs          -- CargoShip (cargo ship consumption model)
  data/
    mod.rs                 -- GameData struct
    loader.rs              -- JSON loading + validation
  engine/
    mod.rs                 -- Engine public API
    solver.rs              -- Recursive chain solver + cycle detection
    balance.rs             -- BalanceSheet operations
    gap.rs                 -- Gap analysis + building suggestions
    population.rs          -- Population demand calculator
  cli/
    mod.rs                 -- REPL main loop
    commands.rs            -- Command parsing
    display.rs             -- Table formatting (Traditional Chinese output)
  error.rs                 -- Error types

data/
  resources.json
  buildings/{mining,smelting,manufacturing,food,power,waste,housing,services,farming,storage,research,logistics}.json
  recipes/{smelting,manufacturing,food,power,waste,farming,chemical,research}.json
  vehicles.json
  cargo_ships.json
  research.json
  population.json
  edicts.json
  difficulty_presets.json
```

---

## 2. Core Data Model

### 2.1 ResourceCategory

```
RawMaterial      — Ores and raw materials (mining, quarrying)
Intermediate     — Semi-processed (steel billets, glass mix, etc.)
FinalProduct     — End products (Construction Parts I-IV, Vehicle Parts, etc.)
Waste            — Byproducts (Slag, Wastewater, Exhaust)
Maintenance      — Maintenance I / II / III (non-interchangeable)
Fuel             — Fuels (Diesel, Heavy Oil, Hydrogen)
Electricity      — Power (KW, virtual resource)
Computing        — Computing power (TFlops, virtual resource)
Unity            — Unity (virtual resource with production/consumption balance)
Food             — Food products
Service          — Services (healthcare, goods, etc.)
Housing          — Housing
MoltenMaterial   — Molten materials (transported via Molten Channel)
```

Each Resource has:
- `is_primary: bool` — Extraction resources; production may exceed consumption
- `is_waste: bool` — Waste products (require processing equipment, e.g., Exhaust → Smokestack → Air Pollution)
- `is_pollution: bool` — Final environmental pollution (Air/Water Pollution quality values, listed only, not consumed)
- `is_virtual: bool` — Electricity/Computing/Unity virtual resources

**Waste vs Pollution distinction**:
- **Waste**: Recipe byproducts (Exhaust, Wastewater) — actual resource flows that need processing equipment to consume
- **Pollution**: Final output of processing equipment (Air Pollution, Water Pollution) — environmental quality metrics, listed only

### 2.2 Building

```rust
Building {
    id, name, name_en, category,
    footprint: { width, height },
    construction_costs: Vec<(ResourceId, f64)>,
    workers: u32,
    base_electricity_kw: f64,               // Base power consumption
    computing_tflops: f64,                  // Computing requirement (0 = none)
    maintenance: Option<MaintenanceCost>,    // tier + amount_per_month + idle_fraction(0.33)
    unity_consumption_per_month: f64,       // Unity consumption (e.g., Research Lab)
    available_recipes: Vec<RecipeId>,
    research_required: Option<ResearchId>,   // Specific research node that unlocks this building
    unity_boost: Option<f64>,               // Production boost from Unity spending
}
```

### 2.3 Recipe — With Electricity Multiplier

```rust
Recipe {
    id, name, name_en,
    inputs: Vec<Ingredient>,
    outputs: Vec<Ingredient>,
    duration: f64,                  // seconds
    building_id: BuildingId,
    is_default: bool,
    electricity_multiplier: f64,    // Default 1.0; some recipes change building power draw
    research_required: Option<ResearchId>,  // Research that unlocks this recipe (independent of building unlock)
}
```

**Key design**:
- **Independent recipe unlocks**: Building a facility does NOT mean all its recipes are available. For example, after unlocking the Blast Furnace, only the basic Molten Iron recipe is available; the Molten Copper recipe requires separate "Copper Smelting" research. The solver must check that BOTH `building.research_required` and `recipe.research_required` are unlocked.
- **Electricity multiplier**: Some buildings' electricity consumption changes based on the active recipe. For example, certain recipes draw 1.5x or 2x the building's base power. Actual power = `building.base_electricity_kw * recipe.electricity_multiplier`.

### 2.4 DifficultySettings

```rust
DifficultySettings {
    maintenance_multiplier: f64,            // Default 1.0
    fuel_multiplier: f64,                   // Default 1.0
    food_consumption_multiplier: f64,       // Default 1.0
    goods_services_multiplier: f64,         // Default 1.0
    conveyor_power_enabled: bool,           // Whether conveyors consume power
    storage_power_enabled: bool,            // Whether storage consumes power
}
```

**Conveyor/Storage power**: The game allows toggling whether conveyors and storage facilities consume electricity. When enabled, their power draw is included in the total electricity balance.

### 2.5 Edict (Policy System)

```rust
EdictEffect {
    target: EdictTarget,        // What it affects
    modifier_type: ModifierType, // Multiply / Add
    value: f64,                 // e.g., -0.15 = 15% reduction
}

enum EdictTarget {
    FoodConsumption,
    VehicleFuel,
    ShipFuel,
    MaintenanceConsumption,
    TruckCapacity,
    TruckMaintenance,
    RecyclingEfficiency,
    FarmYield,
    FarmWater,
    WaterConsumption,
    SolarOutput,
    PopulationGrowth,
    HealthPoints,
    UnityFromGoods,
    HouseholdGoodsConsumption,
    HouseholdAppliancesConsumption,
    ConsumerElectronicsConsumption,
}

Edict {
    id, name, name_en,
    unity_cost_per_month: f64,   // Positive = produces, Negative = consumes
    effects: Vec<EdictEffect>,
    research_required: Option<ResearchId>,
}
```

**Policy effects**: Active edicts apply as additional multipliers to their respective calculations:
- "Fuel Saver I": Vehicle fuel × 0.85
- "Food Saver I": Food consumption × 0.80
- "Maintenance Reducer I": Maintenance consumption × 0.85
- "Plenty of Food I": Food consumption × 1.25 but generates Unity

### 2.6 CargoShip

```rust
CargoShip {
    size: u32,                          // Module count (2/4/6/8)
    fuel_type: ResourceId,              // diesel / heavy_oil / hydrogen
    fuel_per_trip: f64,                 // Varies by size
    fuel_per_trip_save_mode: f64,       // Save fuel mode
    capacity_unit: u32,                 // Unit/loose capacity (360/module)
    capacity_fluid: u32,               // Fluid capacity (440/module)
    workers: u32,                       // 12-36 based on size
    travel_time_normal: f64,            // 180 seconds
    travel_time_save_fuel: f64,         // 360 seconds
    maintenance: MaintenanceCost,
}
```

**Cargo ship consumption in balance sheet**:
- Fuel per trip → converted to per-minute consumption rate
- Crew count added to total worker demand
- Maintenance consumption added to maintenance balance

### 2.7 Unity Balance

Unity is tracked independently for production and consumption:

**Sources (production)**:
- Settlement service satisfaction (food, goods, healthcare) → ~1-2/month per satisfied resource
- "Plenty of Food" and similar edicts that generate Unity
- Specific buildings/landmarks

**Sinks (consumption)**:
- Outposts (Quartz Mine, Oil Rig, etc.)
- Research Lab I (direct consumption)
- Edict costs (most edicts consume 0.5-2 Unity/month)
- Building boosts (-0.25/month per boosted building)
- Quick trade/contracts

### 2.8 Computing Balance

**Production**:
- Mainframe Computer: +8 TFlops
- Data Center: 0-192 TFlops

**Consumption**:
- Assembly IV (3), Assembly V (6)
- Crystallizer (4), Diamond Reactor (2)
- Lens Polisher (4), Microchip Machine (4/12)
- Research Lab IV (12), Nuclear Reprocessing Plant (16)

### 2.9 Pollution Handling

**Two-stage model**: Waste is an actual resource flow that is converted by processing equipment into final pollution values.

```
Recipe produces waste → Processing equipment consumes waste → Outputs final pollution (environmental quality value)
```

**Example**:
- Blast Furnace produces 24 Exhaust/min
- Smokestack recipe: consumes 60 Exhaust per 60s → produces 30 Air Pollution
- 1 Smokestack capacity = 60 Exhaust/min > 24 Exhaust/min → **no bottleneck**
- Actual operating ratio = 24/60 = 0.4
- Actual Air Pollution output = 30 × 0.4 = **12 Air Pollution/min**

Calculator approach:
1. **Waste participates in balance calculation** — has production (recipe byproducts) and consumption (processing equipment input)
2. **Processing capacity may exceed waste production** — equipment runs at partial load, calculated proportionally
3. **Final pollution calculated proportionally** — processing equipment pollution output × (actual waste input / full-load capacity)
4. **Final pollution (Pollution) listed only** — Air/Water Pollution are environmental quality values, not consumed resources
5. **If processing < production** — flagged as waste bottleneck, gap analysis suggests more processing equipment

---

## 3. Core Algorithm

### 3.1 Recursive Production Chain Solver

```
solve(target_resource, target_rate_per_min, context):
  1. If primary resource (is_primary=true) → add to balance sheet as raw input, return
  2. If pollution (is_pollution=true) → list production value only, return
  3. Find all recipes producing target_resource → filter by unlocked research (BOTH building AND recipe must be unlocked)
  4. Select best recipe (user preference > is_default > alphabetical)
  5. Calculate machines_needed and per-minute input/output rates
  6. Calculate actual power = building.base_electricity_kw * recipe.electricity_multiplier * machines_needed
  7. Calculate maintenance = base * difficulty.maintenance_multiplier * edict_multipliers
  8. Calculate computing = building.computing_tflops * machines_needed
  9. Calculate unity consumption = building.unity_consumption * machines_needed
  10. Create ProductionNode
  11. Recursively solve each input (with cycle detection via HashSet)
  12. Accumulate all consumption to balance sheet
  13. Return ProductionChain
```

### 3.2 Multiplier Stacking Order

```
Final multiplier = difficulty_multiplier × Π(edict_multipliers)

Example: Maintenance consumption
  base × difficulty.maintenance_multiplier × edict_reducer_1(0.85) × edict_reducer_2(0.90)
```

### 3.3 Cycle Detection

When encountering a resource already being solved, record it as a feedback loop deficit rather than recursing infinitely. Example:
- Power plant consumes fuel → fuel needs transport → transport consumes power → back to power plant

### 3.4 Multi-Output Handling

When a recipe produces byproducts (e.g., slag, exhaust):
- All outputs credited to balance sheet
- If a byproduct is consumed elsewhere in the chain, reduce independent production needs
- Second pass reconciles byproduct surpluses

---

## 4. Balance Sheet

Tracks per-minute production/consumption of all resources:

| Category | Resource | Produced/min | Consumed/min | Net | Status |
|----------|----------|-------------|-------------|-----|--------|
| Raw | Iron Ore | — | 120.0 | -120.0 | Needs mining |
| Intermediate | Molten Iron | 60.0 | 60.0 | 0.0 | Balanced |
| Power | KW | 500 | 450 | +50 | Surplus |
| Maintenance I | Maint. I | 160 | 120 | +40 | Surplus |
| Computing | TFlops | 192 | 28 | +164 | Surplus |
| Unity | Unity | 8.0 | 5.5 | +2.5 | Surplus |
| Pollution | Exhaust | 48 | — | 48 | (Listed only) |
| Workers | Workers | 500 | 380 | +120 | Surplus |

Special handling:
- **Electricity**: Includes conveyor/storage power (toggleable via difficulty settings)
- **Maintenance I/II/III**: Tracked separately, non-interchangeable
- **Unity**: Includes edict costs, building boosts, outpost costs
- **Computing**: Production vs consumption balance
- **Pollution**: Listed only, not counted in consumption balance
- **Cargo ships**: Fuel consumption + crew count included

---

## 5. Gap Analysis + Building Suggestions

For every deficit (net < 0) in the balance sheet:

1. Find all recipes that produce the deficit resource
2. Filter by unlocked research (only suggest buildings the player has researched)
3. Calculate machines needed to fill the gap
4. Score and rank: efficiency > fewer workers > smaller footprint
5. Suggest the best option (with cascading effects noted)

Optional: Iterative solving — fill gap → recompute balance → fill new gaps → repeat until only raw materials remain

---

## 6. CLI Interactive Commands

```
Welcome to Captain of Industry Production Calculator!

> calculate <resource_id> <rate>      — Calculate full production chain
> population <pop_count>              — Calculate population demands
> balance                             — Show global resource balance sheet
> gaps                                — Analyze gaps, suggest buildings
> difficulty <preset|custom>          — Set difficulty multipliers
> unlock <research_id>                — Mark research as unlocked (determines available buildings)
> unlocked                            — Show unlocked/locked research
> prefer <resource> <recipe>          — Set recipe preference
> edict <edict_id> [on|off]           — Enable/disable edict
> ship <size> <fuel_type> <trips>     — Add cargo ship consumption
> resources / buildings / recipes     — Query game data
> help                                — Show help
> quit                                — Exit
```

Architecture: engine/model layers have zero dependency on CLI. Future GUI (egui/Tauri/WASM) can import engine+model directly.

---

## 7. Implementation Phases

### Phase 1: Foundation — Restructure + Data Model + JSON Loader
- Restructure `src/` into module layout
- Define all model structs (including electricity_multiplier, computing, unity, edict, cargo_ship)
- Create `data/` JSON files (start with iron chain)
- Implement `data/loader.rs`
- **Verify**: Unit tests confirm deserialization

### Phase 2: Recursive Production Chain Solver
- `engine/solver.rs` (multi-output, cycle detection, unlocked research filtering)
- `engine/balance.rs` (BalanceSheet accumulation/merge)
- Migrate existing Calculator logic
- **Verify**: Iron/copper/steel chains produce correct results

### Phase 3: Difficulty + Maintenance + Power + Computing + Unity
- DifficultySettings applied (including conveyor/storage power toggle)
- Maintenance I/II/III tracked separately
- Power KW balance (with recipe.electricity_multiplier)
- Computing TFlops balance
- Unity production/consumption tracking
- Worker count totals
- **Verify**: Different difficulty multipliers produce different results

### Phase 4: Edict System
- Implement Edict model and effect stacking
- Integrate edict multipliers into solver
- Unity costs from edicts in balance sheet
- **Verify**: Fuel Saver edict correctly reduces fuel consumption

### Phase 5: CLI Interactive Interface
- Add `rustyline` + `comfy-table` dependencies
- REPL loop + command parsing + table output
- Output in Traditional Chinese
- **Verify**: Interactive `calculate iron_plate 60` works

### Phase 6: Population Demands
- `engine/population.rs`
- `data/population.json`
- Integrate into balance sheet (food/housing/services/unity output)
- **Verify**: `population 500` shows demands

### Phase 7: Gap Analysis + Building Suggestions
- `engine/gap.rs`
- Unlocked research filtering + efficiency scoring
- Iterative gap resolution
- **Verify**: Reasonable suggestions

### Phase 8: Cargo Ships + Vehicles + Pollution Listing
- Cargo ship fuel/crew in balance sheet
- Vehicle fuel/maintenance in balance sheet
- Waste output listed (not consumed)
- **Verify**: Full balance sheet includes logistics costs

### Phase 9: Data Completion + Polish
- Complete all JSON game data files
- Difficulty presets
- Error messages + edge cases
- **Verify**: Real game scenario integration tests

---

## 8. Dependencies

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2"
rustyline = "15"
comfy-table = "7"
```

---

## 9. Key Design Decisions

| Item | Decision | Rationale |
|------|----------|-----------|
| Rate unit | Per minute | Community standard; 1 game month = 60s @1x speed |
| Machine count | f64 for calc + ceil for display | Precise calculation + practical build count |
| Power multiplier | recipe.electricity_multiplier | Some recipes change building power draw |
| Conveyor/storage power | Boolean toggle in difficulty | Game difficulty option |
| Waste | Participates in balance calc | Exhaust/Wastewater are actual resource flows needing processing |
| Pollution | Proportionally calculated, listed only | Processing equipment output, environmental quality metric |
| Tech filtering | Unlocked research set | No unified tier; each building has independent research unlock |
| Unity | Full production/consumption tracking | Edicts, boosts, outposts all consume it |
| Computing | TFlops balance tracking | Required by late-game buildings |
| Edicts | Multipliers stacked onto respective calculations | Multiple edicts can be active simultaneously |
| Cargo ships | Fuel + crew in balance | Logistics is significant resource consumption |
| UI separation | Engine has no CLI dependency | Future Web/GUI extensibility |
