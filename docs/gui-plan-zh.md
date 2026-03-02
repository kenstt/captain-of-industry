# GUI 實作計畫 — Dioxus 桌面應用

## 背景

已有完整的 engine/model 層（遞迴求解器、平衡表、難度系統）和 CLI 介面。加入 Dioxus GUI 作為第二個前端，直接複用 engine 層，不影響現有 CLI。

## 架構

```
src/
  main.rs          -- 依啟動參數選擇 CLI 或 GUI
  gui/
    mod.rs         -- Dioxus launch 入口
    app.rs         -- 主應用元件（Tab 切換）
    state.rs       -- 共享狀態 (Signal<AppState>)
    components/
      mod.rs
      calculator.rs   -- 計算面板（選資源+速率→求解）
      balance.rs      -- 平衡表顯示
      settings.rs     -- 難度/研究解鎖/偏好設定面板
      data_browser.rs -- 資源/建築/配方/研究瀏覽
    styles.css        -- 全局 CSS 樣式（工業風暗色主題）
```

## 依賴

```toml
dioxus = { version = "0.6", features = ["desktop"] }
```

Dioxus desktop 使用系統 WebView 渲染，打包 < 5MB。

## 共享狀態 (state.rs)

```rust
struct AppState {
    game_data: Arc<GameData>,
    engine: Engine,
    difficulty: DifficultySettings,
    unlocked_research: HashSet<ResearchId>,
    recipe_preferences: HashMap<ResourceId, RecipeId>,
    last_chain: Option<ProductionChain>,
}
```

Engine 和 GameData 在啟動時載入一次，透過 Dioxus `use_context` 共享。

## 頁面設計

### 1. 計算面板 (calculator.rs)
- 資源下拉選單（支援搜尋過濾）
- 速率輸入框
- 「計算」按鈕
- 結果：生產鏈節點表格 + 平衡表

### 2. 平衡表 (balance.rs)
- 按類別分組，顏色標示（盈餘綠/赤字紅/原料黃/污染灰）
- 總工人數/電力/維護摘要

### 3. 設定面板 (settings.rs)
- 難度倍率滑桿
- 傳送帶/倉儲耗電開關
- 研究解鎖勾選列表
- 配方偏好設定

### 4. 資料瀏覽 (data_browser.rs)
- 資源/建築/配方/研究 四個分頁表格
- 搜尋/篩選

## 啟動方式

```
cargo run              # 預設啟動 GUI
cargo run -- --cli     # 啟動 CLI
```

## 實作步驟

1. Cargo.toml 加入 dioxus
2. src/gui/state.rs — 狀態定義
3. src/gui/mod.rs — launch 入口
4. src/gui/app.rs — 主應用 + Tab
5. src/gui/components/calculator.rs — 計算面板
6. src/gui/components/balance.rs — 平衡表
7. src/gui/components/settings.rs — 設定面板
8. src/gui/components/data_browser.rs — 資料瀏覽
9. src/gui/styles.css — CSS
10. src/main.rs — CLI/GUI 分流
