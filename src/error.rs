use thiserror::Error;

use crate::model::ids::{BuildingId, RecipeId, ResearchId, ResourceId};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("找不到資源: {0}")]
    ResourceNotFound(ResourceId),

    #[error("找不到建築: {0}")]
    BuildingNotFound(BuildingId),

    #[error("找不到配方: {0}")]
    RecipeNotFound(RecipeId),

    #[error("找不到研究: {0}")]
    ResearchNotFound(ResearchId),

    #[error("無可用配方產出資源 {0}（可能尚未解鎖）")]
    NoAvailableRecipe(ResourceId),

    #[error("偵測到循環依賴: {0}")]
    CycleDetected(ResourceId),

    #[error("資料驗證失敗: {0}")]
    ValidationError(String),

    #[error("JSON 讀取錯誤: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("檔案讀取錯誤: {0}")]
    IoError(#[from] std::io::Error),
}
