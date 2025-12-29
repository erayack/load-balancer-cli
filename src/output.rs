use crate::models::SimulationResult;

pub fn print_summary(result: &SimulationResult) {
    println!("Summary:");
    for summary in &result.totals {
        println!(
            "{}: {} requests (avg response: {}ms)",
            summary.name, summary.requests, summary.avg_response_ms
        );
    }
}

pub fn print_full(result: &SimulationResult) {
    println!("Tie-break: {}", result.tie_break);

    for assignment in &result.assignments {
        if let Some(score) = assignment.score {
            println!(
                "Request {} -> {} (score: {}ms)",
                assignment.request_id, assignment.server_name, score
            );
        } else {
            println!(
                "Request {} -> {}",
                assignment.request_id, assignment.server_name
            );
        }
    }

    print_summary(result);
}
