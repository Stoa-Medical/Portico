use actix_web::{App, HttpServer};
use pyo3::prelude::*;

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    /// Preps python interpreter (only needs to run once, though repeat calls are negligible)
    pyo3::prepare_freethreaded_python();

    /// Starts the API server
    HttpServer::new(|| {
        App::new()
            .configure(api::configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     let database_url = "postgres://username:password@localhost:5432/dbname";
//     let db = Database::new(database_url).await?;
    
//     // Create user
//     let user_id = db.create_user("John Doe", "john@example.com").await?;
    
//     // Get user
//     if let Some(user) = db.get_user(user_id).await? {
//         println!("Found user: {:?}", user);
//     }
    
//     Ok(())
// }