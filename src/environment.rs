use crate::trace::{Trace, TraceType, Waypoint};

/// Aggregate statistics about the shared environment.
#[derive(Debug, Default)]
pub struct Stats {
    pub total_traces: usize,
    pub total_reads: u64,
    pub avg_strength: f64,
    pub by_type: [usize; 5], // Info, Warning, Claim, Waypoint, Boundary
}

/// Shared environment where agents deposit and consume stigmergic traces.
#[derive(Debug, Default)]
pub struct SharedEnvironment {
    traces: Vec<Trace>,
}

impl SharedEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    /// Deposit a new trace into the environment.
    pub fn deposit(&mut self, trace: Trace) -> usize {
        let idx = self.traces.len();
        self.traces.push(trace);
        idx
    }

    /// Read a trace by exact key (increments read count).
    pub fn read(&mut self, key: &str) -> Option<&Trace> {
        self.traces.iter_mut().find(|t| t.key == key).map(|t| {
            t.reads += 1;
            &*t
        })
    }

    /// Read traces whose key starts with `prefix`, up to `max` results.
    pub fn read_all(&mut self, prefix: &str, max: usize) -> Vec<&Trace> {
        self.traces
            .iter_mut()
            .filter(|t| t.key.starts_with(prefix))
            .take(max)
            .map(|t| {
                t.reads += 1;
                &*t
            })
            .collect()
    }

    /// Modify a trace's value and add to its strength. Only the author can modify.
    pub fn modify(
        &mut self,
        author: u16,
        key: &str,
        new_value: &str,
        strength_add: u32,
    ) -> bool {
        if let Some(t) = self.traces.iter_mut().find(|t| t.key == key) {
            if t.author_id != author {
                return false;
            }
            t.value = new_value.to_string();
            t.strength = (t.strength as u32 + strength_add).min(1000);
            true
        } else {
            false
        }
    }

    /// Erase a trace by key. Only the author can erase their own traces.
    pub fn erase(&mut self, author: u16, key: &str) -> bool {
        if let Some(pos) = self.traces.iter().position(|t| t.key == key && t.author_id == author)
        {
            self.traces.remove(pos);
            true
        } else {
            false
        }
    }

    /// Apply exponential decay to all traces. Returns number removed by GC.
    pub fn decay(
        &mut self,
        half_life_secs: u64,
        read_boost: f64,
        min_strength: u32,
        now: u64,
    ) -> usize {
        let lambda = 0.6931471805599453 / (half_life_secs as f64); // ln(2)
        let before = self.traces.len();

        for t in &mut self.traces {
            let age = (now - t.timestamp) as f64;
            let decayed = (t.strength as f64) * (-lambda * age).exp();
            let boosted = decayed + (t.reads as f64) * read_boost;
            t.strength = (boosted.max(0.0) as u32).min(1000);
        }

        self.traces.retain(|t| t.strength >= min_strength);
        before - self.traces.len()
    }

    /// Remove traces with strength below threshold.
    pub fn gc(&mut self, min_strength: u32) -> usize {
        let before = self.traces.len();
        self.traces.retain(|t| t.strength >= min_strength);
        before - self.traces.len()
    }

    /// Get all traces by a specific author.
    pub fn by_author(&self, author_id: u16) -> Vec<&Trace> {
        self.traces.iter().filter(|t| t.author_id == author_id).collect()
    }

    /// Get all traces of a specific type.
    pub fn by_type(&self, trace_type: &TraceType) -> Vec<&Trace> {
        self.traces.iter().filter(|t| &t.trace_type == trace_type).collect()
    }

    /// Get the n strongest traces.
    pub fn strongest(&self, n: usize) -> Vec<&Trace> {
        let mut refs: Vec<&Trace> = self.traces.iter().collect();
        refs.sort_by_key(|t| std::cmp::Reverse(t.strength));
        refs.truncate(n);
        refs
    }

    /// Get the n oldest traces.
    pub fn oldest(&self, n: usize) -> Vec<&Trace> {
        let mut refs: Vec<&Trace> = self.traces.iter().collect();
        refs.sort_by_key(|t| t.timestamp);
        refs.truncate(n);
        refs
    }

    /// Get aggregate statistics.
    pub fn stats(&self) -> Stats {
        let total_traces = self.traces.len();
        let total_reads: u64 = self.traces.iter().map(|t| t.reads as u64).sum();
        let avg_strength = if total_traces > 0 {
            self.traces.iter().map(|t| t.strength as f64).sum::<f64>() / total_traces as f64
        } else {
            0.0
        };
        let mut by_type = [0usize; 5];
        for t in &self.traces {
            match t.trace_type {
                TraceType::Info => by_type[0] += 1,
                TraceType::Warning => by_type[1] += 1,
                TraceType::Claim => by_type[2] += 1,
                TraceType::Waypoint => by_type[3] += 1,
                TraceType::Boundary => by_type[4] += 1,
            }
        }
        Stats {
            total_traces,
            total_reads,
            avg_strength,
            by_type,
        }
    }
}
