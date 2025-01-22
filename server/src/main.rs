use actix_web::{App, HttpServer};
use portico_server::api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Preps python interpreter (only needs to run once, though repeat calls are negligible)
    pyo3::prepare_freethreaded_python();

    // Starts the API server
    HttpServer::new(|| {
        App::new()
            .configure(api::configure)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
