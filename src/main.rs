use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

struct AppState {
    db_pool: PgPool,
}

#[get("/")]
async fn hello() -> impl Responder {
    "Hello from Rust on Cloud Run!"
}

#[get("/db-test")]
async fn db_test(state: actix_web::web::Data<AppState>) -> impl Responder {
    match sqlx::query("SELECT 1 as test")
        .fetch_one(&state.db_pool)
        .await 
    {
        Ok(row) => {
            let result: i32 = row.get("test");
            HttpResponse::Ok().body(format!("✅ Conexión exitosa a PostgreSQL! Resultado: {}", result))
        }
        Err(e) => {
            eprintln!("❌ Error de conexión: {:?}", e);
            HttpResponse::InternalServerError().body(format!("❌ Error conectando a BD: {}", e))
        }
    }
}

#[get("/db-info")]
async fn db_info() -> impl Responder {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "No configurada".to_string());
    let masked = if db_url.contains('@') {
        let parts: Vec<&str> = db_url.split('@').collect();
        if parts.len() == 2 {
            let before = parts[0];
            let after = parts[1];
            if before.contains(':') {
                let auth_parts: Vec<&str> = before.split(':').collect();
                if auth_parts.len() == 2 {
                    format!("{}:***@{}", auth_parts[0], after)
                } else {
                    db_url
                }
            } else {
                db_url
            }
        } else {
            db_url
        }
    } else {
        db_url
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "database_url_configured": env::var("DATABASE_URL").is_ok(),
        "database_url_masked": masked,
        "port": env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
        "cloud_run": env::var("K_SERVICE").is_ok(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cargar variables de entorno
    dotenv::dotenv().ok();
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let database_url = env::var("DATABASE_URL")
        .expect("❌ DATABASE_URL no está configurada!");
    
    println!("🚀 Iniciando aplicación...");
    println!("📡 Puerto: {}", port);
    println!("🛢️  Conectando a PostgreSQL...");
    
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("❌ No se pudo conectar a PostgreSQL!");
    
    println!("✅ Conexión a PostgreSQL establecida!");
    
    let pool_clone = pool.clone();
    
    println!("🌐 Servidor iniciado en http://0.0.0.0:{}", port);
    
    // 🔴 IMPORTANTE: Aquí es donde se registran TODOS los endpoints
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(AppState {
                db_pool: pool_clone.clone(),
            }))
            .service(hello)      // <- GET /
            .service(db_test)    // <- GET /db-test
            .service(db_info)    // <- GET /db-info
    })
    .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
    .run()
    .await
}