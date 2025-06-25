use serde::{Serialize, Deserialize};

/// Position sur la carte
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    /// Crée une nouvelle position
    pub fn new(x: usize, y: usize) -> Self {
        Position { x, y }
    }
    
    /// Calcule la nouvelle position selon une direction
    pub fn move_towards(&self, direction: Direction) -> Position {
        match direction {
            Direction::North => Position::new(self.x, self.y.saturating_sub(1)),
            Direction::South => Position::new(self.x, self.y + 1),
            Direction::East => Position::new(self.x + 1, self.y),
            Direction::West => Position::new(self.x.saturating_sub(1), self.y),
        }
    }
    
    /// Calcule la distance de Manhattan entre deux positions
    pub fn manhattan_distance(&self, other: &Position) -> usize {
        ((self.x as i32 - other.x as i32).abs() + 
         (self.y as i32 - other.y as i32).abs()) as usize
    }
    
    /// Obtient les positions voisines (4 directions)
    pub fn get_neighbors(&self) -> Vec<Position> {
        vec![
            self.move_towards(Direction::North),
            self.move_towards(Direction::South),
            self.move_towards(Direction::East),
            self.move_towards(Direction::West),
        ]
    }
}

/// Directions de déplacement possibles
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Obtient toutes les directions
    pub fn all() -> Vec<Direction> {
        vec![
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ]
    }
    
    /// Obtient la direction opposée
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
    
    /// Convertit une chaîne en direction
    pub fn from_str(s: &str) -> Option<Direction> {
        match s.to_lowercase().as_str() {
            "north" | "n" => Some(Direction::North),
            "south" | "s" => Some(Direction::South),
            "east" | "e" => Some(Direction::East),
            "west" | "w" => Some(Direction::West),
            _ => None,
        }
    }
}