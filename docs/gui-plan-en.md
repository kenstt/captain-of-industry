# GUI Implementation Plan — Dioxus Desktop App

## Background

The engine/model layer (recursive solver, balance sheet, difficulty system) and CLI interface are complete. Adding a Dioxus GUI as a second frontend, directly reusing the engine layer without affecting the existing CLI.

## Architecture

```
src/
  main.rs          -- Route to CLI or GUI based on args
  gui/
    mod.rs         -- Dioxus launch entry point
    app.rs         -- Main app component (Tab navigation)
    state.rs       -- Shared state (Signal<AppState>)
    components/
      mod.rs
      calculator.rs   -- Calculator panel (select resource + rate → solve)
      balance.rs      -- Balance sheet display
      settings.rs     -- Difficulty/research unlock/preference settings
      data_browser.rs -- Resource/building/recipe/research browser
    styles.css        -- Global CSS (industrial dark theme)
```

## Dependencies

```toml
dioxus = { version = "0.6", features = ["desktop"] }
```

Dioxus desktop uses system WebView for rendering, bundle < 5MB.

## Shared State (state.rs)

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

Engine and GameData loaded once at startup, shared via Dioxus `use_context`.

## Pages

### 1. Calculator (calculator.rs)
- Resource dropdown (with search filter)
- Rate input field
- "Calculate" button
- Results: production chain node table + balance sheet

### 2. Balance Sheet (balance.rs)
- Grouped by category, color-coded (surplus green / deficit red / raw yellow / pollution gray)
- Total workers / electricity / maintenance summary

### 3. Settings (settings.rs)
- Difficulty multiplier sliders
- Conveyor/storage power toggles
- Research unlock checkboxes
- Recipe preference settings

### 4. Data Browser (data_browser.rs)
- Resources/buildings/recipes/research in tabbed tables
- Search/filter

## Launch

```
cargo run              # Default: launch GUI
cargo run -- --cli     # Launch CLI
```

## Implementation Steps

1. Add dioxus to Cargo.toml
2. src/gui/state.rs — State definition
3. src/gui/mod.rs — Launch entry
4. src/gui/app.rs — Main app + Tabs
5. src/gui/components/calculator.rs — Calculator panel
6. src/gui/components/balance.rs — Balance sheet
7. src/gui/components/settings.rs — Settings panel
8. src/gui/components/data_browser.rs — Data browser
9. src/gui/styles.css — CSS styling
10. src/main.rs — CLI/GUI routing
