/// Test the simple workflow planner service
#[cfg(test)]
mod tests {
    use super::simple_workflow_planner::SimpleWorkflowPlannerService;
    use serde_json::json;

    #[tokio::test]
    async fn test_simple_workflow_planner() {
        let planner = SimpleWorkflowPlannerService::new();

        let result = planner.plan_workflow(
            1,
            "Analyze customer data from CSV file".to_string(),
            Some(json!({"file": "customers.csv"})),
            None,
            false
        ).await;

        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.success);
        assert!(plan.estimated_steps > 0);
        assert!(plan.workflow_spec.is_some());

        println!("Plan message: {}", plan.message);
        println!("Workflow spec: {}", serde_json::to_string_pretty(&plan.workflow_spec).unwrap());
    }

    #[tokio::test]
    async fn test_webscraping_workflow() {
        let planner = SimpleWorkflowPlannerService::new();

        let result = planner.plan_workflow(
            2,
            "Scrape website data from news site".to_string(),
            Some(json!({"url": "https://example.com"})),
            None,
            true // ephemeral
        ).await;

        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.success);
        assert!(plan.requires_approval); // Web scraping should require approval
        assert_eq!(plan.estimated_steps, 3); // webscrape + process + summary

        println!("Web scraping plan: {}", plan.message);
    }

    #[tokio::test]
    async fn test_system_workflow() {
        let planner = SimpleWorkflowPlannerService::new();

        let result = planner.plan_workflow(
            3,
            "Perform system administration task".to_string(),
            None,
            None,
            false
        ).await;

        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.success);
        assert!(plan.requires_approval); // System tasks should require approval

        println!("System task plan: {}", plan.message);
    }
}
