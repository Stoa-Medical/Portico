use crate::SharedWorkflowMap;
use crate::proto::SignalResponse;
use crate::json_to_proto_struct;
use portico_database::{DatabaseItem, IdFields};
use portico_database::models::Step;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Instant;
use tonic::Status;

// Run signal handler for workflows
pub async fn handle_run_signal(
    workflow_map: SharedWorkflowMap,
    workflow_id: i32,
    run_data: Value,
    pool: PgPool,
) -> Result<SignalResponse, Status> {
    println!("[INFO] Processing run signal for workflow_id: {}", workflow_id);

    // Look up the workflow by ID
    let workflow_uuid = {
        let workflows = workflow_map.read().await;
        let workflow = workflows
            .values()
            .find(|w| w.identifiers.local_id == Some(workflow_id))
            .ok_or_else(|| {
                Status::not_found(format!("Workflow with ID {} not found", workflow_id))
            })?;
        workflow.identifiers.global_uuid.clone()
    };

    // Get the workflow with steps
    let workflow = {
        let workflows = workflow_map.read().await;
        workflows.get(&workflow_uuid)
            .cloned()
            .ok_or_else(|| Status::not_found(format!("Workflow {} not found", workflow_uuid)))?
    };
    let step_ids = workflow.step_ids.clone().unwrap_or_default();

    // Load steps by local IDs
    let mut steps: Vec<Step> = Vec::new();
    for local_id in &step_ids {
        let id = IdFields::with_values(Some(*local_id), String::new());
        if let Some(step) = Step::try_db_select_by_id(&pool, &id)
            .await
            .map_err(|e| Status::internal(format!("Failed to load step {}: {}", local_id, e)))?
        {
            steps.push(step);
        }
    }

    // Sort steps by step_order if available
    steps.sort_by_key(|s| s.step_order.unwrap_or(0));

    // Create a runtime session (validates workflow state, creates session record)
    let mut runtime_session = match workflow.run_with_steps(run_data.clone(), steps.clone()).await {
        Ok(session) => session,
        Err(e) => {
            eprintln!("[ERROR] Failed to create runtime session for workflow {}: {}", workflow_uuid, e);
            return Err(Status::internal(format!("Failed to create runtime session: {}", e)));
        }
    };

    // Execute steps sequentially using the step dispatch system.
    // Each step's output becomes the next step's input.
    let total_start = Instant::now();
    let mut current_input = run_data;
    let mut step_results: Vec<Option<Value>> = Vec::new();
    let mut step_execution_times: Vec<std::time::Duration> = Vec::new();
    let mut last_successful_result: Option<Value> = None;
    let mut last_step_idx: Option<i32> = None;
    let mut failed = false;

    for (idx, step) in steps.iter().enumerate() {
        let step_name = step.name.as_deref().unwrap_or("unnamed");
        println!(
            "[INFO] Executing step {} ({}) — type: {:?}",
            idx, step_name, step.step_type
        );

        let config = step.config.clone().unwrap_or(json!({}));
        let step_start = Instant::now();

        match crate::steps::execute_step(&step.step_type, &config, current_input.clone()).await {
            Ok(output) => {
                let elapsed = step_start.elapsed();
                println!(
                    "[INFO] Step {} ({}) completed in {:.2}ms",
                    idx,
                    step_name,
                    elapsed.as_secs_f64() * 1000.0
                );
                step_execution_times.push(elapsed);
                step_results.push(Some(output.clone()));
                last_successful_result = Some(output.clone());
                last_step_idx = Some(idx as i32);
                current_input = output;
            }
            Err(e) => {
                let elapsed = step_start.elapsed();
                eprintln!(
                    "[ERROR] Step {} ({}) failed after {:.2}ms: {}",
                    idx,
                    step_name,
                    elapsed.as_secs_f64() * 1000.0,
                    e
                );
                step_execution_times.push(elapsed);
                step_results.push(None);
                last_step_idx = Some(idx as i32);
                failed = true;
                break;
            }
        }
    }

    let total_elapsed = total_start.elapsed();

    // Update the runtime session with execution results
    runtime_session.last_step_idx = last_step_idx;
    runtime_session.last_successful_result = last_successful_result;
    runtime_session.step_execution_times = step_execution_times;
    runtime_session.total_execution_time = total_elapsed;
    runtime_session.step_results = step_results;
    runtime_session.status = if failed {
        portico_database::RunningStatus::Cancelled
    } else {
        portico_database::RunningStatus::Completed
    };

    // Save the runtime session to database
    if let Err(e) = runtime_session.try_db_create(&pool).await {
        eprintln!(
            "[ERROR] Failed to save runtime session to database: {}",
            e
        );
        return Err(Status::internal("Failed to save runtime session"));
    }

    if failed {
        return Err(Status::internal(format!(
            "Workflow execution failed at step {}",
            last_step_idx.unwrap_or(-1)
        )));
    }

    // Convert result to proto format
    let result_data = runtime_session
        .last_successful_result
        .as_ref()
        .map(|result| json_to_proto_struct(result));

    println!(
        "[INFO] Workflow {} completed in {:.2}ms ({} steps)",
        workflow_uuid,
        total_elapsed.as_secs_f64() * 1000.0,
        steps.len()
    );

    Ok(SignalResponse {
        success: true,
        message: "Workflow executed successfully".to_string(),
        runtime_session_uuid: runtime_session.identifiers.global_uuid,
        result_data,
    })
}
