
/// API for interacting with the server
use actix_web::{get, post, delete, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use actix_web::Error;
use anyhow::{Result};
use actix_web::dev::ServiceRequest;
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::bearer::BearerAuth;

async fn validator(req: ServiceRequest, credentials: BearerAuth) 
    -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let valid_token = std::env::var("API_TOKEN")
        .unwrap_or_else(|_| "valid-token".to_string());
    
    if credentials.token() == valid_token {
        Ok(req)
    } else {
        Err((Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "invalid token"
        )), req))
    }
}
/// Configure routes (scope and endpoints)
pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(validator);
    cfg.service(
        web::scope("/api")
            .wrap(auth)
            // Add routes here when you uncomment them
            // .service(register_pubkey)
            // .service(verify_pubkey)
            // .service(get_all_agents)
            // .service(create_agent)
            // .service(get_agent)
            // .service(update_agent)
            // .service(delete_agent)
    );
}
// // Structures for requests/responses
// #[derive(Deserialize)]
// struct RegisterKeyRequest {
//     secret: String,
//     public_key: String,
// }

// #[derive(Serialize)]
// struct AgentSettings {
//     // Add your agent settings fields here
//     agent_id: String,
//     settings: Value,
// }

// /// Given a correct secret, registers the public key to the device
// #[post("/crypto/register-pubkey")]
// async fn register_pubkey(req: web::Json<RegisterKeyRequest>) -> impl Responder {
//     // Validate against REGISTER_SECRET
//     let register_secret = std::env::var("REGISTER_SECRET")
//         .map_err(|_| HttpResponse::InternalServerError().finish())?;
    
//     if req.secret != register_secret {
//         return Ok(HttpResponse::Unauthorized().finish());
//     }
    
//     // TODO: Store public key
//     Ok(HttpResponse::Ok().finish())
// }

// /// Verifies that the given public key and signature is on the server + valid
// #[get("/crypto/verify-pubkey")]
// async fn verify_pubkey(
//     signature: web::Query<String>,
//     public_key: web::Query<String>,
// ) -> impl Responder {
//     // TODO: Implement signature verification
//     HttpResponse::Ok().finish()
// }


// /// Gets serialized IDs for all current agents
// #[get("/agents/all")]
// async fn get_all_agents() -> impl Responder {
//     // TODO: Implement fetching all agents
//     HttpResponse::Ok().json(Vec::<AgentSettings>::new())
// }

// /// Creates a new agent, returns the ID
// #[post("/agent")]
// async fn create_agent(settings: web::Json<serde_json::Value>) -> impl Responder {
//     let agent_id = Uuid::new_v4().to_string();
    
//     // TODO: Store agent settings
//     HttpResponse::Created().json(AgentSettings {
//         agent_id,
//         settings: settings.into_inner(),
//     })
// }

// /// Gets serialized data for a specific agent
// #[get("/agent/{agent_id}")]
// async fn get_agent(path: web::Path<String>) -> impl Responder {
//     let agent_id = path.into_inner();
//     // TODO: Implement fetching specific agent
//     HttpResponse::Ok().json(AgentSettings {
//         agent_id,
//         settings: serde_json::Value::Null,
//     })
// }

// /// Updates agent data for a specific agent
// #[put("/agent/{agent_id}")]
// async fn update_agent(
//     path: web::Path<String>,
//     settings: web::Json<serde_json::Value>,
// ) -> impl Responder {
//     let agent_id = path.into_inner();
//     // TODO: Implement updating agent settings
//     HttpResponse::Ok().finish()
// }

// /// Deletes the agent
// #[delete("/agent/{agent_id}")]
// async fn delete_agent(path: web::Path<String>) -> impl Responder {
//     let agent_id = path.into_inner();
//     // TODO: Implement agent deletion
//     HttpResponse::Ok().finish()
// }
