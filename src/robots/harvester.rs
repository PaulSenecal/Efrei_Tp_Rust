use serde::{Serialize, Deserialize};
use crate::map::{Map, Tile, ResourceType};
use super::{Robot, RobotType, RobotStatus, Position, RobotConfig, RobotMemory, Direction};

/// Robot récolteur d'énergie autonome
#[derive(Debug, Serialize, Deserialize)]
pub struct EnergyHarvester {
    id: u32,
    position: Position,
    base_position: Position,
    status: RobotStatus,
    energy: u32,
    config: RobotConfig,
    cargo: Vec<ResourceType>,
    total_harvested: u32,
    target_resource: Option<Position>,
}

impl EnergyHarvester {
    pub fn new(id: u32, position: Position, base_position: Position) -> Self {
        let mut config = RobotConfig::default();
        config.action_cost = 8;
        config.max_cargo = 15;
        
        EnergyHarvester {
            id,
            position,
            base_position,
            status: RobotStatus::Idle,
            energy: config.max_energy,
            config,
            cargo: Vec::new(),
            total_harvested: 0,
            target_resource: None,
        }
    }
    
    fn find_nearest_energy(&self, known_resources: &[(Position, String)]) -> Option<Position> {
        known_resources.iter()
            .filter(|(_, res_type)| res_type.starts_with("Energy"))
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

impl Robot for EnergyHarvester {
    fn get_id(&self) -> u32 { self.id }
    fn get_type(&self) -> RobotType { RobotType::EnergyHarvester }
    fn get_position(&self) -> Position { self.position }
    fn get_status(&self) -> RobotStatus { self.status }
    fn get_energy(&self) -> u32 { self.energy }
    fn get_cargo(&self) -> Vec<ResourceType> { self.cargo.clone() }
    fn get_base_position(&self) -> Position { self.base_position }
    
    fn move_to(&mut self, new_position: Position) {
        self.position = new_position;
        let cargo_penalty = (self.cargo.len() as u32) / 3;
        self.consume_energy(self.config.move_cost + cargo_penalty);
    }
    
    fn think_and_act(&mut self, map: &mut Map, known_resources: &[(Position, String)]) -> String {
        // Si cargo plein ou énergie faible, retourner à la base
        if self.cargo.len() >= self.config.max_cargo || self.needs_recharge() {
            self.status = RobotStatus::ReturningToBase;
            self.target_resource = None;
            
            if self.position == self.base_position {
                // Décharger le cargo
                let cargo_value: u32 = self.cargo.iter().map(|r| r.value()).sum();
                self.cargo.clear();
                self.recharge_energy(20);
                return format!("EnergyBot #{} décharge {} unités d'énergie", self.id, cargo_value);
            }
            
            // Se déplacer vers la base
            if let Some(dir) = self.calculate_move_towards(self.base_position, map) {
                self.move_to(self.position.move_towards(dir));
                return format!("EnergyBot #{} retourne à la base", self.id);
            }
        }
        
        // Récolter si sur une ressource
        if let Some(resource) = map.consume_resource(self.position.x, self.position.y) {
            if matches!(resource, ResourceType::Energy(_)) {
                self.cargo.push(resource);
                self.total_harvested += resource.value();
                self.consume_energy(self.config.action_cost);
                self.target_resource = None;
                return format!("EnergyBot #{} récolte {} énergie", self.id, resource.value());
            }
        }
        
        // Chercher une nouvelle cible
        if self.target_resource.is_none() {
            self.target_resource = self.find_nearest_energy(known_resources);
            if self.target_resource.is_some() {
                self.status = RobotStatus::MovingToTarget;
            }
        }
        
        // Se déplacer vers la cible
        if let Some(target) = self.target_resource {
            if let Some(dir) = self.calculate_move_towards(target, map) {
                self.move_to(self.position.move_towards(dir));
                return format!("EnergyBot #{} cherche de l'énergie", self.id);
            }
        }
        
        self.status = RobotStatus::Idle;
        format!("EnergyBot #{} attend", self.id)
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
            known_resources: vec![],
            target_position: self.target_resource,
            home_position: self.base_position,
        }
    }
}

/// Robot récolteur de minerais autonome
#[derive(Debug, Serialize, Deserialize)]
pub struct MineralHarvester {
    id: u32,
    position: Position,
    base_position: Position,
    status: RobotStatus,
    energy: u32,
    config: RobotConfig,
    cargo: Vec<ResourceType>,
    total_harvested: u32,
    target_resource: Option<Position>,
}

impl MineralHarvester {
    pub fn new(id: u32, position: Position, base_position: Position) -> Self {
        let mut config = RobotConfig::default();
        config.action_cost = 10;
        config.max_cargo = 10;
        
        MineralHarvester {
            id,
            position,
            base_position,
            status: RobotStatus::Idle,
            energy: config.max_energy,
            config,
            cargo: Vec::new(),
            total_harvested: 0,
            target_resource: None,
        }
    }
    
