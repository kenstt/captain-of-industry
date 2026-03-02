# 工業隊長 (Captain of Industry) 生產計算機 — 設計規劃

## 背景

現有 Rust 專案有基礎的單配方計算邏輯（目標產量 → 機器數 + 輸入/輸出速率）。需要擴展為完整的生產鏈遞迴計算器，包含資源平衡表、缺口建議、人口需求、難度權重、政策影響，並以 JSON 維護遊戲資料。

參考：https://wiki.coigame.com/

---

## 1. 模組結構

```
src/
  main.rs                  -- 載入資料 + 啟動 CLI
  lib.rs                   -- 公開 API re-exports
  model/
    mod.rs                 -- re-exports
    ids.rs                 -- ResourceId, BuildingId, RecipeId, VehicleId, ResearchId
    resource.rs            -- Resource, ResourceCategory
    building.rs            -- Building, MaintenanceCost, Footprint, BuildingCategory
    recipe.rs              -- Recipe, Ingredient（含耗電倍率）
    vehicle.rs             -- Vehicle
    research.rs            -- Research
    difficulty.rs          -- DifficultySettings
    edict.rs               -- Edict, EdictEffect（政策系統）
    results.rs             -- ProductionNode, ProductionChain, BalanceSheet, GapSuggestion
    cargo_ship.rs          -- CargoShip（貨船消耗模型）
  data/
    mod.rs                 -- GameData 結構
    loader.rs              -- JSON 讀取 + 驗證
  engine/
    mod.rs                 -- Engine 公開 API
    solver.rs              -- 遞迴生產鏈求解器 + 循環偵測
    balance.rs             -- BalanceSheet 操作
    gap.rs                 -- 缺口分析 + 建築建議
    population.rs          -- 人口需求計算
  cli/
    mod.rs                 -- REPL 主迴圈
    commands.rs            -- 指令解析
    display.rs             -- 表格格式化 (繁體中文)
  error.rs                 -- 錯誤類型

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

## 2. 核心資料模型

### 2.1 ResourceCategory（資源分類）

```
RawMaterial      — 原礦/原料（採礦、採石等）
Intermediate     — 中間產物（鋼胚、玻璃混合料等）
FinalProduct     — 最終產品（建設零件 I-IV、車輛零件等）
Waste            — 廢棄物（礦渣 Slag、廢水 Wastewater、廢氣 Exhaust）
Maintenance      — 維護 I / II / III（三者不可替代）
Fuel             — 燃料（柴油 Diesel、重油 Heavy Oil、氫氣 Hydrogen）
Electricity      — 電力（KW 虛擬資源）
Computing        — 算力（TFlops 虛擬資源）
Unity            — 凝聚力（虛擬資源，有生產/消耗平衡）
Food             — 食物
Service          — 服務（醫療、商品等）
Housing          — 住宅
MoltenMaterial   — 熔融材料（使用熔道 Molten Channel 運輸）
```

每個 Resource 有：
- `is_primary: bool` — 採礦類，允許產出 > 消耗
- `is_waste: bool` — 廢棄物（需要處理設備轉換，如廢氣→煙囪→空汙）
- `is_pollution: bool` — 最終環境污染（空汙、水汙等品質數值，僅列出不消耗）
- `is_virtual: bool` — 電力/算力/凝聚力等虛擬資源

**廢棄物 vs 污染的區別**：
- **廢棄物 (Waste)**：配方副產物（廢氣 Exhaust、廢水 Wastewater），是實際資源流，需要處理設備消耗
- **污染 (Pollution)**：處理設備的最終產出（空汙 Air Pollution、水汙 Water Pollution），是環境品質數值，僅列出

### 2.2 Building（建築）

```rust
Building {
    id, name, name_en, category,
    footprint: { width, height },
    construction_costs: Vec<(ResourceId, f64)>,
    workers: u32,
    base_electricity_kw: f64,               // 基礎耗電
    computing_tflops: f64,                  // 算力需求（0 = 不需要）
    maintenance: Option<MaintenanceCost>,    // tier + amount_per_month + idle_fraction(0.33)
    unity_consumption_per_month: f64,       // 凝聚力消耗（如研發中心）
    available_recipes: Vec<RecipeId>,
    research_required: Option<ResearchId>,   // 解鎖此建築的具體研究節點
    unity_boost: Option<f64>,               // 加速生產的凝聚力倍率
}
```

### 2.3 Recipe（配方）— 含耗電倍率

```rust
Recipe {
    id, name, name_en,
    inputs: Vec<Ingredient>,
    outputs: Vec<Ingredient>,
    duration: f64,                  // 秒
    building_id: BuildingId,
    is_default: bool,
    electricity_multiplier: f64,    // 耗電倍率，預設 1.0（部分配方會改變建築基礎耗電）
    research_required: Option<ResearchId>,  // 解鎖此配方的研究（獨立於建築解鎖）
}
```

**關鍵設計**：
- **配方獨立解鎖**：建築可建造不代表所有配方可用。例如高爐解鎖後只有基礎鐵水配方，銅水配方需額外研究「銅冶煉」才能使用。求解器需同時檢查 `building.research_required` 和 `recipe.research_required` 都已解鎖。
- **耗電倍率**：部分建築的耗電會隨配方改變。例如某些配方耗電為建築基礎耗電的 1.5x 或 2x。實際耗電 = `building.base_electricity_kw * recipe.electricity_multiplier`。

### 2.4 DifficultySettings（難度設定）

```rust
DifficultySettings {
    maintenance_multiplier: f64,            // 預設 1.0
    fuel_multiplier: f64,                   // 預設 1.0
    food_consumption_multiplier: f64,       // 預設 1.0
    goods_services_multiplier: f64,         // 預設 1.0
    conveyor_power_enabled: bool,           // 傳送帶是否耗電
    storage_power_enabled: bool,            // 倉儲是否耗電
}
```

**傳送帶/倉儲耗電**：遊戲可設定傳送帶和倉儲是否消耗電力。啟用時，這些設施的耗電會計入總電力平衡。

### 2.5 Edict（政策）

```rust
EdictEffect {
    target: EdictTarget,        // 作用目標
    modifier_type: ModifierType, // Multiply / Add
    value: f64,                 // 例如 -0.15 = 減少 15%
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
    unity_cost_per_month: f64,   // 正數=產出, 負數=消耗
    effects: Vec<EdictEffect>,
    research_required: Option<ResearchId>,
}
```

**政策影響**：啟用的政策會作為額外倍率疊加到對應的計算上。例如：
- 「節油 I」使車輛燃油 × 0.85
- 「食物節約 I」使食物消耗 × 0.80
- 「維護減少 I」使維護消耗 × 0.85
- 「豐盛飲食 I」使食物消耗 × 1.25 但產出凝聚力

### 2.6 CargoShip（貨船）

```rust
CargoShip {
    size: u32,                          // 船體模組數 (2/4/6/8)
    fuel_type: ResourceId,              // diesel / heavy_oil / hydrogen
    fuel_per_trip: f64,                 // 隨 size 變化
    fuel_per_trip_save_mode: f64,       // 省油模式
    capacity_unit: u32,                 // 單位/散裝容量 (360/模組)
    capacity_fluid: u32,               // 流體容量 (440/模組)
    workers: u32,                       // 12-36 隨 size 變化
    travel_time_normal: f64,            // 180 秒
    travel_time_save_fuel: f64,         // 360 秒
    maintenance: MaintenanceCost,
}
```

**貨船資源消耗計入平衡表**：
- 每趟燃油消耗 → 換算為每分鐘消耗率
- 船員人數計入總工人需求
- 維護消耗計入維護平衡

### 2.7 凝聚力 (Unity) 平衡

Unity 獨立追蹤生產與消耗：

**產出來源**：
- 聚落滿足服務需求（食物、商品、醫療等）→ 每項約 1-2/月
- 「豐盛飲食」等政策增加的額外凝聚力
- 特定建築/地標

**消耗來源**：
- 前哨站（採石場、油井等遠端設施）
- 研發中心 I（直接消耗）
- 政策成本（大部分政策消耗 0.5-2 凝聚力/月）
- 建築加速（每棟 -0.25/月）
- 快速貿易/合約

### 2.8 Computing（算力）平衡

**產出**：
- 大型電腦 (Mainframe Computer): +8 TFlops
- 資料中心 (Data Center): 0-192 TFlops

**消耗**：
- Assembly IV (3), Assembly V (6)
- Crystallizer (4), Diamond Reactor (2)
- Lens Polisher (4), Microchip Machine (4/12)
- Research Lab IV (12), Nuclear Reprocessing Plant (16)

### 2.9 廢棄物處理與污染計算

**二階段模型**：廢棄物是實際資源流，經過處理設備轉換為最終污染數值。

```
配方產出廢棄物 → 處理設備消耗廢棄物 → 產出最終污染（環境品質數值）
```

**範例**：
- 高爐每分鐘產出廢氣 24 單位
- 煙囪 (Smokestack) 配方：每 60 秒消耗廢氣 60 → 產出空汙 30
- 1 座煙囪處理能力 = 60 廢氣/分鐘 > 24 廢氣/分鐘 → **無瓶頸**
- 實際運轉比例 = 24/60 = 0.4
- 實際空汙產出 = 30 × 0.4 = **12 空汙/分鐘**

計算機的處理方式：
1. **廢棄物 (Waste) 作為普通資源參與平衡計算** — 有產出（配方副產物）和消耗（處理設備輸入）
2. **處理設備允許處理量 > 產出量** — 不需要滿載運行，按實際比例計算
3. **最終污染按比例計算** — 處理設備的污染產出 × (實際廢棄物輸入量 / 設備滿載處理量)
4. **最終污染 (Pollution) 僅列出** — 空汙/水汙等為環境品質數值，不作為需要消耗的資源
5. **若處理量 < 產出量** — 標示為廢棄物瓶頸，缺口分析建議增建處理設備

---

## 3. 核心演算法

### 3.1 遞迴生產鏈求解器

```
solve(target_resource, target_rate_per_min, context):
  1. 若為 primary resource（is_primary=true）→ 加入平衡表為原料需求，return
  2. 若為 pollution（is_pollution=true）→ 僅列出產出數值，return
  3. 找出所有能產出 target_resource 的配方 → 依已解鎖研究過濾（需同時滿足建築已解鎖 + 配方已解鎖）
  4. 選擇最佳配方（使用者偏好 > is_default > 字母序）
  5. 計算 machines_needed 及每分鐘 input/output 速率
  6. 計算實際耗電 = building.base_electricity_kw * recipe.electricity_multiplier * machines_needed
  7. 計算維護消耗 = base_maintenance * difficulty.maintenance_multiplier * 政策倍率
  8. 計算算力需求 = building.computing_tflops * machines_needed
  9. 計算凝聚力消耗 = building.unity_consumption * machines_needed
  10. 建立 ProductionNode
  11. 對每個 input 遞迴 solve（帶循環偵測 HashSet）
  12. 累加所有消耗到平衡表
  13. 回傳 ProductionChain
