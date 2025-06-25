use serde::{Serialize, Deserialize};
use crate::map::{Map, ResourceType};
use super::Position;

/// Types de robots disponibles
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    Explorer,
    EnergyHarvester,
    MineralHarvester,
    ScientistRobot,
}

/// États possibles d'un robot
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RobotStatus {
    Idle,              // En attente
    Exploring,         // En exploration
    MovingToTarget,    // Se dirige vers une cible
    Harvesting,        // En récolte
    ReturningToBase,   // Retour à la base
    Recharging,        // En recharge
    OutOfEnergy,       // Plus d'énergie
}

/// Comportement autonome d'un robot
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RobotBehavior {
    Explore,           // Explorer de nouvelles zones
    SeekEnergy,        // Chercher de l'énergie
    SeekMinerals,      // Chercher des minerais
    SeekScience,       // Chercher des points scientifiques
    ReturnToBase,      // Retourner à la base
}

/// Trait définissant le comportement commun à tous les robots autonomes
pub trait Robot: Send + Sync {
    /// Obtient l'identifiant unique du robot
    fn get_id(&self) -> u32;
    
    /// Obtient le type du robot
    fn get_type(&self) -> RobotType;
    
    /// Obtient la position actuelle
    fn get_position(&self) -> Position;
    
    /// Obtient le statut actuel
    fn get_status(&self) -> RobotStatus;
    
    /// Obtient le niveau d'énergie
    fn get_energy(&self) -> u32;
    
    /// Obtient le contenu du cargo
    fn get_cargo(&self) -> Vec<ResourceType>;
    
    /// Obtient la position de la base
    fn get_base_position(&self) -> Position;
    
    /// Déplace le robot vers une nouvelle position
    fn move_to(&mut self, new_position: Position);
    
    /// Décide et effectue la prochaine action de manière autonome
    fn think_and_act(&mut self, map: &mut Map, known_resources: &[(Position, String)]) -> String;
    
    /// Consomme de l'énergie
    fn consume_energy(&mut self, amount: u32) -> bool;
    
    /// Recharge l'énergie du robot
    fn recharge_energy(&mut self, amount: u32);
    
    /// Vérifie si le robot peut agir
    fn can_act(&self) -> bool {
        self.get_energy() > 0 && self.get_status() != RobotStatus::OutOfEnergy
    }
    
    /// Vérifie si le robot a besoin de recharger
    fn needs_recharge(&self) -> bool {
        self.get_energy() < 30
    }
    
    /// Obtient la mémoire du robot (pour sérialisation)
    fn get_memory(&self) -> RobotMemory;
}

#[derive(Debug, Clone, Serialize, Deserialize)]  // Ajouter Serialize, Deserialize
pub struct RobotConfig {
    pub max_energy: u32,
    pub move_cost: u32,
    pub action_cost: u32,
    pub max_cargo: usize,
    pub vision_range: usize,
    pub low_energy_threshold: u32,
}

impl Default for RobotConfig {
    fn default() -> Self {
        RobotConfig {
            max_energy: 100,
            move_cost: 1,
            action_cost: 5,
            max_cargo: 10,
            vision_range: 3,
            low_energy_threshold: 30,
        }
    }
}

/// Mémoire sérialisable d'un robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotMemory {
    pub explored_tiles: Vec<(usize, usize)>,
    pub known_resources: Vec<(Position, String)>,
    pub target_position: Option<Position>,
    pub home_position: Position,
}