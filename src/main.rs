use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use actix_files::Files;
use serde::{Serialize, Deserialize};
use log::info;

mod map;

use map::{Map, MapConfig};

#[derive(Serialize, Deserialize)]
struct MapRequest {
    seed: Option<u64>,
    width: Option<usize>,
    height: Option<usize>,
}

async fn get_map(query: web::Query<MapRequest>) -> HttpResponse {
    let config = MapConfig {
        width: query.width.unwrap_or(50),
        height: query.height.unwrap_or(50),
        seed: query.seed.unwrap_or(42),
        obstacle_threshold: 0.3,
        energy_density: 0.05,
        mineral_density: 0.03,
        science_density: 0.02,
    };
    
    let map = Map::new(config);
    HttpResponse::Ok().json(&map)
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    info!("Démarrage du serveur sur http://localhost:8080");
    
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .route("/api/map", web::get().to(get_map))
            .service(Files::new("/static", "./src/static"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}