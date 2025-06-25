use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use super::Tile;

pub struct MapGenerator {
    noise: Perlin,
    rng: Pcg64,
}

impl MapGenerator {
    pub fn new(seed: u64) -> Self {
        let noise = Perlin::new(seed as u32);
        let rng = Pcg64::seed_from_u64(seed);
        
        MapGenerator { noise, rng }
    }
    
    pub fn generate_terrain(
        &self, 
        width: usize, 
        height: usize, 
        obstacle_threshold: f64
    ) -> Vec<Vec<Tile>> {
        let mut tiles = vec![vec![Tile::Empty; width]; height];
        
        // Utiliser Perlin noise pour générer les obstacles
        let scale = 0.1; // Échelle du bruit
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.noise.get([
                    x as f64 * scale,
                    y as f64 * scale,
                ]);
                
                // Normaliser le bruit entre 0 et 1
                let normalized = (noise_value + 1.0) / 2.0;
                
                if normalized > obstacle_threshold {
                    tiles[y][x] = Tile::Obstacle;
                }
            }
        }
        
        // Ajouter des bordures comme obstacles (optionnel)
        for x in 0..width {
            tiles[0][x] = Tile::Obstacle;
            tiles[height - 1][x] = Tile::Obstacle;
        }
        for y in 0..height {
            tiles[y][0] = Tile::Obstacle;
            tiles[y][width - 1] = Tile::Obstacle;
        }
        
        tiles
    }
    
    pub fn place_resources(
        &self,
        tiles: &mut Vec<Vec<Tile>>,
        energy_density: f64,
        mineral_density: f64,
        science_density: f64,
    ) {
        let mut rng = self.rng.clone();
        let height = tiles.len();
        let width = if height > 0 { tiles[0].len() } else { 0 };
        
        // Placer les ressources de manière aléatoire mais reproductible
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                if tiles[y][x] != Tile::Empty {
                    continue;
                }
                
                let rand_value: f64 = rng.gen();
                
                if rand_value < energy_density {
                    // Quantité d'énergie entre 10 et 50
                    let amount = rng.gen_range(10..=50);
                    tiles[y][x] = Tile::Energy(amount);
                } else if rand_value < energy_density + mineral_density {
                    // Quantité de minerais entre 5 et 25
                    let amount = rng.gen_range(5..=25);
                    tiles[y][x] = Tile::Mineral(amount);
                } else if rand_value < energy_density + mineral_density + science_density {
                    tiles[y][x] = Tile::SciencePoint;
                }
            }
        }
    }
}