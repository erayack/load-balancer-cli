use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub id: usize,
    pub arrival_time_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    RequestArrival(Request),
    RequestComplete { server_id: usize, request_id: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledEvent {
    pub time_ms: u64,
    pub event: Event,
}

impl ScheduledEvent {
    pub fn new(time_ms: u64, event: Event) -> Self {
        Self { time_ms, event }
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time_ms
            .cmp(&other.time_ms)
            .then_with(|| self.event.priority().cmp(&other.event.priority()))
            .then_with(|| self.event.tiebreaker().cmp(&other.event.tiebreaker()))
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Event {
    fn priority(&self) -> u8 {
        match self {
            Event::RequestComplete { .. } => 0,
            Event::RequestArrival(_) => 1,
        }
    }

    fn tiebreaker(&self) -> usize {
        match self {
            Event::RequestComplete { request_id, .. } => *request_id,
            Event::RequestArrival(request) => request.id,
        }
    }
}
