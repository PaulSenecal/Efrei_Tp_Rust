mod generator;
mod resources;

pub use generator::MapGenerator;
pub use resources::ResourceType;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub obstacle_threshold: f64,
    pub energy_density: f64,
    pub mineral_density: f64,
    pub science_density: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Obstacle,
    Energy(u32),
    Mineral(u32),
    SciencePoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub seed: u64,
}

impl Map {
    pub fn new(config: MapConfig) -> Self {
        let generator = MapGenerator::new(config.seed);
        let mut tiles = generator.generate_terrain(
            config.width, 
            config.height, 
            config.obstacle_threshold
        );
        
        // Ajouter les ressources
        generator.place_resources(
            &mut tiles,
            config.energy_density,
            config.mineral_density,
            config.science_density,
        );
        
        Map {
            width: config.width,
            height: config.height,
            tiles,
            seed: config.seed,
        }
    }
    
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y).and_then(|row| row.get(x))
    }
    
    pub fn consume_resource(&mut self, x: usize, y: usize) -> Option<ResourceType> {
        if let Some(row) = self.tiles.get_mut(y) {
            if let Some(tile) = row.get_mut(x) {
                match tile {
                    Tile::Energy(amount) => {
                        let resource_type = ResourceType::Energy(*amount);
                        *tile = Tile::Empty;
                        Some(resource_type)
                    }
                    Tile::Mineral(amount) => {
                        let resource_type = ResourceType::Mineral(*amount);
                        *tile = Tile::Empty;
                        Some(resource_type)
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}