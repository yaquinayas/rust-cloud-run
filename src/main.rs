use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

// Estado compartido para el pool de conexiones
struct AppState {
    db_pool: PgPool,
}

#[get("/")]
async fn hello() -> impl Responder {
    "Hello from Rust on Cloud Run!"
}

// Endpoint para probar la conexión a la BD
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

// Endpoint para ver la configuración (sin mostrar contraseña)
#[get("/db-info")]
async fn db_info() -> impl Responder {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "No configurada".to_string());
    // Ocultamos la contraseña por seguridad
    let masked = if db_url.contains('@') {
        // Reemplaza la contraseña con ***
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
    // Cargar variables de entorno desde .env (solo para desarrollo local)
    dotenv::dotenv().ok();
    
    // Configurar logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let database_url = env::var("DATABASE_URL")
        .expect("❌ DATABASE_URL no está configurada! Revisa tus variables de entorno");
    
    println!("🚀 Iniciando aplicación...");
    println!("📡 Puerto: {}", port);
    println!("🛢️  Conectando a PostgreSQL...");
    
    // Crear pool de conexiones
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("❌ No se pudo conectar a PostgreSQL! Verifica DATABASE_URL");
    
    println!("✅ Conexión a PostgreSQL establecida!");
    
    // Verificar conexión con un ping
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => println!("✅ Ping a BD exitoso"),
        Err(e) => eprintln!("⚠️  Ping a BD falló: {}", e),
    }
    
    let pool_clone = pool.clone();
    
    println!("🌐 Servidor iniciado en http://0.0.0.0:{}", port);
    println!("📋 Endpoints disponibles:");
    println!("   - GET /        → Mensaje de bienvenida");
    println!("   - GET /db-test → Prueba de conexión a BD");
    println!("   - GET /db-info → Información de configuración");
    
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(AppState {
                db_pool: pool_clone.clone(),
            }))
            .service(hello)
            .service(db_test)
            .service(db_info)
    })
    .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
    .run()
    .await
}