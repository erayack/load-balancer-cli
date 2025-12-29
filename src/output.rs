use crate::state::{Assignment, ServerSummary, SimulationResult};

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
            write_assignment(&mut output, assignment);
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
        serde_json::to_string_pretty(result).unwrap()
    }
}

fn write_metadata(output: &mut String, result: &SimulationResult) {
    output.push_str("Metadata:\n");
    output.push_str(&format!("algo: {}\n", result.metadata.algo));
    output.push_str(&format!("tie_break: {}\n", result.metadata.tie_break));
    output.push_str(&format!("duration_ms: {}\n", result.metadata.duration_ms));
}

fn write_assignment(output: &mut String, assignment: &Assignment) {
    if let Some(score) = assignment.score {
        output.push_str(&format!(
            "Request {} -> {} (score: {}ms)\n",
            assignment.request_id, assignment.server_name, score
        ));
    } else {
        output.push_str(&format!(
            "Request {} -> {}\n",
            assignment.request_id, assignment.server_name
        ));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Assignment, RunMetadata, ServerSummary, SimulationResult};

    fn sample_result() -> SimulationResult {
        SimulationResult {
            assignments: vec![Assignment {
                request_id: 1,
                server_id: 0,
                server_name: "api".to_string(),
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
  }
}"#;
        assert_eq!(output, expected);
    }
}
