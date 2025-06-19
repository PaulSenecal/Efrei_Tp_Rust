use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    Energy(u32),
    Mineral(u32),
    Science,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub x: usize,
    pub y: usize,
}

impl ResourceType {
    pub fn name(&self) -> &'static str {
        match self {
            ResourceType::Energy(_) => "Énergie",
            ResourceType::Mineral(_) => "Minerai",
            ResourceType::Science => "Point scientifique",
        }
    }
    
    pub fn value(&self) -> u32 {
        match self {
            ResourceType::Energy(v) | ResourceType::Mineral(v) => *v,
            ResourceType::Science => 1,
        }
    }
    
    pub fn is_consumable(&self) -> bool {
        matches!(self, ResourceType::Energy(_) | ResourceType::Mineral(_))
    }
}