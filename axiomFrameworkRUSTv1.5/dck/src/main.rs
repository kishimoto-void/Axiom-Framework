use std::collections::HashMap;
use std::sync::Arc;

use dck_modular::{
    DCKConfig, Intent, IntentId, KernelBuilder, MetricGoal, ResourceVector,
    ReversibleResource, IrreversibleResource, StubObserver, StubPredictor, StubExecutor,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let initial_res = ResourceVector {
        rev: ReversibleResource {
            compute_cpu: 100.0,
            compute_gpu: 8.0,
            bandwidth: 1000.0,
        },
        irr: IrreversibleResource {
            capital_money: 10000.0,
            energy_power: 50.0,
            time_window: 100.0,
        },
    };

    let kernel = KernelBuilder::new(initial_res)
        .with_config(DCKConfig::default())
        .with_capabilities(
            Arc::new(StubObserver),
            Arc::new(StubPredictor),
            Arc::new(StubExecutor),
        )
        .build()?;

    let mut goals = HashMap::new();
    goals.insert(
        "temperature".into(),
        MetricGoal {
            target_value: 25.0,
            tolerance: 0.5,
        },
    );

    kernel
        .submit_intent(Intent {
            intent_id: IntentId::named("intent_01"),
            description: "Maintain temperature balance".into(),
            goals,
            time_horizon: 5,
            created_turn: 0,
            base_priority: 1.0,
            deadline_turn: None,
            dependencies: vec![],
        })
        .await;

    let mut raw_telemetry = HashMap::new();
    raw_telemetry.insert("temperature".into(), 42.0);

    let mut kernel = kernel;
    let events = kernel.tick(1, raw_telemetry).await?;

    println!("=== DCK Modular + nalgebra v2.0 Execution Results ===");
    for ev in &events {
        println!(
            "Event ID   : {}",
            ev.event_id
        );
        println!(
            "  Stage    : {:?}",
            ev.current_stage
        );
        println!(
            "  Action   : {:?}",
            ev.decision_action
        );
        println!(
            "  Gap      : {:.4}",
            ev.equivalence_gap
        );
        println!(
            "  Velocity : {:.4}",
            ev.computed_velocity
        );
        println!(
            "  Dim      : {}",
            ev.projected_state.dim()
        );
        println!();
    }

    Ok(())
}
