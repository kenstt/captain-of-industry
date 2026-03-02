use serde::{Deserialize, Serialize};

use super::ids::ResearchId;

/// 研究節點定義
///
/// 每個建築和配方各有獨立的解鎖研究節點，無統一科技等級。
/// 例：「鐵冶煉（廢料）」解鎖高爐，「銅冶煉」解鎖高爐的銅水配方。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Research {
    pub id: ResearchId,
    pub name: String,
    pub name_en: String,
    /// 此研究的前置研究
    #[serde(default)]
    pub prerequisites: Vec<ResearchId>,
}
