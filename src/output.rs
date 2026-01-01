use crate::state::{Assignment, Phase1Metrics, RunMetadata, ServerSummary, SimulationResult};
use serde::Serialize;

pub trait Formatter {
    fn write(&self, result: &SimulationResult) -> String;
}

pub struct HumanFormatter;

impl Formatter for HumanFormatter {
    fn write(&self, result: &SimulationResult) -> String {
        let mut output = String::new();
        write_metadata(&mut output, result);
        output.push_str("Assignments:\n");
        for assignment in &result.assignments {
            write_assignment_with_totals(&mut output, assignment, &result.totals);
        }
        write_summary(&mut output, &result.totals);
        output
    }
}

pub struct SummaryFormatter;

impl Formatter for SummaryFormatter {
    fn write(&self, result: &SimulationResult) -> String {
        let mut output = String::new();
        write_metadata(&mut output, result);
        write_summary(&mut output, &result.totals);
        output
    }
}

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn write(&self, result: &SimulationResult) -> String {
        let assignments = result
            .assignments
            .iter()
            .map(|assignment| JsonAssignment {
                request_id: assignment.request_id,
                server_id: assignment.server_id,
                server_name: server_name_for(assignment, &result.totals),
                arrival_time_ms: assignment.arrival_time_ms,
                started_at: assignment.started_at,
                completed_at: assignment.completed_at,
                score: assignment.score,
            })
            .collect::<Vec<_>>();
        let json = JsonSimulationResult {
            assignments,
            totals: &result.totals,
            metadata: &result.metadata,
            phase1_metrics: &result.phase1_metrics,
        };
        serde_json::to_string_pretty(&json).unwrap()
    }
}

fn write_metadata(output: &mut String, result: &SimulationResult) {
    output.push_str("Metadata:\n");
    output.push_str(&format!("algo: {}\n", result.metadata.algo));
    output.push_str(&format!("tie_break: {}\n", result.metadata.tie_break));
    output.push_str(&format!("duration_ms: {}\n", result.metadata.duration_ms));
}

fn write_summary(output: &mut String, totals: &[ServerSummary]) {
    output.push_str("Summary:\n");
    for summary in totals {
        output.push_str(&format!(
            "{}: {} requests (avg response: {}ms)\n",
            summary.name, summary.requests, summary.avg_response_ms
        ));
    }
}

fn write_assignment_with_totals(
    output: &mut String,
    assignment: &Assignment,
    totals: &[ServerSummary],
) {
    let server_name = server_name_for(assignment, totals);
    if let Some(score) = assignment.score {
        output.push_str(&format!(
            "Request {} -> {} (score: {}ms)\n",
            assignment.request_id, server_name, score
        ));
    } else {
        output.push_str(&format!(
            "Request {} -> {}\n",
            assignment.request_id, server_name
        ));
    }
}

fn server_name_for<'a>(assignment: &Assignment, totals: &'a [ServerSummary]) -> &'a str {
    totals
        .get(assignment.server_id)
        .map(|summary| summary.name.as_str())
        .unwrap_or("unknown")
}

#[derive(Serialize)]
struct JsonAssignment<'a> {
    request_id: usize,
    server_id: usize,
    server_name: &'a str,
    arrival_time_ms: u64,
    started_at: u64,
    completed_at: u64,
    score: Option<u64>,
}

#[derive(Serialize)]
struct JsonSimulationResult<'a> {
    assignments: Vec<JsonAssignment<'a>>,
    totals: &'a [ServerSummary],
    metadata: &'a RunMetadata,
    phase1_metrics: &'a Phase1Metrics,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Assignment, RunMetadata, ServerSummary, SimulationResult};

    fn sample_result() -> SimulationResult {
        SimulationResult {
            assignments: vec![Assignment {
                request_id: 1,
                server_id: 0,
                arrival_time_ms: 0,
                score: Some(10),
                started_at: 0,
                completed_at: 10,
            }],
            totals: vec![ServerSummary {
                name: "api".to_string(),
                requests: 1,
                avg_response_ms: 10,
            }],
            metadata: RunMetadata {
                algo: "round-robin".to_string(),
                tie_break: "stable".to_string(),
                duration_ms: 10,
            },
            phase1_metrics: Phase1Metrics {
                response_time: crate::state::ResponseTimePercentiles {
                    p95_ms: Some(10),
                    p99_ms: Some(10),
                },
                per_server_utilization: vec![crate::state::ServerUtilization {
                    name: "api".to_string(),
                    utilization_pct: 100.0,
                }],
                jain_fairness: 1.0,
                throughput_rps: 100.0,
                avg_wait_ms: 0,
            },
        }
    }

    #[test]
    fn human_formatter_includes_assignments_and_summary() {
        let formatter = HumanFormatter;
        let output = formatter.write(&sample_result());
        let expected = concat!(
            "Metadata:\n",
            "algo: round-robin\n",
            "tie_break: stable\n",
            "duration_ms: 10\n",
            "Assignments:\n",
            "Request 1 -> api (score: 10ms)\n",
            "Summary:\n",
            "api: 1 requests (avg response: 10ms)\n",
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn summary_formatter_includes_metadata_and_summary_only() {
        let formatter = SummaryFormatter;
        let output = formatter.write(&sample_result());
        let expected = concat!(
            "Metadata:\n",
            "algo: round-robin\n",
            "tie_break: stable\n",
            "duration_ms: 10\n",
            "Summary:\n",
            "api: 1 requests (avg response: 10ms)\n",
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn json_formatter_emits_structured_output() {
        let formatter = JsonFormatter;
        let output = formatter.write(&sample_result());
        let expected = r#"{
  "assignments": [
    {
      "request_id": 1,
      "server_id": 0,
      "server_name": "api",
      "arrival_time_ms": 0,
      "started_at": 0,
      "completed_at": 10,
      "score": 10
    }
  ],
  "totals": [
    {
      "name": "api",
      "requests": 1,
      "avg_response_ms": 10
    }
  ],
  "metadata": {
    "algo": "round-robin",
    "tie_break": "stable",
    "duration_ms": 10
  },
  "phase1_metrics": {
    "response_time": {
      "p95_ms": 10,
      "p99_ms": 10
    },
    "per_server_utilization": [
      {
        "name": "api",
        "utilization_pct": 100.0
      }
    ],
    "jain_fairness": 1.0,
    "throughput_rps": 100.0,
    "avg_wait_ms": 0
  }
}"#;
        assert_eq!(output, expected);
    }
}
