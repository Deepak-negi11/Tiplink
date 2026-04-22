use actix_web::{web::Data, App, HttpServer};
use actix_cors::Cors;

mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = db::conn::create_pool();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(Data::new(pool.clone()))
            .configure(routes::init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}