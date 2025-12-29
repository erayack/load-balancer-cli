use crate::error::{SimError, SimResult};
use crate::models::{SimConfig, TieBreakConfig};
use crate::state::SimulationResult;

pub fn print_summary(result: &SimulationResult) {
    println!("Summary:");
    for summary in &result.totals {
        println!(
            "{}: {} requests (avg response: {}ms)",
            summary.name, summary.requests, summary.avg_response_ms
        );
    }
}

pub fn print_full(config: &SimConfig, result: &SimulationResult) -> SimResult<()> {
    match result.tie_break {
        TieBreakConfig::Stable => println!("Tie-break: stable"),
        TieBreakConfig::Seeded => {
            let seed = result.seed.unwrap_or_default();
            println!("Tie-break: seeded({})", seed);
        }
    }

    for assignment in &result.assignments {
        let server_name = config
            .servers
            .get(assignment.server_id)
            .map(|server| server.name.as_str())
            .ok_or_else(|| {
                SimError::InvalidServerEntry(format!(
                    "missing server for id {}",
                    assignment.server_id
                ))
            })?;
        if let Some(score) = assignment.score {
            println!(
                "Request {} -> {} (score: {}ms)",
                assignment.request_id, server_name, score
            );
        } else {
            println!("Request {} -> {}", assignment.request_id, server_name);
        }
    }

    print_summary(result);

    Ok(())
}
