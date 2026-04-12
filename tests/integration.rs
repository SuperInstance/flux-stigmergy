use flux_stigmergy::*;

/// Integration test: full lifecycle — deposit, read, modify, decay, gc.
#[test]
fn full_lifecycle() {
    let mut env = SharedEnvironment::new();

    // Agent 1 deposits an info trace
    env.deposit(Trace::new(1, "status:health", "ok", 0, 800, TraceType::Info));

    // Agent 2 reads it
    let t = env.read("status:health").unwrap();
    assert_eq!(t.value, "ok");
    assert_eq!(t.reads, 1);

    // Agent 1 modifies it
    assert!(env.modify(1, "status:health", "warn", 50));
    let t2 = env.read("status:health").unwrap();
    assert_eq!(t2.value, "warn");
    assert_eq!(t2.strength, 850);

    // Agent 2 cannot modify it
    assert!(!env.modify(2, "status:health", "hacked", 100));

    // GC removes nothing (strength 850 > 100)
    assert_eq!(env.gc(100), 0);

    // Agent 1 erases it
    assert!(env.erase(1, "status:health"));
    assert!(env.read("status:health").is_none());
}

/// Integration test: multiple agents collaborate via traces.
#[test]
fn multi_agent_collaboration() {
    let mut env = SharedEnvironment::new();

    // Agent 1 leaves waypoints
    env.deposit(Trace::new(1, "wp:hallway", "clear", 0, 600, TraceType::Waypoint));
    env.deposit(Trace::new(1, "wp:exit", "blocked", 0, 700, TraceType::Waypoint));

    // Agent 2 leaves a warning
    env.deposit(Trace::new(2, "alert:exit", "blocked", 0, 500, TraceType::Warning));

    // Agent 3 reads waypoints
    let wps = env.read_all("wp:", 10);
    assert_eq!(wps.len(), 2);

    // Agent 3 reads alerts
    let alerts = env.read_all("alert:", 10);
    assert_eq!(alerts.len(), 1);

    // Verify by-author queries
    assert_eq!(env.by_author(1).len(), 2);
    assert_eq!(env.by_author(2).len(), 1);
    assert_eq!(env.by_author(3).len(), 0);
}

/// Integration test: decay removes weak traces while preserving strong ones.
#[test]
fn decay_removes_weak_traces() {
    let mut env = SharedEnvironment::new();

    // Weak trace (will be removed by GC threshold)
    env.deposit(Trace::new(1, "weak", "val", 0, 20, TraceType::Info));

    // Strong trace (will survive)
    env.deposit(Trace::new(1, "strong", "val", 0, 800, TraceType::Info));

    // Mild decay: half_life=1000s, now=100, age=100 for both, read_boost=0
    // lambda = ln(2)/1000 = 0.000693
    // weak: 20 * exp(-0.0693) ≈ 18.66
    // strong: 800 * exp(-0.0693) ≈ 746.5
    let removed = env.decay(1000, 0.0, 19, 100);
    assert_eq!(removed, 1, "Should have removed the weak trace");

    let remaining = env.stats().total_traces;
    assert_eq!(remaining, 1);
    assert!(env.read("strong").is_some());
}

/// Integration test: waypoint path navigation.
#[test]
fn waypoint_navigation() {
    let mut env = SharedEnvironment::new();

    // Build a path of 5 waypoints
    for i in 0..5 {
        env.deposit(Trace::new(
            1,
            format!("wp:{}", i),
            format!("step_{}", i),
            100 + i as u64,
            500,
            TraceType::Waypoint,
        ));
    }

    let wp = Waypoint::from_trace_ids(1, vec![0, 1, 2, 3, 4]);
    let all_wps: Vec<Trace> = env.read_all("wp:", 10).into_iter().cloned().collect();
    let path = wp.follow(&all_wps);

    assert_eq!(path.len(), 5);
    assert_eq!(path[0].value, "step_0");
    assert_eq!(path[4].value, "step_4");
}

/// Integration test: trace types partitioning via by_type.
#[test]
fn trace_type_partitioning() {
    let mut env = SharedEnvironment::new();

    env.deposit(Trace::new(1, "a", "1", 0, 100, TraceType::Info));
    env.deposit(Trace::new(1, "b", "2", 0, 100, TraceType::Warning));
    env.deposit(Trace::new(1, "c", "3", 0, 100, TraceType::Claim));
    env.deposit(Trace::new(1, "d", "4", 0, 100, TraceType::Waypoint));
    env.deposit(Trace::new(1, "e", "5", 0, 100, TraceType::Boundary));

    for tt in &[TraceType::Info, TraceType::Warning, TraceType::Claim, TraceType::Waypoint, TraceType::Boundary] {
        assert_eq!(env.by_type(tt).len(), 1, "Expected 1 trace of type {:?}", tt);
    }

    let stats = env.stats();
    assert_eq!(stats.total_traces, 5);
    assert_eq!(stats.by_type, [1, 1, 1, 1, 1]);
}

/// Integration test: strongest and oldest queries.
#[test]
fn strongest_and_oldest_queries() {
    let mut env = SharedEnvironment::new();

    env.deposit(Trace::new(1, "a", "1", 300, 100, TraceType::Info));
    env.deposit(Trace::new(1, "b", "2", 100, 900, TraceType::Info));
    env.deposit(Trace::new(1, "c", "3", 200, 500, TraceType::Info));

    let strongest = env.strongest(2);
    assert_eq!(strongest[0].key, "b");
    assert_eq!(strongest[1].key, "c");

    let oldest = env.oldest(2);
    assert_eq!(oldest[0].key, "b");
    assert_eq!(oldest[1].key, "c");
}

/// Integration test: read_boost during decay.
#[test]
fn read_boost_preserves_popular_traces() {
    let mut env = SharedEnvironment::new();

    // Both traces have same age and initial strength
    env.deposit(Trace::new(1, "popular", "val", 0, 100, TraceType::Info));
    env.deposit(Trace::new(1, "unpopular", "val", 0, 100, TraceType::Info));

    // Read "popular" many times
    for _ in 0..10 {
        env.read("popular");
    }

    // Apply decay with high read_boost
    env.decay(10, 50.0, 10, 100);

    // "popular" should survive, "unpopular" may not
    assert!(env.read("popular").is_some(), "Popular trace should survive with read boost");
}

/// Integration test: strength capping at 1000.
#[test]
fn strength_cap_enforced() {
    let mut env = SharedEnvironment::new();
    env.deposit(Trace::new(1, "cap", "val", 0, 999, TraceType::Info));
    env.modify(1, "cap", "val2", 500); // would push to 1499

    let t = env.read("cap").unwrap();
    assert_eq!(t.strength, 1000);
}
