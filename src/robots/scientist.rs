use serde::{Serialize, Deserialize};
use crate::map::{Map, Tile, ResourceType};
use super::{Robot, RobotType, RobotStatus, Position, RobotConfig, RobotMemory, Direction};

/// Robot scientifique autonome - visite les points d'intérêt scientifique
#[derive(Debug, Serialize, Deserialize)]
pub struct ScientistRobot {
    id: u32,
    position: Position,
    base_position: Position,
    status: RobotStatus,
    energy: u32,
    config: RobotConfig,
    visited_science_points: Vec<Position>,
    target_science: Option<Position>,
    total_discoveries: u32,
}

impl ScientistRobot {
    pub fn new(id: u32, position: Position, base_position: Position) -> Self {
        let mut config = RobotConfig::default();
        config.action_cost = 15; // Analyser coûte cher
        config.move_cost = 2; // Plus lent mais méthodique
        
        ScientistRobot {
            id,
            position,
            base_position,
            status: RobotStatus::Idle,
            energy: config.max_energy,
            config,
            visited_science_points: Vec::new(),
            target_science: None,
            total_discoveries: 0,
        }
    }
    
    fn find_unvisited_science(&self, known_resources: &[(Position, String)]) -> Option<Position> {
        known_resources.iter()
            .filter(|(pos, res_type)| {
                res_type == "Science" && !self.visited_science_points.contains(pos)
            })
            .min_by_key(|(pos, _)| self.position.manhattan_distance(pos))
            .map(|(pos, _)| *pos)
    }
    
    fn calculate_move_towards(&self, target: Position, map: &Map) -> Option<Direction> {
        let dx = target.x as i32 - self.position.x as i32;
        let dy = target.y as i32 - self.position.y as i32;
        
        let mut directions = Vec::new();
        if dx.abs() >= dy.abs() {
            if dx > 0 { directions.push(Direction::East); }
            if dx < 0 { directions.push(Direction::West); }
            if dy > 0 { directions.push(Direction::South); }
            if dy < 0 { directions.push(Direction::North); }
        } else {
            if dy > 0 { directions.push(Direction::South); }
            if dy < 0 { directions.push(Direction::North); }
            if dx > 0 { directions.push(Direction::East); }
            if dx < 0 { directions.push(Direction::West); }
        }
        
        for dir in directions {
            let new_pos = self.position.move_towards(dir);
            if new_pos.x < map.width && new_pos.y < map.height {
                if let Some(tile) = map.get_tile(new_pos.x, new_pos.y) {
                    if tile != &Tile::Obstacle {
                        return Some(dir);
                    }
                }
            }
        }
        None
    }
}

impl Robot for ScientistRobot {
    fn get_id(&self) -> u32 { self.id }
    fn get_type(&self) -> RobotType { RobotType::ScientistRobot }
    fn get_position(&self) -> Position { self.position }
    fn get_status(&self) -> RobotStatus { self.status }
    fn get_energy(&self) -> u32 { self.energy }
    fn get_cargo(&self) -> Vec<ResourceType> { vec![] }
    fn get_base_position(&self) -> Position { self.base_position }
    
    fn move_to(&mut self, new_position: Position) {
        self.position = new_position;
        self.consume_energy(self.config.move_cost);
    }
    
    fn think_and_act(&mut self, map: &mut Map, known_resources: &[(Position, String)]) -> String {
        // Retourner à la base si énergie faible
        if self.needs_recharge() {
            self.status = RobotStatus::ReturningToBase;
            
            if self.position == self.base_position {
                self.recharge_energy(25);
                self.status = RobotStatus::Recharging;
                return format!("Scientist #{} se recharge (Total: {} découvertes)", 
                    self.id, self.total_discoveries);
            }
            
            if let Some(dir) = self.calculate_move_towards(self.base_position, map) {
                self.move_to(self.position.move_towards(dir));
                return format!("Scientist #{} retourne à la base", self.id);
            }
        }
        
        // Analyser le point scientifique actuel
        if let Some(Tile::SciencePoint) = map.get_tile(self.position.x, self.position.y) {
            if !self.visited_science_points.contains(&self.position) {
                self.visited_science_points.push(self.position);
                self.total_discoveries += 1;
                self.consume_energy(self.config.action_cost);
                self.status = RobotStatus::Harvesting;
                self.target_science = None;
                
                return format!("Scientist #{} analyse le point scientifique #{} en ({},{})", 
                    self.id, self.total_discoveries, self.position.x, self.position.y);
            }
        }
        
        // Chercher un nouveau point scientifique
        if self.target_science.is_none() || 
           self.visited_science_points.contains(&self.target_science.unwrap()) {
            self.target_science = self.find_unvisited_science(known_resources);
            if self.target_science.is_some() {
                self.status = RobotStatus::MovingToTarget;
            } else {
                // Tous les points visités, mission accomplie
                self.status = RobotStatus::Idle;
                return format!("Scientist #{} a visité tous les points scientifiques!", self.id);
            }
        }
        
        // Se déplacer vers la cible
        if let Some(target) = self.target_science {
            if let Some(dir) = self.calculate_move_towards(target, map) {
                self.move_to(self.position.move_towards(dir));
                return format!("Scientist #{} se dirige vers un point scientifique", self.id);
            }
        }
        
        self.status = RobotStatus::Idle;
        format!("Scientist #{} attend", self.id)
    }
    
    fn consume_energy(&mut self, amount: u32) -> bool {
        if self.energy >= amount {
            self.energy -= amount;
            true
        } else {
            self.energy = 0;
            self.status = RobotStatus::OutOfEnergy;
            false
        }
    }
    
    fn recharge_energy(&mut self, amount: u32) {
        self.energy = (self.energy + amount).min(self.config.max_energy);
    }
    
    fn get_memory(&self) -> RobotMemory {
        RobotMemory {
            explored_tiles: vec![],
            known_resources: self.visited_science_points.iter()
                .map(|pos| (*pos, "Science_Visited".to_string()))
                .collect(),
            target_position: self.target_science,
            home_position: self.base_position,
        }
    }
}