    fn find_nearest_mineral(&self, known_resources: &[(Position, String)]) -> Option<Position> {
        known_resources.iter()
            .filter(|(_, res_type)| res_type.starts_with("Mineral"))
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

impl Robot for MineralHarvester {
    fn get_id(&self) -> u32 { self.id }
    fn get_type(&self) -> RobotType { RobotType::MineralHarvester }
    fn get_position(&self) -> Position { self.position }
    fn get_status(&self) -> RobotStatus { self.status }
    fn get_energy(&self) -> u32 { self.energy }
    fn get_cargo(&self) -> Vec<ResourceType> { self.cargo.clone() }
    fn get_base_position(&self) -> Position { self.base_position }
    
    fn move_to(&mut self, new_position: Position) {
        self.position = new_position;
        // Les minerais sont plus lourds, pénalité plus importante
        let cargo_penalty = (self.cargo.len() as u32) / 2;
        self.consume_energy(self.config.move_cost + cargo_penalty);
    }
    
    fn think_and_act(&mut self, map: &mut Map, known_resources: &[(Position, String)]) -> String {
        // Si cargo plein ou énergie faible, retourner à la base
        if self.cargo.len() >= self.config.max_cargo || self.needs_recharge() {
            self.status = RobotStatus::ReturningToBase;
            self.target_resource = None;
            
            if self.position == self.base_position {
                // Décharger le cargo
                let cargo_value: u32 = self.cargo.iter().map(|r| r.value()).sum();
                self.cargo.clear();
                self.recharge_energy(15);
                self.status = RobotStatus::Recharging;
                return format!("MineralBot #{} décharge {} minerais", self.id, cargo_value);
            }
            
            // Se déplacer vers la base
            if let Some(dir) = self.calculate_move_towards(self.base_position, map) {
                self.move_to(self.position.move_towards(dir));
                return format!("MineralBot #{} retourne à la base (cargo: {})", self.id, self.cargo.len());
            }
        }
        
        // Récolter si sur une ressource
        if let Some(resource) = map.consume_resource(self.position.x, self.position.y) {
            if matches!(resource, ResourceType::Mineral(_)) {
                self.cargo.push(resource);
                self.total_harvested += resource.value();
                self.consume_energy(self.config.action_cost);
                self.status = RobotStatus::Harvesting;
                self.target_resource = None;
                return format!("MineralBot #{} récolte {} minerais", self.id, resource.value());
            }
        }
        
        // Chercher une nouvelle cible
        if self.target_resource.is_none() || self.position == self.target_resource.unwrap() {
            self.target_resource = self.find_nearest_mineral(known_resources);
            if self.target_resource.is_some() {
                self.status = RobotStatus::MovingToTarget;
            } else {
                self.status = RobotStatus::Idle;
                return format!("MineralBot #{} - aucun minerai connu", self.id);
            }
        }
        
        // Se déplacer vers la cible
        if let Some(target) = self.target_resource {
            if let Some(dir) = self.calculate_move_towards(target, map) {
                self.move_to(self.position.move_towards(dir));
                return format!("MineralBot #{} cherche des minerais", self.id);
            } else {
                // Impossible d'atteindre la cible, l'abandonner
                self.target_resource = None;
                return format!("MineralBot #{} - cible inaccessible", self.id);
            }
        }
        
        self.status = RobotStatus::Idle;
        format!("MineralBot #{} attend", self.id)
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
        if self.energy >= self.config.low_energy_threshold {
            self.status = RobotStatus::Idle;
        }
    }
    
    fn get_memory(&self) -> RobotMemory {
        RobotMemory {
            explored_tiles: vec![],
            known_resources: vec![],
            target_position: self.target_resource,
            home_position: self.base_position,
        }
    }
}