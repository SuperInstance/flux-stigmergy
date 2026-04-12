/// Types of stigmergic traces agents can leave in the shared environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceType {
    Info,
    Warning,
    Claim,
    Waypoint,
    Boundary,
}

/// A single trace deposited by an agent.
#[derive(Debug, Clone)]
pub struct Trace {
    pub author_id: u16,
    pub key: String,
    pub value: String,
    pub timestamp: u64,
    pub strength: u32,
    pub reads: u32,
    pub trace_type: TraceType,
}

impl Trace {
    pub fn new(
        author_id: u16,
        key: impl Into<String>,
        value: impl Into<String>,
        timestamp: u64,
        strength: u32,
        trace_type: TraceType,
    ) -> Self {
        Self {
            author_id,
            key: key.into(),
            value: value.into(),
            timestamp,
            strength: strength.min(1000),
            reads: 0,
            trace_type,
        }
    }
}

/// A waypoint built by following a chain of trace references.
#[derive(Debug, Clone)]
pub struct Waypoint {
    pub path: Vec<usize>,
    pub builder_id: u16,
}

impl Waypoint {
    pub fn new(builder_id: u16) -> Self {
        Self {
            path: Vec::new(),
            builder_id,
        }
    }

    /// Build a waypoint from a list of trace indices.
    pub fn from_trace_ids(builder_id: u16, indices: Vec<usize>) -> Self {
        Self {
            path: indices,
            builder_id,
        }
    }

    /// Follow the waypoint path to retrieve traces from the environment.
    pub fn follow<'a>(&self, traces: &'a [Trace]) -> Vec<&'a Trace> {
        self.path
            .iter()
            .filter_map(|&i| traces.get(i))
            .collect()
    }
}
