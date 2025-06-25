use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use actix_files::Files;
use serde::{Serialize, Deserialize};
use log::info;
use std::sync::Mutex;

mod map;
mod robots;

use map::{Map, MapConfig};
use robots::{RobotManager, RobotState, SimulationStats};

/// État global de la simulation
struct SimulationState {
    map: Map,
    robots: RobotManager,
}

/// Requête pour générer une carte
#[derive(Serialize, Deserialize)]
struct MapRequest {
    seed: Option<u64>,
    width: Option<usize>,
    height: Option<usize>,
}

/// Requête pour créer un robot
#[derive(Serialize, Deserialize)]
struct CreateRobotRequest {
    robot_type: String,
    x: usize,
    y: usize,
}

/// Configuration de simulation
#[derive(Serialize, Deserialize)]
struct SimulationConfig {
    ticks: Option<u32>,
}

/// Réponse API standard
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    fn error(error: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Génère une nouvelle carte avec une base au centre
async fn generate_map(
    query: web::Query<MapRequest>,
    state: web::Data<Mutex<SimulationState>>
) -> HttpResponse {
    let config = MapConfig {
        width: query.width.unwrap_or(50),
        height: query.height.unwrap_or(50),
        seed: query.seed.unwrap_or(42),
        obstacle_threshold: 0.3,
        energy_density: 0.05,
        mineral_density: 0.03,
        science_density: 0.02,
    };
    
    let mut new_map = Map::new(config);
    
    // Placer la base au centre de la carte
    let base_x = new_map.width / 2;
    let base_y = new_map.height / 2;
    
    // S'ASSURER que la base n'est pas sur un obstacle
    // Nettoyer une zone 3x3 autour de la base
    for dx in -1..=1 {
        for dy in -1..=1 {
            let x = (base_x as i32 + dx).max(0).min(new_map.width as i32 - 1) as usize;
            let y = (base_y as i32 + dy).max(0).min(new_map.height as i32 - 1) as usize;
            
            if let Some(row) = new_map.tiles.get_mut(y) {
                if let Some(tile) = row.get_mut(x) {
                    if *tile == map::Tile::Obstacle {
                        *tile = map::Tile::Empty;
                    }
                }
            }
        }
    }
    
    let mut sim_state = state.lock().unwrap();
    sim_state.map = new_map.clone();
    sim_state.robots = RobotManager::new(base_x, base_y);
    
    HttpResponse::Ok().json(ApiResponse::success(&new_map))
}

/// Crée un nouveau robot
/// Crée un nouveau robot
async fn create_robot(
    body: web::Json<CreateRobotRequest>,
    state: web::Data<Mutex<SimulationState>>
) -> HttpResponse {
    let mut sim_state = state.lock().unwrap();
    
    // Fonction helper pour trouver une position libre près de la base
    let find_spawn_position = |map: &Map, base_x: usize, base_y: usize| -> Option<(usize, usize)> {
        // Essayer d'abord la position exacte de la base
        if let Some(tile) = map.get_tile(base_x, base_y) {
            if tile != &map::Tile::Obstacle {
                return Some((base_x, base_y));
            }
        }
        
        // Chercher en spirale autour de la base
        for radius in 1..=5 {
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    // Vérifier seulement le périmètre du rayon actuel
                    if dx.abs() != radius && dy.abs() != radius {
                        continue;
                    }
                    
                    let x = (base_x as i32 + dx).max(0) as usize;
                    let y = (base_y as i32 + dy).max(0) as usize;
                    
                    if x < map.width && y < map.height {
                        if let Some(tile) = map.get_tile(x, y) {
                            if tile != &map::Tile::Obstacle {
                                return Some((x, y));
                            }
                        }
                    }
                }
            }
        }
        
        None // Aucune position libre trouvée
    };
    
    // Trouver une position libre près de la base
    let base_pos = sim_state.robots.get_base_position(); // Récupérer la position de la base
    let (spawn_x, spawn_y) = match find_spawn_position(&sim_state.map, base_pos.x, base_pos.y) {
        Some(pos) => pos,
        None => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Aucune position libre trouvée près de la base".to_string()
            ));
        }
    };
    
    // Créer le robot à la position trouvée
    let robot_id = match body.robot_type.to_lowercase().as_str() {
        "explorer" => sim_state.robots.add_explorer(spawn_x, spawn_y),
        "energy_harvester" => sim_state.robots.add_energy_harvester(spawn_x, spawn_y),
        "mineral_harvester" => sim_state.robots.add_mineral_harvester(spawn_x, spawn_y),
        "scientist" => sim_state.robots.add_scientist(spawn_x, spawn_y),
        _ => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Type de robot invalide. Types disponibles: explorer, energy_harvester, mineral_harvester, scientist".to_string()
            ));
        }
    };
    
    if let Some(robot_state) = sim_state.robots.get_robot_state(robot_id) {
        HttpResponse::Ok().json(ApiResponse::success(robot_state))
    } else {
        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            "Erreur lors de la création du robot".to_string()
        ))
    }
}