```

### 3.2 倍率疊加順序

```
最終倍率 = 難度倍率 × Π(政策倍率)

例：維護消耗
  base × difficulty.maintenance_multiplier × edict_maintenance_reducer_1(0.85) × edict_maintenance_reducer_2(0.90)
```

### 3.3 循環偵測

遇到正在求解的資源時，記錄為回饋迴路缺口，不無限遞迴。例如：
- 發電廠消耗燃料 → 燃料需要運輸 → 運輸消耗電力 → 回到發電廠

### 3.4 多產出處理

當配方產出副產物（如礦渣、廢氣）：
- 所有產出計入平衡表
- 若副產物被其他配方消耗，減少該副產物的獨立生產需求
- 二次遍歷調和副產物盈餘

---

## 4. 平衡表 (BalanceSheet)

追蹤每分鐘所有資源的產出/消耗：

| 類別 | 資源 | 產出/分 | 消耗/分 | 淨值 | 狀態 |
|------|------|--------|--------|------|------|
| 原料 | 鐵礦 | — | 120.0 | -120.0 | 需採礦 |
| 中間 | 鐵水 | 60.0 | 60.0 | 0.0 | 平衡 |
| 電力 | KW | 500 | 450 | +50 | 盈餘 |
| 維護I | 維護 I | 160 | 120 | +40 | 盈餘 |
| 算力 | TFlops | 192 | 28 | +164 | 盈餘 |
| 凝聚力 | Unity | 8.0 | 5.5 | +2.5 | 盈餘 |
| 污染 | 廢氣 | 48 | — | 48 | (僅列出) |
| 人力 | 工人 | 500 | 380 | +120 | 盈餘 |

特殊處理：
- **電力**：含傳送帶/倉儲耗電（依難度設定 on/off）
- **維護 I/II/III**：分開追蹤，不可替代
- **凝聚力**：含政策成本、建築加速、前哨站等消耗
- **算力**：產出 vs 消耗平衡
- **污染**：僅列出數值，不計入消耗平衡
- **貨船**：燃油消耗 + 船員人數計入

---

## 5. 缺口分析 + 建築建議

對平衡表中每個赤字（淨值 < 0）：

1. 查找所有能產出該資源的配方
2. 依已解鎖研究過濾（僅建議已解鎖的建築）
3. 計算所需機器數以填補缺口
4. 評分排序：效率 > 工人少 > 佔地小
5. 建議最佳選項（含連鎖影響）

可選：迭代求解 — 填補缺口 → 重算平衡 → 填補新缺口 → 直到只剩原料

---

## 6. CLI 互動指令

```
歡迎使用《工業隊長》生產計算機！

