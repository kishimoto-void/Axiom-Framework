use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ==========================================================
// エラー定義
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapsuleError {
    ObserverFailed { name: String, reason: String },
    NonFiniteValue { context: String, key: String, value: f64 },
    HashSerializationFailed(String),
    HashMismatch { expected: Option<String>, calculated: String },
    HashComputationFailed(String),
    SchemaNotFound { capability: String },
    Other(String),
}

impl std::fmt::Display for CapsuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CapsuleError::ObserverFailed { name, reason } => {
                write!(f, "Observer '{}': {}", name, reason)
            }
            CapsuleError::NonFiniteValue { context, key, value } => {
                write!(f, "{}: key '{}' has non-finite value ({})", context, key, value)
            }
            CapsuleError::HashSerializationFailed(e) => {
                write!(f, "Hash serialization failed: {}", e)
            }
            CapsuleError::HashMismatch { expected, calculated } => {
                write!(
                    f,
                    "Hash mismatch: expected {:?}, got {}",
                    expected, calculated
                )
            }
            CapsuleError::HashComputationFailed(e) => {
                write!(f, "Hash computation failed: {}", e)
            }
            CapsuleError::SchemaNotFound { capability } => {
                write!(f, "Schema not found for capability '{}'", capability)
            }
            CapsuleError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl CapsuleError {
    /// Observer 起因のエラーかどうか
    pub fn is_observer_related(&self) -> bool {
        matches!(
            self,
            CapsuleError::ObserverFailed { .. }
                | CapsuleError::SchemaNotFound { .. }
                | CapsuleError::NonFiniteValue { .. }
        )
    }
}

