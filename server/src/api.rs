use actix_web::{
    get, post, put, delete,
    web::{self, Data, Path, Query, ServiceConfig},
    HttpResponse, Responder,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::dev::ServiceRequest;
use actix_web::Error;
use anyhow::{Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// use crate::models::agents::Agent;
// /// A simplified "create/update" model that excludes the ID,
// /// because it will be auto-generated.
// #[derive(Serialize, Deserialize)]
// pub struct AgentForm {
//     pub name: String,
//     pub description: String,
// }

// /// Application-wide state, storing Agents in-memory.
// /// In production, you might connect to a real database or other storage.
// #[derive(Clone)]
// pub struct AppState {
//     pub agents: Arc<Mutex<HashMap<Uuid, Agent>>>,
// }

// /// Simple bearer token validator
// async fn validator(req: ServiceRequest, credentials: BearerAuth)
//     -> Result<ServiceRequest, (Error, ServiceRequest)>
// {
//     let valid_token = std::env::var("API_TOKEN").unwrap_or_else(|_| "valid-token".to_string());
//     if credentials.token() == valid_token {
//         Ok(req)
//     } else {
//         Err((Error::from(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             "Invalid token",
//         )), req))
//     }
// }

// #[post("/agents")]
// /// POST /api/agents
// /// Creates a new Agent and returns its full details (including generated UUID).
// async fn create_agent(
//     state: Data<AppState>,
//     form: web::Json<AgentForm>,
// ) -> impl Responder {
//     let mut agents_map = state.agents.lock().unwrap();
//     let new_id = Uuid::new_v4();

//     let agent = Agent {
//         id: new_id,
//         name: form.name.clone(),
//         steps: form.steps.clone(),
//     };

//     agents_map.insert(new_id, agent.clone());

//     HttpResponse::Created().json(agent)
// }

// #[get("/agents/all")]
// /// GET /api/agents/all
// /// Returns a list of all configured agents with full details.
// async fn get_all_agents(state: Data<AppState>) -> impl Responder {
//     let agents_map = state.agents.lock().expect("retrieve all agents");
//     let agents: Vec<_> = agents_map
//         .values()
//         .collect();

//     HttpResponse::Ok().json(agents)
// }

// #[get("/agents/{agent_id}")]
// /// GET /api/agents/{agent_id}
// /// Reads the full agent record by its UUID.
// async fn read_agent(
//     state: Data<AppState>,
//     path: Path<Uuid>,
// ) -> impl Responder {
//     let agent_id = path.into_inner();
//     let agents_map = state.agents.lock().unwrap();
//     match agents_map.get(&agent_id) {
//         Some(agent) => HttpResponse::Ok().json(agent),
//         None => HttpResponse::NotFound().body("Agent not found"),
//     }
// }

// #[put("/agents/{agent_id}")]
// /// PUT /api/agents/{agent_id}
// /// Updates an existing agent. In a real system, you can be selective about which fields are updated.
// async fn update_agent(
//     state: Data<AppState>,
//     path: Path<Uuid>,
//     form: web::Json<AgentForm>,
// ) -> impl Responder {
//     let agent_id = path.into_inner();
//     let mut agents_map = state.agents.lock().unwrap();
//     if let Some(agent) = agents_map.get_mut(&agent_id) {
//         agent.name = form.name.clone();
//         agent.steps = form.steps.clone();
//         // Update other fields as needed
//         HttpResponse::Ok().json(agent.clone())
//     } else {
//         HttpResponse::NotFound().body("Agent not found")
//     }
// }

// #[delete("/agents/{agent_id}")]
// /// DELETE /api/agents/{agent_id}
// /// Removes the specified agent from the system.
// async fn delete_agent(
//     state: Data<AppState>,
//     path: Path<Uuid>,
// ) -> impl Responder {
//     let agent_id = path.into_inner();
//     let mut agents_map = state.agents.lock().unwrap();
//     match agents_map.remove(&agent_id) {
//         Some(_) => HttpResponse::Ok().body("Agent deleted"),
//         None => HttpResponse::NotFound().body("Agent not found"),
//     }
// }


// #[derive(Serialize, Deserialize)]
// #[serde(rename_all = "lowercase")]
// pub enum StepActionAsString {
//     Python,
//     Prompt,
// }

// #[derive(Serialize, Deserialize)]
// pub struct StepForm {
//     pub name: String,
//     pub instruction: String,
//     pub instruction_type: StepActionAsString
// }

// #[get("/steps/all")]
// async fn all_steps(...) -> impl Responder {
//     // TODO
// }

// #[get("/steps/{step_id}")]
// async fn read_step(...) -> impl Responder {
//     // TODO
// }

// #[post("/steps")]
// async fn create_step(...) -> impl Responder {
//     // TODO
// }

// #[put("/steps/{step_id}")
// async fn update_step(...) -> impl Responder {
//     // TODO
// }

// #[delete("/steps/{step_id}")]
// async fn delete_step(...) -> impl Responder {
//     // TODO
// }

// #[post("/steps/{step_id}/assign")]
// async fn assign_step(...) -> impl Responder {
//     // TODO: tries to find the step id, and then adds it to the corresponding agent (if applicable)
//     // By default, appends to end of the list
// }

// #[post("/steps/{step_id}/unassign")]
// async fn unassign_step(...) -> impl Repsonder {
//     // TODO: tries to find the Step, then removes it from corresponding agent (if applicable)
//     // removes wherever the step is located
// }

/// Configure routes (scope and endpoints)
pub fn configure(cfg: &mut ServiceConfig) {
    // let auth = HttpAuthentication::bearer(validator);

    cfg.service(
        web::scope("/api")
            // .wrap(auth)
            // .service(create_agent)
            // .service(get_all_agents)
            // .service(read_agent)
            // .service(update_agent)
            // .service(delete_agent)
    );
}