> 計算 <resource_id> <rate>         — 計算完整生產鏈
> 人口 <population>                  — 計算人口需求
> 平衡                               — 顯示全局資源平衡表
> 缺口                               — 分析缺口，建議建築
> 難度 <preset|custom>               — 設定難度倍率
> 解鎖 <research_id>                 — 標記已解鎖的研究（決定可用建築）
> 解鎖列表                            — 顯示已解鎖/未解鎖的研究
> 偏好 <resource> <recipe>           — 指定配方偏好
> 政策 <edict_id> [on|off]           — 啟用/停用政策
> 貨船 <size> <fuel_type> <trips>    — 加入貨船消耗
> 資源列表 / 建築列表 / 配方列表      — 查詢遊戲資料
> 幫助                                — 顯示說明
> 離開                                — 結束程式
```

架構設計：engine/model 層完全不依賴 CLI，未來可直接接 egui/Tauri/WASM。

---

## 7. 實作階段

### Phase 1: 基礎重構 + 資料模型 + JSON 載入
- 重構 `src/` 為模組結構
- 定義所有 model structs（含 electricity_multiplier、computing、unity、edict、cargo_ship）
- 建立 `data/` JSON 檔（從鐵鏈開始）
- 實作 `data/loader.rs`
- **驗證**: 單元測試確認反序列化正確

### Phase 2: 遞迴生產鏈求解器
- `engine/solver.rs`（含多產出、循環偵測、已解鎖研究過濾）
- `engine/balance.rs`（BalanceSheet 累加/合併）
- 遷移現有 Calculator 邏輯
- **驗證**: 鐵/銅/鋼鏈正確

### Phase 3: 難度 + 維護 + 電力 + 算力 + 凝聚力
- DifficultySettings 套用（含傳送帶/倉儲耗電開關）
- 維護 I/II/III 分別追蹤
- 電力 KW 收支（含 recipe.electricity_multiplier）
- 算力 TFlops 收支
- 凝聚力生產/消耗追蹤
- 工人數統計
- **驗證**: 不同難度倍率產生不同結果

### Phase 4: 政策系統
- 實作 Edict 模型和效果疊加
- 政策倍率整合到求解器
- 凝聚力成本計入平衡表
- **驗證**: 啟用節油政策後燃油消耗正確降低

### Phase 5: CLI 互動介面
- 新增 `rustyline` + `comfy-table` 依賴
- REPL 迴圈 + 指令解析 + 表格輸出
- 所有輸出為繁體中文
- **驗證**: 互動執行 `計算 iron_plate 60` 正確

### Phase 6: 人口需求
- `engine/population.rs`
- `data/population.json`
- 整合到平衡表（食物/住房/服務/凝聚力產出）
- **驗證**: `人口 500` 顯示需求

### Phase 7: 缺口分析 + 建築建議
- `engine/gap.rs`
- 已解鎖研究過濾 + 效率評分
- 迭代缺口解決
- **驗證**: 缺口建議合理

### Phase 8: 貨船 + 車輛 + 污染列出
- 貨船燃油/人力計入平衡表
- 車輛燃油/維護計入
- 廢棄物僅列出數值
- **驗證**: 完整平衡表含物流成本

### Phase 9: 資料補完 + 拋光
- 補齊所有 JSON 遊戲資料
- 難度預設檔
- 錯誤訊息 + 邊界情況
- **驗證**: 真實遊戲場景整合測試

---

## 8. 依賴套件

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2"
rustyline = "15"
comfy-table = "7"
```