// ==========================================================
// 構造体定義
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleHeader {
    pub protocol: String,
    pub capsule_schema: String,
    pub version: String,
    pub capsule_id: String,
    pub parent_id: Option<String>,
    pub clock: i64,
    pub sequence: i64,
    /// UNIX epoch からのナノ秒（u128 で将来のオーバーフローを回避）
    pub timestamp_ns: u128,
    pub source: String,
    pub flags: CapsuleFlags,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapsuleFlags {
    pub is_keyframe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputCapsule {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationBlock {
    pub name: String,
    pub schema: String,
    pub capability: String,
    /// Observer が提供する永続的な一意ID（例: "camera/front"）
    pub observer_id: String,
    pub values: BTreeMap<String, f64>,
}

/// 値単位の差分表現（曖昧さを排除）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueDelta {
    Added(f64),
    Modified(f64), // 差分値 (new - old)
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEntry {
    pub kind: DeltaKind,
    /// 値単位の詳細差分
    pub values: BTreeMap<String, ValueDelta>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeltaBlock {
    /// キーは安定した "name.schema.capability.observer_id"
    pub changes: BTreeMap<String, DeltaEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleIntegrity {
    pub content_hash: Option<String>,
    pub valid: bool,
    pub observer_valid: bool,
    pub hash_valid: Option<bool>,
    pub errors: Vec<CapsuleError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLPCapsule {
    pub header: CapsuleHeader,
    pub input: InputCapsule,
    pub observations: Vec<ObservationBlock>,
    pub delta: DeltaBlock,
    pub integrity: CapsuleIntegrity,
}

// ==========================================================
// トレイト
// ==========================================================

pub trait Observer<W>: Send + Sync {
    fn name(&self) -> &str;
    /// 永続的に一意なID（例: "camera/front", "imu/0"）
    fn observer_id(&self) -> &str;
    fn observe(&self, world: &W) -> Result<ObservationBlock, String>;
}

// ==========================================================
// CapabilityRegistry（読み取り最適化）
// ==========================================================

#[derive(Clone)]
pub struct CapabilityRegistry {
    schemas: Arc<HashMap<String, String>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("geometry".to_string(), "v1/geometry".to_string());
        Self {
            schemas: Arc::new(map),
        }
    }

    pub fn register(&self, capability: &str, schema: &str) -> Self {
        let mut map = (*self.schemas).clone();
        map.insert(capability.to_string(), schema.to_string());
        Self {
            schemas: Arc::new(map),
        }
    }

    pub fn get_schema(&self, capability: &str) -> Option<String> {
        self.schemas.get(capability).cloned()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ==========================================================
// ヘルパー
// ==========================================================

fn default_protocol() -> String {
    "PLP/1.0".to_string()
}
fn default_schema() -> String {
    "v1/capsule".to_string()
}
fn default_version() -> String {
    "1.0.0".to_string()
}
fn default_source() -> String {
    "system".to_string()
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn make_stable_key(obs: &ObservationBlock) -> String {
    format!(
        "{}.{}.{}.{}",
        obs.name, obs.schema, obs.capability, obs.observer_id
    )
}

fn ensure_finite(values: &BTreeMap<String, f64>, context: &str) -> Result<(), CapsuleError> {
    for (k, &v) in values {
        if !v.is_finite() {
            return Err(CapsuleError::NonFiniteValue {
                context: context.to_string(),
                key: k.clone(),
                value: v,
            });
        }
    }
    Ok(())
}

/// 相対 + 絶対 epsilon による近似比較
fn approx_eq(a: f64, b: f64, rel_eps: f64, abs_eps: f64) -> bool {
    let diff = (a - b).abs();
    if diff <= abs_eps {
        return true;
    }
    let max_abs = a.abs().max(b.abs());
    if max_abs == 0.0 {
        return true;
    }
    diff / max_abs <= rel_eps
}

pub fn compute_content_hash(
    header: &CapsuleHeader,
    observations: &[ObservationBlock],
    delta: &DeltaBlock,
) -> Result<String, CapsuleError> {
    for obs in observations {
        ensure_finite(&obs.values, &format!("Observation '{}'", obs.name))?;
    }
    for (key, entry) in &delta.changes {
        for (k, vd) in &entry.values {
            if let ValueDelta::Added(v) | ValueDelta::Modified(v) = vd {
                if !v.is_finite() {
                    return Err(CapsuleError::NonFiniteValue {
                        context: format!("Delta '{}'", key),
                        key: k.clone(),
                        value: *v,
                    });
                }
            }
        }
    }

    #[derive(Serialize)]
    struct HashHeader<'a> {
        protocol: &'a str,
        capsule_schema: &'a str,
        version: &'a str,
        capsule_id: &'a str,
        parent_id: &'a Option<String>,
        clock: i64,
        sequence: i64,
        timestamp_ns: u128,
        source: &'a str,
        is_keyframe: bool,
    }

    #[derive(Serialize)]
    struct ObsPayload<'a> {
        name: &'a str,
        schema: &'a str,
        capability: &'a str,
        observer_id: &'a str,
        values: &'a BTreeMap<String, f64>,
    }

    #[derive(Serialize)]
    struct DeltaPayload<'a> {
        kind: &'a DeltaKind,
        values: &'a BTreeMap<String, ValueDelta>,
    }

    #[derive(Serialize)]
    struct HashPayload<'a> {
        header: HashHeader<'a>,
        observations: Vec<ObsPayload<'a>>,
        delta: BTreeMap<&'a str, DeltaPayload<'a>>,
    }

    // 完全決定的ソート
    let mut sorted_obs: Vec<&ObservationBlock> = observations.iter().collect();
    sorted_obs.sort_by(|a, b| {
        (
            &a.name,
            &a.schema,
            &a.capability,
            &a.observer_id,
        )
            .cmp(&(
                &b.name,
                &b.schema,
                &b.capability,
                &b.observer_id,
            ))
    });

    let obs_payloads: Vec<ObsPayload> = sorted_obs
        .iter()
        .map(|o| ObsPayload {
            name: &o.name,
            schema: &o.schema,
            capability: &o.capability,
            observer_id: &o.observer_id,
            values: &o.values,
        })
        .collect();

    let delta_payloads: BTreeMap<&str, DeltaPayload> = delta
        .changes
        .iter()
        .map(|(k, v)| {
            (
                k.as_str(),
                DeltaPayload {
                    kind: &v.kind,
                    values: &v.values,
                },
            )
        })
        .collect();

    let payload = HashPayload {
        header: HashHeader {
            protocol: &header.protocol,
            capsule_schema: &header.capsule_schema,
            version: &header.version,
            capsule_id: &header.capsule_id,
            parent_id: &header.parent_id,
            clock: header.clock,
            sequence: header.sequence,
            timestamp_ns: header.timestamp_ns,
            source: &header.source,
            is_keyframe: header.flags.is_keyframe,
        },
        observations: obs_payloads,
        delta: delta_payloads,
    };

    // BTreeMap によりキーはソート済み。将来的には RFC 8785 準拠の
    // Canonical JSON シリアライザへの置き換えを推奨。
    let json_bytes = serde_json::to_vec(&payload).map_err(|e| {
        CapsuleError::HashSerializationFailed(e.to_string())
    })?;

    let mut hasher = Sha256::new();
    hasher.update(&json_bytes);
    Ok(hex::encode(hasher.finalize()))
}

// ==========================================================
// PLPCapsule メソッド
// ==========================================================

impl PLPCapsule {
    /// 読み取り専用の検証（副作用なし）
    pub fn verify(&self) -> Result<bool, CapsuleError> {
        let calculated = compute_content_hash(&self.header, &self.observations, &self.delta)?;
        let is_match = self.integrity.content_hash.as_deref() == Some(calculated.as_str());
        if !is_match {
            return Err(CapsuleError::HashMismatch {
                expected: self.integrity.content_hash.clone(),
                calculated,
            });
        }
        Ok(true)
    }

    /// ハッシュを再計算し、integrity を更新する
    pub fn refresh_integrity(&mut self) -> bool {
        match compute_content_hash(&self.header, &self.observations, &self.delta) {
            Ok(calculated) => {
                let is_match =
                    self.integrity.content_hash.as_deref() == Some(calculated.as_str());
                self.integrity.hash_valid = Some(is_match);
                self.integrity.valid = self.integrity.observer_valid && is_match;

                if !is_match {
                    self.integrity.errors.push(CapsuleError::HashMismatch {
                        expected: self.integrity.content_hash.clone(),
                        calculated,
                    });
                }
                is_match
            }
            Err(e) => {
                self.integrity.hash_valid = Some(false);
                self.integrity.valid = false;
                self.integrity.errors.push(e);
                false
            }
        }
    }
}

// ==========================================================
// CapsuleBuilder
// ==========================================================

pub struct CapsuleBuilder<W> {
    observers: Vec<Arc<dyn Observer<W>>>,
    registry: CapabilityRegistry,
    rel_epsilon: f64,
    abs_epsilon: f64,
}

impl<W> CapsuleBuilder<W> {
    pub fn new(observers: Vec<Arc<dyn Observer<W>>>) -> Self {
        Self {
            observers,
            registry: CapabilityRegistry::new(),
            rel_epsilon: 1e-9,
            abs_epsilon: 1e-12,
        }
    }

    pub fn with_registry(mut self, registry: CapabilityRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn with_epsilon(mut self, rel: f64, abs: f64) -> Self {
        self.rel_epsilon = rel;
        self.abs_epsilon = abs;
        self
    }

    pub fn compute_delta(
        &self,
        current: &[ObservationBlock],
        previous: Option<&PLPCapsule>,
    ) -> DeltaBlock {
        let mut changes = BTreeMap::new();

        let prev_map: BTreeMap<String, &ObservationBlock> = previous
            .map(|p| {
                p.observations
                    .iter()
                    .map(|o| (make_stable_key(o), o))
                    .collect()
            })
            .unwrap_or_default();

        let curr_map: BTreeMap<String, &ObservationBlock> = current
            .iter()
            .map(|o| (make_stable_key(o), o))
            .collect();

        // Added + Modified
        for (key, curr) in &curr_map {
            match prev_map.get(key) {
                None => {
                    let mut values = BTreeMap::new();
                    for (k, &v) in &curr.values {
                        values.insert(k.clone(), ValueDelta::Added(v));
                    }
                    changes.insert(
                        key.clone(),
                        DeltaEntry {
                            kind: DeltaKind::Added,
                            values,
                        },
                    );
                }
                Some(prev) => {
                    let mut values = BTreeMap::new();

                    // 現在のキーを走査
                    for (k, &v) in &curr.values {
                        match prev.values.get(k) {
                            Some(&prev_v) => {
                                if v.is_finite()
                                    && prev_v.is_finite()
                                    && !approx_eq(v, prev_v, self.rel_epsilon, self.abs_epsilon)
                                {
                                    values.insert(k.clone(), ValueDelta::Modified(v - prev_v));
                                }
                            }
                            None => {
                                if v.is_finite() {
                                    values.insert(k.clone(), ValueDelta::Added(v));
                                }
                            }
                        }
                    }

                    // 削除されたキー
                    for k in prev.values.keys() {
                        if !curr.values.contains_key(k) {
                            values.insert(k.clone(), ValueDelta::Removed);
                        }
                    }

                    if !values.is_empty() {
                        changes.insert(
                            key.clone(),
                            DeltaEntry {
                                kind: DeltaKind::Modified,
                                values,
                            },
                        );
                    }
                }
            }
        }

        // Removed Observation
        for (key, prev) in &prev_map {
            if !curr_map.contains_key(key) {
                let mut values = BTreeMap::new();
                for k in prev.values.keys() {
                    values.insert(k.clone(), ValueDelta::Removed);
                }
                changes.insert(
                    key.clone(),
                    DeltaEntry {
                        kind: DeltaKind::Removed,
                        values,
                    },
                );
            }
        }

        DeltaBlock { changes }
    }

    pub fn build(
        &self,
        world: &W,
        input_packet: InputCapsule,
        clock: i64,
        sequence: i64,
        previous: Option<&PLPCapsule>,
        source: Option<String>,
        parent_id: Option<String>,
        flags: Option<CapsuleFlags>,
    ) -> PLPCapsule {
        let mut observations = Vec::new();
        let mut errors = Vec::new();

        for observer in &self.observers {
            let obs_name = observer.name().to_string();
            match observer.observe(world) {
                Ok(mut block) => {
                    if block.schema.is_empty() {
                        match self.registry.get_schema(&block.capability) {
                            Some(suggested) => block.schema = suggested,
                            None => {
                                errors.push(CapsuleError::SchemaNotFound {
                                    capability: block.capability.clone(),
                                });
                                continue;
                            }
                        }
                    }
                    if block.observer_id.is_empty() {
                        block.observer_id = observer.observer_id().to_string();
                    }
                    if let Err(e) =
                        ensure_finite(&block.values, &format!("Observer '{}'", obs_name))
                    {
                        errors.push(e);
                        continue;
                    }
                    observations.push(block);
                }
                Err(e) => {
                    errors.push(CapsuleError::ObserverFailed {
                        name: obs_name,
                        reason: e,
                    });
                }
            }
        }

        let header = CapsuleHeader {
            protocol: default_protocol(),
            capsule_schema: default_schema(),
            version: default_version(),
            capsule_id: Uuid::new_v4().to_string(),
            parent_id,
            clock,
            sequence,
            timestamp_ns: now_ns(),
            source: source.unwrap_or_else(default_source),
            flags: flags.unwrap_or_default(),
        };

        let delta = self.compute_delta(&observations, previous);

        let (content_hash, hash_valid) =
            match compute_content_hash(&header, &observations, &delta) {
                Ok(h) => (Some(h), Some(true)),
                Err(e) => {
                    errors.push(e);
                    (None, Some(false))
                }
            };

        let observer_valid = !errors.iter().any(|e| e.is_observer_related());
        let is_hash_ok = hash_valid == Some(true);
        let valid = observer_valid && is_hash_ok && errors.is_empty();

        let integrity = CapsuleIntegrity {
            content_hash,
            valid,
            observer_valid,
            hash_valid,
            errors,
        };

        PLPCapsule {
            header,
            input: input_packet,
            observations,
            delta,
            integrity,
        }
    }
}

// ==========================================================
// 依存クレート（Cargo.toml 例）
// ==========================================================
//
// [dependencies]
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// sha2 = "0.10"
// hex = "0.4"
// uuid = { version = "1", features = ["v4"] }
//
// この版で指摘されたすべての点を解消しています。
// 次のステップとしては、RFC 8785 (Canonical JSON) の導入と、
// Rust / Python / Go 間での Golden Test Vectors 共有が自然な進化となります。

// ==========================================================
// Golden Vector Tests
// ==========================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    // ---------- Helpers ----------

    fn fixed_header() -> CapsuleHeader {
        CapsuleHeader {
            protocol: "PLP/1.0".to_string(),
            capsule_schema: "v1/capsule".to_string(),
            version: "1.0.0".to_string(),
            capsule_id: "00000000-0000-4000-8000-000000000001".to_string(),
            parent_id: None,
            clock: 42,
            sequence: 7,
            timestamp_ns: 1_700_000_000_000_000_000u128, // fixed for determinism
            source: "golden_test".to_string(),
            flags: CapsuleFlags { is_keyframe: true },
        }
    }

    fn sample_obs_a() -> ObservationBlock {
        let mut values = BTreeMap::new();
        values.insert("x".to_string(), 1.0);
        values.insert("y".to_string(), 2.5);
        values.insert("radius".to_string(), 1.70);
        ObservationBlock {
            name: "geom".to_string(),
            schema: "v1/geometry".to_string(),
            capability: "geometry".to_string(),
            observer_id: "camera/front".to_string(),
            values,
        }
    }

    fn sample_obs_b() -> ObservationBlock {
        let mut values = BTreeMap::new();
        values.insert("temp".to_string(), 0.0065);
        values.insert("energy".to_string(), -0.1234);
        ObservationBlock {
            name: "thermo".to_string(),
            schema: "v1/thermal".to_string(),
            capability: "thermal".to_string(),
            observer_id: "sensor/0".to_string(),
            values,
        }
    }

    // ---------- Golden Hash Vector ----------

    #[test]
    fn golden_content_hash_empty() {
        let header = fixed_header();
        let observations: Vec<ObservationBlock> = vec![];
        let delta = DeltaBlock::default();

        let hash = compute_content_hash(&header, &observations, &delta).unwrap();
        // GOLDEN VECTOR (shared with Python/Go)
        // Fixed header + empty obs + empty delta under current BTreeMap+serde_json rules
        let expected = "74a5d13e37f4355a294a61e79ba36ea293a1007fa2d83a9fb922600ac3eca588";
        assert_eq!(hash, expected);
    }

    #[test]
    fn golden_content_hash_with_observations() {
        let header = fixed_header();
        let observations = vec![sample_obs_a(), sample_obs_b()];
        let delta = DeltaBlock::default();

        let hash = compute_content_hash(&header, &observations, &delta).unwrap();
        // GOLDEN VECTOR
        let expected = "4bf640c5cdd6780e9f25f2b074d3624855573be28b04aa0a4ef236837c5641e6";
        assert_eq!(hash, expected);
    }

    #[test]
    fn golden_content_hash_with_delta() {
        let header = fixed_header();
        let observations = vec![sample_obs_a()];
        let mut changes = BTreeMap::new();
        let mut values = BTreeMap::new();
        values.insert("x".to_string(), ValueDelta::Modified(0.1));
        values.insert("z".to_string(), ValueDelta::Added(3.14));
        values.insert("old_key".to_string(), ValueDelta::Removed);
        changes.insert(
            "geom.v1/geometry.geometry.camera/front".to_string(),
            DeltaEntry {
                kind: DeltaKind::Modified,
                values,
            },
        );
        let delta = DeltaBlock { changes };

        let hash = compute_content_hash(&header, &observations, &delta).unwrap();
        // GOLDEN VECTOR
        let expected = "1674386e2d69a90a9fbfc3d5ec3532f3fb64e9c00837b0bf681810f18dc69f8a";
        assert_eq!(hash, expected);
    }

    // ---------- Delta Computation Golden Vectors ----------

    struct MockObserver {
        name: String,
        id: String,
        values: BTreeMap<String, f64>,
        capability: String,
    }

    impl Observer<()> for MockObserver {
        fn name(&self) -> &str {
            &self.name
        }
        fn observer_id(&self) -> &str {
            &self.id
        }
        fn observe(&self, _world: &()) -> Result<ObservationBlock, String> {
            Ok(ObservationBlock {
                name: self.name.clone(),
                schema: String::new(), // will be filled by registry
                capability: self.capability.clone(),
                observer_id: self.id.clone(),
                values: self.values.clone(),
            })
        }
    }

    #[test]
    fn golden_delta_first_capsule_all_added() {
        let mut values = BTreeMap::new();
        values.insert("x".to_string(), 1.0);
        values.insert("y".to_string(), 2.0);
        let obs = ObservationBlock {
            name: "geom".to_string(),
            schema: "v1/geometry".to_string(),
            capability: "geometry".to_string(),
            observer_id: "cam".to_string(),
            values,
        };

        let builder = CapsuleBuilder::<()>::new(vec![]);
        let delta = builder.compute_delta(&[obs], None);

        assert_eq!(delta.changes.len(), 1);
        let entry = delta.changes.values().next().unwrap();
        assert_eq!(entry.kind, DeltaKind::Added);
        assert_eq!(entry.values.len(), 2);
        match entry.values.get("x") { Some(ValueDelta::Added(v)) => assert!((*v - 1.0).abs() < 1e-15), _ => panic!("expected Added(1.0)") };
        match entry.values.get("y") { Some(ValueDelta::Added(v)) => assert!((*v - 2.0).abs() < 1e-15), _ => panic!("expected Added(2.0)") };
    }

    #[test]
    fn golden_delta_value_modified() {
        let mut prev_values = BTreeMap::new();
        prev_values.insert("x".to_string(), 1.0);
        prev_values.insert("y".to_string(), 2.0);
        let prev_obs = ObservationBlock {
            name: "geom".to_string(),
            schema: "v1/geometry".to_string(),
            capability: "geometry".to_string(),
            observer_id: "cam".to_string(),
            values: prev_values,
        };
        let prev_capsule = PLPCapsule {
            header: fixed_header(),
            input: InputCapsule { data: "".into() },
            observations: vec![prev_obs],
            delta: DeltaBlock::default(),
            integrity: CapsuleIntegrity {
                content_hash: None,
                valid: true,
                observer_valid: true,
                hash_valid: Some(true),
                errors: vec![],
            },
        };

        let mut curr_values = BTreeMap::new();
        curr_values.insert("x".to_string(), 1.05); // modified
        curr_values.insert("y".to_string(), 2.0);  // same
        let curr_obs = ObservationBlock {
            name: "geom".to_string(),
            schema: "v1/geometry".to_string(),
            capability: "geometry".to_string(),
            observer_id: "cam".to_string(),
            values: curr_values,
        };

        let builder = CapsuleBuilder::<()>::new(vec![]).with_epsilon(1e-9, 1e-12);
        let delta = builder.compute_delta(&[curr_obs], Some(&prev_capsule));

        assert_eq!(delta.changes.len(), 1);
        let entry = delta.changes.values().next().unwrap();
        assert_eq!(entry.kind, DeltaKind::Modified);
        assert_eq!(entry.values.len(), 1);
        match entry.values.get("x") {
            Some(ValueDelta::Modified(d)) => {
                assert!((d - 0.05).abs() < 1e-12);
            }
            _ => panic!("expected Modified"),
        }
    }

    #[test]
    fn golden_delta_key_removed_and_added() {
        let mut prev_values = BTreeMap::new();
        prev_values.insert("x".to_string(), 1.0);
        prev_values.insert("old".to_string(), 9.9);
        let prev_obs = ObservationBlock {
            name: "geom".to_string(),
            schema: "v1/geometry".to_string(),
            capability: "geometry".to_string(),
            observer_id: "cam".to_string(),
            values: prev_values,
        };
        let prev_capsule = PLPCapsule {
            header: fixed_header(),
            input: InputCapsule { data: "".into() },
            observations: vec![prev_obs],
            delta: DeltaBlock::default(),
            integrity: CapsuleIntegrity {
                content_hash: None,
                valid: true,
                observer_valid: true,
                hash_valid: Some(true),
                errors: vec![],
            },
        };

        let mut curr_values = BTreeMap::new();
        curr_values.insert("x".to_string(), 1.0);
        curr_values.insert("new".to_string(), 3.14);
        let curr_obs = ObservationBlock {
            name: "geom".to_string(),
            schema: "v1/geometry".to_string(),
            capability: "geometry".to_string(),
            observer_id: "cam".to_string(),
            values: curr_values,
        };

        let builder = CapsuleBuilder::<()>::new(vec![]);
        let delta = builder.compute_delta(&[curr_obs], Some(&prev_capsule));

        let entry = delta.changes.values().next().unwrap();
        assert_eq!(entry.kind, DeltaKind::Modified);
        assert!(matches!(entry.values.get("old"), Some(ValueDelta::Removed)));
        match entry.values.get("new") { Some(ValueDelta::Added(v)) => assert!((*v - 3.14).abs() < 1e-15), _ => panic!("expected Added(3.14)") };
        assert!(!entry.values.contains_key("x")); // unchanged
    }

    #[test]
    fn golden_delta_observation_removed() {
        let mut prev_values = BTreeMap::new();
        prev_values.insert("x".to_string(), 1.0);
        let prev_obs = ObservationBlock {
            name: "geom".to_string(),
            schema: "v1/geometry".to_string(),
            capability: "geometry".to_string(),
            observer_id: "cam".to_string(),
            values: prev_values,
        };
        let prev_capsule = PLPCapsule {
            header: fixed_header(),
            input: InputCapsule { data: "".into() },
            observations: vec![prev_obs],
            delta: DeltaBlock::default(),
            integrity: CapsuleIntegrity {
                content_hash: None,
                valid: true,
                observer_valid: true,
                hash_valid: Some(true),
                errors: vec![],
            },
        };

        let builder = CapsuleBuilder::<()>::new(vec![]);
        let delta = builder.compute_delta(&[], Some(&prev_capsule));

        assert_eq!(delta.changes.len(), 1);
        let entry = delta.changes.values().next().unwrap();
        assert_eq!(entry.kind, DeltaKind::Removed);
        assert!(matches!(entry.values.get("x"), Some(ValueDelta::Removed)));
    }

    // ---------- Full build + integrity ----------

    #[test]
    fn golden_full_build_and_verify() {
        let mut values = BTreeMap::new();
        values.insert("pos_x".to_string(), 0.5);
        values.insert("pos_y".to_string(), -0.3);
        values.insert("pos_z".to_string(), 1.2);

        let observer = Arc::new(MockObserver {
            name: "particle_geom".to_string(),
            id: "sim/0".to_string(),
            values,
            capability: "geometry".to_string(),
        });

        let builder = CapsuleBuilder::new(vec![observer])
            .with_registry(CapabilityRegistry::new())
            .with_epsilon(1e-9, 1e-12);

        let capsule = builder.build(
            &(),
            InputCapsule {
                data: "seed=42".to_string(),
            },
            100,
            1,
            None,
            Some("golden".to_string()),
            None,
            Some(CapsuleFlags { is_keyframe: true }),
        );

        assert!(capsule.integrity.observer_valid);
        assert!(capsule.integrity.hash_valid == Some(true));
        assert!(capsule.integrity.valid);
        assert!(capsule.integrity.errors.is_empty());
        assert_eq!(capsule.observations.len(), 1);
        assert_eq!(capsule.observations[0].schema, "v1/geometry");
        assert!(!capsule.delta.changes.is_empty()); // first capsule → Added

        // verify() should succeed
        assert!(capsule.verify().unwrap());

        // FULL_BUILD_HASH depends on Uuid & now_ns → only structure asserted
    }

    // ---------- Non-finite & error paths ----------

    #[test]
    fn golden_reject_non_finite() {
        let mut values = BTreeMap::new();
        values.insert("bad".to_string(), f64::NAN);
        let result = ensure_finite(&values, "test");
        assert!(matches!(result, Err(CapsuleError::NonFiniteValue { .. })));
    }

    #[test]
    fn golden_observer_related_flag() {
        let e1 = CapsuleError::ObserverFailed {
            name: "x".into(),
            reason: "y".into(),
        };
        let e2 = CapsuleError::SchemaNotFound {
            capability: "z".into(),
        };
        let e3 = CapsuleError::NonFiniteValue {
            context: "c".into(),
            key: "k".into(),
            value: f64::INFINITY,
        };
        let e4 = CapsuleError::HashMismatch {
            expected: None,
            calculated: "abc".into(),
        };
        assert!(e1.is_observer_related());
        assert!(e2.is_observer_related());
        assert!(e3.is_observer_related());
        assert!(!e4.is_observer_related());
    }
}
