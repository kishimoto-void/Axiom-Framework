use lrp_research::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LRP v{} — Reasoning Transition Protocol (Strict Final) ===", VERSION);
    println!("Philosophy: {}\n", PHILOSOPHY[..4].join(" / "));

    let conditions = vec!["baseline", "ablation_low_conf", "ablation_fail_validation"];
    let mut results = Vec::new();

    for (i, cond) in conditions.iter().enumerate() {
        let seed = 100 + i;
        let session = run_condition(seed, cond)?;
        let m = compute_session_metrics(&session);
        results.push((cond.to_string(), m));

        println!("--- condition: {} (seed={}) ---", cond, seed);
        println!("{}\n", paper_summary(&session));
    }

    println!("=== Absolute Determinism Self-Test ===");
    let s1 = run_condition(42, "baseline")?;
    let s2 = run_condition(42, "baseline")?;

    let json1 = serde_json::to_string_pretty(&s1)?;
    let json2 = serde_json::to_string_pretty(&s2)?;

    if json1 == json2 {
        println!("SUCCESS: 100% Binary/String Determinism Verified across executions!");
        println!("JSON Payload Byte Length: {} bytes", json1.len());
    } else {
        panic!("FATAL: Determinism Broken!");
    }

    // Extra: show that panic isolation works in live run
    println!("\n=== Observer Panic Isolation Live Check ===");
    let runtime = LRPRuntime::new(55, None).with_observers(vec![
        std::sync::Arc::new(LatencyObserver),
        std::sync::Arc::new(PanickingObserver),
        std::sync::Arc::new(MetricObserver),
    ]);
    let mut sess = runtime.create_session("live_panic", vec![], vec![], None, "exp", "live")?;
    sess = runtime.transition(
        &sess,
        ReasoningPrimitive::Observe,
        "live isolation test",
        vec![],
        vec![],
        vec![],
        vec![],
        true,
        "",
        vec![],
    )?;
    println!("Observer records after intentional panic:");
    for (i, r) in sess.observer_records.iter().enumerate() {
        println!(
            "  [{}] protocol={} success={} error={:?}",
            i,
            r.protocol_id,
            r.error.is_none(),
            r.error
        );
    }
    println!("Runtime survived. Isolation confirmed.");

    Ok(())
}
