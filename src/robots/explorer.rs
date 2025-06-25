use serde::{Serialize, Deserialize};
use crate::map::{Map, Tile, ResourceType};
use super::{Robot, RobotType, RobotStatus, Position, RobotConfig, RobotMemory, Direction};
use std::collections::{HashSet, VecDeque};


/// Robot exploreur autonome - découvre la carte et marque les ressources
#[derive(Debug, Serialize, Deserialize)]
pub struct Explorer {
    id: u32,
    position: Position,
    base_position: Position,
    status: RobotStatus,
    energy: u32,
    config: RobotConfig,
    explored_tiles: HashSet<(usize, usize)>,
    discovered_resources: Vec<(Position, String)>,
    exploration_queue: VecDeque<Position>,
    current_target: Option<Position>,
}

impl Explorer {
    /// Crée un nouveau robot exploreur autonome
    pub fn new(id: u32, position: Position, base_position: Position) -> Self {
        let mut config = RobotConfig::default();
        config.action_cost = 2; // Explorer coûte moins cher en énergie
        config.vision_range = 4; // Vision plus large
        
        Explorer {
            id,
            position,
            base_position,
            status: RobotStatus::Idle,
            energy: config.max_energy,
            config,
            explored_tiles: HashSet::new(),
            discovered_resources: Vec::new(),
            exploration_queue: VecDeque::new(),
            current_target: None,
        }
    }
    
    /// Trouve la prochaine zone inexplorée
    fn find_unexplored_area(&self, map: &Map) -> Option<Position> {
    let max_distance: i32 = 20; // Spécifier le type ici
    
    for distance in 1..=max_distance {
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                    // Vérifier seulement le périmètre de la distance actuelle
                    if dx.abs() != distance && dy.abs() != distance {
                        continue;
                    }
                    
                    let x = (self.position.x as i32 + dx).max(0) as usize;
                    let y = (self.position.y as i32 + dy).max(0) as usize;
                    
                    if x < map.width && y < map.height {
                        let pos = Position::new(x, y);
                        if !self.explored_tiles.contains(&(x, y)) {
                            // Vérifier qu'on peut atteindre cette position
                            if let Some(Tile::Obstacle) = map.get_tile(x, y) {
                                continue;
                            }
                            return Some(pos);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Explore les tuiles visibles depuis la position actuelle
    fn scan_area(&mut self, map: &Map) -> Vec<String> {
        let mut discoveries = Vec::new();
        let vision_range = self.config.vision_range as i32;
        
        for dx in -vision_range..=vision_range {
            for dy in -vision_range..=vision_range {
                let x = (self.position.x as i32 + dx).max(0) as usize;
                let y = (self.position.y as i32 + dy).max(0) as usize;
                
                if x >= map.width || y >= map.height {
                    continue;
                }
                
                let tile_key = (x, y);
                if self.explored_tiles.insert(tile_key) {
                    // Nouvelle tuile explorée
                    if let Some(tile) = map.get_tile(x, y) {
                        let pos = Position::new(x, y);
                        match tile {
                            Tile::Energy(amount) => {
                                let discovery = format!("Énergie[{}] en ({},{})", amount, x, y);
                                discoveries.push(discovery.clone());
                                self.discovered_resources.push((pos, format!("Energy:{}", amount)));
                            },
                            Tile::Mineral(amount) => {
                                let discovery = format!("Minerai[{}] en ({},{})", amount, x, y);
                                discoveries.push(discovery.clone());
                                self.discovered_resources.push((pos, format!("Mineral:{}", amount)));
                            },
                            Tile::SciencePoint => {
                                let discovery = format!("Science en ({},{})", x, y);
                                discoveries.push(discovery.clone());
                                self.discovered_resources.push((pos, "Science".to_string()));
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        discoveries
    }
    
    /// Calcule le meilleur mouvement vers la cible
    fn calculate_move_towards(&self, target: Position, map: &Map) -> Option<Direction> {
        let dx = target.x as i32 - self.position.x as i32;
        let dy = target.y as i32 - self.position.y as i32;
        
        // Prioriser le mouvement sur l'axe le plus long
        let try_horizontal_first = dx.abs() >= dy.abs();
        
        let mut directions = Vec::new();
        
        if try_horizontal_first {
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
        
        // Essayer chaque direction dans l'ordre
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

impl Robot for Explorer {
    fn get_id(&self) -> u32 {
        self.id
    }
    
    fn get_type(&self) -> RobotType {
        RobotType::Explorer
    }
    
    fn get_position(&self) -> Position {
        self.position
    }
    
    fn get_status(&self) -> RobotStatus {
        self.status
    }
    
    fn get_energy(&self) -> u32 {
        self.energy
    }
    
    fn get_cargo(&self) -> Vec<ResourceType> {
        Vec::new() // Les exploreurs ne transportent pas de ressources
    }
    
    fn get_base_position(&self) -> Position {
        self.base_position
    }
    
    fn move_to(&mut self, new_position: Position) {
        self.position = new_position;
        self.consume_energy(self.config.move_cost);
        self.explored_tiles.insert((new_position.x, new_position.y));
    }
    
    fn think_and_act(&mut self, map: &mut Map, _known_resources: &[(Position, String)]) -> String {
        // Vérifier l'énergie
        if self.needs_recharge() && self.position != self.base_position {
            self.status = RobotStatus::ReturningToBase;
            self.current_target = Some(self.base_position);
        }
        
        // Si à la base et besoin de recharge
        if self.position == self.base_position && self.energy < self.config.max_energy {
            self.recharge_energy(10);
            self.status = RobotStatus::Recharging;
            return format!("Explorer #{} se recharge à la base", self.id);
        }
        
        // Explorer la zone actuelle
        let discoveries = self.scan_area(map);
        if !discoveries.is_empty() {
            self.consume_energy(self.config.action_cost);
            return format!("Explorer #{} a découvert: {}", self.id, discoveries.join(", "));
        }
        
        // Choisir une nouvelle cible si nécessaire
        if self.current_target.is_none() || self.position == self.current_target.unwrap() {
            if let Some(unexplored) = self.find_unexplored_area(map) {
                self.current_target = Some(unexplored);
                self.status = RobotStatus::MovingToTarget;
            } else {
                // Tout est exploré, retourner à la base
                self.current_target = Some(self.base_position);
                self.status = RobotStatus::ReturningToBase;
            }
        }
        
        // Se déplacer vers la cible
        if let Some(target) = self.current_target {
            if let Some(direction) = self.calculate_move_towards(target, map) {
                let new_pos = self.position.move_towards(direction);
                self.move_to(new_pos);
                return format!("Explorer #{} se déplace vers {:?}", self.id, direction);
            }
        }
        
        self.status = RobotStatus::Idle;
        format!("Explorer #{} attend", self.id)
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
            explored_tiles: self.explored_tiles.iter().cloned().collect(),
            known_resources: self.discovered_resources.clone(),
            target_position: self.current_target,
            home_position: self.base_position,
        }
    }
}