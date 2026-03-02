use serde::{Deserialize, Serialize};

use super::ids::ResourceId;

/// 資源分類
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ResourceCategory {
    /// 原礦/原料（採礦、採石等）
    RawMaterial,
    /// 中間產物（鋼胚、玻璃混合料等）
    Intermediate,
    /// 最終產品（建設零件 I-IV、車輛零件等）
    FinalProduct,
    /// 廢棄物（礦渣、廢水、廢氣）— 實際資源流，需處理設備消耗
    Waste,
    /// 維護 I / II / III（三者不可替代）
    Maintenance,
    /// 燃料（柴油、重油、氫氣）
    Fuel,
    /// 電力（KW 虛擬資源）
    Electricity,
    /// 算力（TFlops 虛擬資源）
    Computing,
    /// 凝聚力（虛擬資源，有生產/消耗平衡）
    Unity,
    /// 食物
    Food,
    /// 服務（醫療、商品等）
    Service,
    /// 住宅
    Housing,
    /// 熔融材料（使用熔道運輸）
    MoltenMaterial,
    /// 最終環境污染（空汙、水汙等品質數值，僅列出不消耗）
    Pollution,
}

/// 遊戲資源定義
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resource {
    pub id: ResourceId,
    /// 繁體中文名稱
    pub name: String,
    /// 英文名稱
    pub name_en: String,
    pub category: ResourceCategory,
    /// 採礦類資源，允許產出 > 消耗
    #[serde(default)]
    pub is_primary: bool,
    /// 廢棄物，需要處理設備消耗（如廢氣→煙囪）
    #[serde(default)]
    pub is_waste: bool,
    /// 最終環境污染，僅列出不計入消耗平衡
    #[serde(default)]
    pub is_pollution: bool,
    /// 虛擬資源（電力/算力/凝聚力）
    #[serde(default)]
    pub is_virtual: bool,
}