---

## 9. 關鍵設計決策

| 項目 | 決策 | 理由 |
|------|------|------|
| 速率單位 | 每分鐘 | 遊戲社群標準，1 月 = 60 秒 @1x |
| 機器數 | f64 計算 + ceil 顯示 | 精確計算 + 實際建造數 |
| 耗電倍率 | recipe.electricity_multiplier | 部分配方改變建築耗電 |
| 傳送帶/倉儲耗電 | 難度設定 bool 開關 | 遊戲難度選項 |
| 廢棄物 | 參與平衡計算 | 廢氣/廢水是實際資源流，需處理設備消耗 |
| 污染 | 按比例計算後列出 | 處理設備產出的環境品質數值，按運轉比例計算 |
| 科技過濾 | 已解鎖研究集合 | 無統一 tier，每建築有獨立解鎖研究 |
| 凝聚力 | 完整生產/消耗追蹤 | 政策、建築加速、前哨站都消耗 |
| 算力 | TFlops 平衡追蹤 | 高階建築必需 |
| 政策 | 倍率疊加到對應計算 | 多政策可同時啟用 |
| 貨船 | 燃油+人力計入平衡 | 物流是顯著資源消耗 |
| UI 分離 | engine 不知道 CLI | 未來可接 Web/GUI |
