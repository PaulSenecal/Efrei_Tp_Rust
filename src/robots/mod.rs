mod robot_trait;
mod explorer;
mod harvester;
mod scientist;
mod movement;

pub use robot_trait::{Robot, RobotType, RobotStatus, RobotMemory, RobotConfig};
pub use explorer::Explorer;
pub use harvester::{EnergyHarvester, MineralHarvester};
pub use scientist::ScientistRobot;
pub use movement::{Direction, Position};

use serde::{Serialize, Deserialize};
use crate::map::{Map, ResourceType};
use std::sync::Arc;
use std::sync::Mutex;

/// Gestionnaire de simulation autonome des robots
pub struct RobotManager {
    robots: Vec<Box<dyn Robot>>,
    next_id: u32,
    base_position: Position,
    known_resources: Arc<Mutex<Vec<(Position, String)>>>,
    simulation_tick: u64,
}

impl std::fmt::Debug for RobotManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RobotManager")
            .field("robot_count", &self.robots.len())
            .field("next_id", &self.next_id)
            .field("base_position", &self.base_position)
            .field("simulation_tick", &self.simulation_tick)
            .finish()
    }
}

impl RobotManager {
    /// Crée un nouveau gestionnaire avec une base
    pub fn new(base_x: usize, base_y: usize) -> Self {
        RobotManager {
            robots: Vec::new(),
            next_id: 1,
            base_position: Position::new(base_x, base_y),
            known_resources: Arc::new(Mutex::new(Vec::new())),
            simulation_tick: 0,
        }
    }
    
    /// Ajoute un robot exploreur
    pub fn add_explorer(&mut self, x: usize, y: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let explorer = Explorer::new(id, Position::new(x, y), self.base_position);
        self.robots.push(Box::new(explorer));
        id
    }
    
    /// Ajoute un robot récolteur d'énergie
    pub fn add_energy_harvester(&mut self, x: usize, y: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let harvester = EnergyHarvester::new(id, Position::new(x, y), self.base_position);
        self.robots.push(Box::new(harvester));
        id
    }
    
    /// Ajoute un robot récolteur de minerais
    pub fn add_mineral_harvester(&mut self, x: usize, y: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let harvester = MineralHarvester::new(id, Position::new(x, y), self.base_position);
        self.robots.push(Box::new(harvester));
        id
    }
    
    /// Ajoute un robot scientifique
    pub fn add_scientist(&mut self, x: usize, y: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let scientist = ScientistRobot::new(id, Position::new(x, y), self.base_position);
        self.robots.push(Box::new(scientist));
        id
    }
    
    /// Fait avancer la simulation d'un tick - tous les robots agissent
    pub fn simulate_tick(&mut self, map: &mut Map) -> Vec<String> {
        self.simulation_tick += 1;
        let mut actions = Vec::new();
        
        // Mettre à jour les ressources connues avec celles des exploreurs
        for robot in &self.robots {
            if matches!(robot.get_type(), RobotType::Explorer) {
                let memory = robot.get_memory();
                let mut known = self.known_resources.lock().unwrap();
                for resource in memory.known_resources {
                    if !known.contains(&resource) {
                        known.push(resource);
                    }
                }
            }
        }
        
        // Faire agir chaque robot
        let known_resources = self.known_resources.lock().unwrap().clone();
        for robot in &mut self.robots {
            if robot.get_status() != RobotStatus::OutOfEnergy {
                let action = robot.think_and_act(map, &known_resources);
                actions.push(action);
            } else {
                actions.push(format!("Robot #{} est hors service (plus d'énergie)", robot.get_id()));
            }
        }
        
        actions
    }
    
    /// Obtient l'état de tous les robots pour l'API
    pub fn get_all_robots_state(&self) -> Vec<RobotState> {
        self.robots.iter()
            .map(|robot| RobotState {
                id: robot.get_id(),
                robot_type: robot.get_type(),
                position: robot.get_position(),
                status: robot.get_status(),
                energy: robot.get_energy(),
                cargo: robot.get_cargo(),
                base_position: robot.get_base_position(),
            })
            .collect()
    }
    
    /// Obtient l'état d'un robot spécifique
    pub fn get_robot_state(&self, robot_id: u32) -> Option<RobotState> {
        self.robots.iter()
            .find(|r| r.get_id() == robot_id)
            .map(|robot| RobotState {
                id: robot.get_id(),
                robot_type: robot.get_type(),
                position: robot.get_position(),
                status: robot.get_status(),
                energy: robot.get_energy(),
                cargo: robot.get_cargo(),
                base_position: robot.get_base_position(),
            })
    }
    
    /// Obtient les statistiques de la simulation
    pub fn get_statistics(&self) -> SimulationStats {
        let total_energy_collected: u32 = self.robots.iter()
            .filter(|r| matches!(r.get_type(), RobotType::EnergyHarvester))
            .flat_map(|r| r.get_memory().known_resources)
            .filter(|(_, res)| res.starts_with("Energy"))
            .count() as u32;
            
        let total_minerals_collected: u32 = self.robots.iter()
            .filter(|r| matches!(r.get_type(), RobotType::MineralHarvester))
            .flat_map(|r| r.get_memory().known_resources)
            .filter(|(_, res)| res.starts_with("Mineral"))
            .count() as u32;
            
        let science_points_visited: u32 = self.robots.iter()
            .filter(|r| matches!(r.get_type(), RobotType::ScientistRobot))
            .flat_map(|r| r.get_memory().known_resources)
            .count() as u32;
            
        let explored_tiles: usize = self.robots.iter()
            .filter(|r| matches!(r.get_type(), RobotType::Explorer))
            .flat_map(|r| r.get_memory().explored_tiles)
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        SimulationStats {
            tick: self.simulation_tick,
            total_robots: self.robots.len(),
            active_robots: self.robots.iter().filter(|r| r.can_act()).count(),
            total_energy_collected,
            total_minerals_collected,
            science_points_visited,
            explored_tiles,
        }
    }
}

/// État sérialisable d'un robot pour l'API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RobotState {
    pub id: u32,
    pub robot_type: RobotType,
    pub position: Position,
    pub status: RobotStatus,
    pub energy: u32,
    pub cargo: Vec<ResourceType>,
    pub base_position: Position,
}

/// Statistiques de la simulation
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationStats {
    pub tick: u64,
    pub total_robots: usize,
    pub active_robots: usize,
    pub total_energy_collected: u32,
    pub total_minerals_collected: u32,
    pub science_points_visited: u32,
    pub explored_tiles: usize,
}