/// Fait avancer la simulation d'un ou plusieurs ticks
async fn simulate_ticks(
    body: web::Json<SimulationConfig>,
    state: web::Data<Mutex<SimulationState>>
) -> HttpResponse {
    let mut sim_state = state.lock().unwrap();
    let ticks = body.ticks.unwrap_or(1);
    
    let mut all_actions = Vec::new();
    
    for _ in 0..ticks {
       // let actions = sim_state.robots.simulate_tick(&mut sim_state.map);4
        let actions = {
        let SimulationState { ref mut map, ref mut robots, .. } = &mut *sim_state;
        robots.simulate_tick(map)
        };
        all_actions.extend(actions);
    }
    
    #[derive(Serialize)]
    struct SimulationResult {
        actions: Vec<String>,
        robots: Vec<RobotState>,
        stats: SimulationStats,
    }
    
    let result = SimulationResult {
        actions: all_actions,
        robots: sim_state.robots.get_all_robots_state(),
        stats: sim_state.robots.get_statistics(),
    };
    
    HttpResponse::Ok().json(ApiResponse::success(result))
}

/// Obtient l'état de tous les robots
async fn get_all_robots(state: web::Data<Mutex<SimulationState>>) -> HttpResponse {
    let sim_state = state.lock().unwrap();
    let robots = sim_state.robots.get_all_robots_state();
    HttpResponse::Ok().json(ApiResponse::success(robots))
}

/// Obtient l'état d'un robot spécifique
async fn get_robot(
    path: web::Path<u32>,
    state: web::Data<Mutex<SimulationState>>
) -> HttpResponse {
    let robot_id = path.into_inner();
    let sim_state = state.lock().unwrap();
    
    match sim_state.robots.get_robot_state(robot_id) {
        Some(robot_state) => HttpResponse::Ok().json(ApiResponse::success(robot_state)),
        None => HttpResponse::NotFound().json(ApiResponse::<()>::error(
            format!("Robot {} non trouvé", robot_id)
        )),
    }
}

/// Obtient les statistiques de la simulation
async fn get_statistics(state: web::Data<Mutex<SimulationState>>) -> HttpResponse {
    let sim_state = state.lock().unwrap();
    let stats = sim_state.robots.get_statistics();
    HttpResponse::Ok().json(ApiResponse::success(stats))
}

/// Page d'accueil
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("static/index.html"))
}
async fn get_map(state: web::Data<Mutex<SimulationState>>) -> HttpResponse {
    let sim_state = state.lock().unwrap();
    HttpResponse::Ok().json(ApiResponse::success(&sim_state.map))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Créer l'état initial de la simulation
    let initial_config = MapConfig {
        width: 50,
        height: 50,
        seed: 42,
        obstacle_threshold: 0.3,
        energy_density: 0.05,
        mineral_density: 0.03,
        science_density: 0.02,
    };
    
    let initial_map = Map::new(initial_config);
    let base_x = initial_map.width / 2;
    let base_y = initial_map.height / 2;
    
    let simulation_state = web::Data::new(Mutex::new(SimulationState {
        map: initial_map,
        robots: RobotManager::new(base_x, base_y),
    }));
    
    info!("🚀 Démarrage du serveur sur http://localhost:8080");
    info!("🤖 Simulation de robots autonomes activée");
    
    HttpServer::new(move || {
        App::new()
            .app_data(simulation_state.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            // Endpoints de carte
            .route("/api/map", web::get().to(get_map))
            .route("/api/map/generate", web::post().to(generate_map))
            // Endpoints de robots
            .route("/api/robots", web::get().to(get_all_robots))
            .route("/api/robots", web::post().to(create_robot))
            .route("/api/robots/{id}", web::get().to(get_robot))
            // Endpoints de simulation
            .route("/api/simulation/tick", web::post().to(simulate_ticks))
            .route("/api/simulation/stats", web::get().to(get_statistics))
            // Fichiers statiques
            .service(Files::new("/static", "./src/static"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}