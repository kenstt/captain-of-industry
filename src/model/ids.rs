use serde::{Deserialize, Serialize};
use std::fmt;

/// 建立具名 ID 型別的巨集，避免重複樣板程式碼
macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
    };
}

define_id!(ResourceId, "資源 ID");
define_id!(BuildingId, "建築 ID");
define_id!(RecipeId, "配方 ID");
define_id!(VehicleId, "車輛 ID");
define_id!(ResearchId, "研究 ID");
define_id!(EdictId, "政策 ID");
