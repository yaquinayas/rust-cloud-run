use actix_web::{get, App, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    "Hello from Rust on Cloud Run!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    HttpServer::new(|| App::new().service(hello))
        .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
        .run()
        .await
}
