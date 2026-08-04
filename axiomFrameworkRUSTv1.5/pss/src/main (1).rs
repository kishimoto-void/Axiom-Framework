use pss::*;

fn main() {
    println!("{}", "=".repeat(60));
    println!("PSS v1.0.0-rc1 — Production Reference Implementation");
    println!("{}", "=".repeat(60));

    let (spec, gate) = ProblemBuilder::new()
        .identity("リリース判断", "product", "新機能リリースのリスク評価")
        .main_mission("来月リリースの可否判断", &["重大バグゼロ"], Priority::Critical)
        .add_sub_mission(SubMissionKind::AskMissing, "負荷試験結果の確認", Priority::High, false)
        .knowledge(&["開発完了率95%"], &[], &[], &["負荷試験データ"], &["負荷試験データ"])
        .thinking_profile(
            ThinkingStance::Cautious,
            Perspective::Reviewer,
            ThinkingDepth::Normal,
            ReasoningBias::RiskFirst,
            EvidenceLevel::Low,
            "リスク優先で査定",
        )
        .prediction_policy(
            true,
            EvidenceLevel::High,
            BehaviorAction::Ask,
            true,
            true,
            true,
        )
        .evaluation(vec![("安全性", 0.7, ""), ("スピード", 0.3, "")])
        .phase(Phase::Clarify, 1, "", false)
        .build()
        .with_gate_evaluated();

    println!("{}", spec.summary());
    println!("Gate Proceed: {}", gate.can_proceed);

    // テスト兼用の JSON 出力
    let json_str = spec.to_json().expect("JSON serialization failed");
    println!("\nGenerated JSON (safe bytes preview):\n{}", safe_truncate_bytes(&json_str, 300));
    println!("...");
}
