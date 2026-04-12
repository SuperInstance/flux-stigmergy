# flux-stigmergy

Stigmergic communication for multi-agent systems. Agents leave traces in a shared environment, and other agents react to those traces — indirect coordination through the environment.

## Core Concepts

- **Trace**: A message left by an agent (key-value with metadata)
- **SharedEnvironment**: The shared space where traces are deposited and consumed
- **Waypoint**: A path through related traces, built by following trace references
- **Decay**: Traces lose strength over time (exponential decay), but reading boosts them

## Usage

```rust
use flux_stigmergy::{SharedEnvironment, Trace, TraceType};

let mut env = SharedEnvironment::new();
env.deposit(Trace::new(1, "location:door", "unlocked", 1000, 800, TraceType::Info));

if let Some(trace) = env.read("location:door") {
    println!("Found: {}", trace.value);
}
```
