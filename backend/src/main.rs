use actix-web::{web::Data,App , HttpServer}




#[actix-web::main]
async fn main()->std::io::Result<()>{
    let config = Config::from_env();
    let db_pool = db::create
    HttpServer::new(|| {
        App::new()
            .service
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}