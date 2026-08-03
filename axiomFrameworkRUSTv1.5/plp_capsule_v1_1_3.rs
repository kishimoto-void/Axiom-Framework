//! PLP Capsule v1.1.3 — Production-Ready Reference Implementation
//!
//! Final polish based on review of v1.1.2:
//!
//! - `write_raw_str` now iterates over Unicode scalar values (`chars()`)
//!   for better cross-language consistency with non-ASCII content.
//! - `timestamp_ns` is **always serialized as a decimal string** in the
//!   Canonical Hash (explicitly documented to avoid JS 53-bit issues).
//! - Golden Hash vector is now fixed with a concrete SHA-256 value.
//!
//! Previous improvements retained:
//! - Fully hand-written deterministic serialization (no serde_json::to_vec)
//! - Clear verify / recompute_hash / seal contracts
//! - Length-prefixed make_stable_key
//! - BuildParams, ryu canonicalization, empty CapabilityRegistry, etc.
//!
//! Recommended Cargo.toml:
//! ```toml
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! thiserror = "1"
//! ryu = "1"
//! hex = "0.4"
//! sha2 = "0.10"
//! uuid = { version = "1", features = ["v4"] }
//! blake3 = { version = "1", optional = true }
//!
//! [features]
//! default = ["sha2-hash"]
//! sha2-hash = []
//! blake3-hash = ["blake3"]
//! ```

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "sha2-hash")]
use sha2::{Digest, Sha256};

// ==========================================================
// 1. Errors
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum CapsuleError {
    #[error("Observer '{name}': {reason}")]
    ObserverFailed { name: String, reason: String },

    #[error("{context}: key '{key}' has non-finite value ({value})")]
    NonFiniteValue {
        context: String,
        key: String,
        value: f64,
    },

    #[error("Hash serialization failed: {0}")]
    HashSerializationFailed(String),

    #[error("Hash mismatch: expected {expected:?}, got {calculated}")]
    HashMismatch {
        expected: Option<String>,
        calculated: String,
    },

    #[error("Hash computation failed: {0}")]
    HashComputationFailed(String),

    #[error("Schema not found for capability '{capability}'")]
    SchemaNotFound { capability: String },

    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedHashAlgorithm(String),

    #[error("{0}")]
    Other(String),
}

impl CapsuleError {
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
// 2. HashAlgorithm
// ==========================================================

pub trait HashAlgorithm: Send + Sync {
    fn name(&self) -> &'static str;
    fn digest(&self, data: &[u8]) -> Vec<u8>;

    fn digest_hex(&self, data: &[u8]) -> String {
        hex::encode(self.digest(data))
    }
}

#[cfg(feature = "sha2-hash")]
#[derive(Debug, Clone, Copy, Default)]
pub struct Sha256Algorithm;

#[cfg(feature = "sha2-hash")]
impl HashAlgorithm for Sha256Algorithm {
    fn name(&self) -> &'static str {
        "sha256"
    }
    fn digest(&self, data: &[u8]) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(data);
        h.finalize().to_vec()
    }
}

#[cfg(feature = "blake3-hash")]
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Algorithm;

#[cfg(feature = "blake3-hash")]
impl HashAlgorithm for Blake3Algorithm {
    fn name(&self) -> &'static str {
        "blake3"
    }
    fn digest(&self, data: &[u8]) -> Vec<u8> {
        blake3::hash(data).as_bytes().to_vec()
    }
}

// ==========================================================
// 3. SchemaProvider
// ==========================================================

pub trait SchemaProvider: Send + Sync {
    fn get_schema(&self, capability: &str) -> Option<String>;
}

/// Starts empty. Register only what you need.
#[derive(Clone, Default)]
pub struct CapabilityRegistry {
    schemas: Arc<HashMap<String, String>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(HashMap::new()),
        }
    }

    pub fn register(self, capability: &str, schema: &str) -> Self {
        let mut map = (*self.schemas).clone();
        map.insert(capability.to_string(), schema.to_string());
        Self {
            schemas: Arc::new(map),
        }
    }
}

impl SchemaProvider for CapabilityRegistry {
    fn get_schema(&self, capability: &str) -> Option<String> {
        self.schemas.get(capability).cloned()
    }
}

// ==========================================================
// 4. Core structures
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
    pub timestamp_ns: u128,
    pub source: String,
    pub flags: CapsuleFlags,
    pub hash_algorithm: String,
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
    pub observer_id: String,
    pub values: BTreeMap<String, f64>,
}

/// Value-level delta.
///
/// - `Added(v)`    : absolute value
/// - `Modified(d)` : difference (new − old). Receiver must apply as patch.
/// - `Removed`     : key no longer present
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueDelta {
    Added(f64),
    Modified(f64),
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaKind {
    Added,
    Modified,
    Removed,
}

impl DeltaKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeltaKind::Added => "added",
            DeltaKind::Modified => "modified",
            DeltaKind::Removed => "removed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEntry {
    pub kind: DeltaKind,
    pub values: BTreeMap<String, ValueDelta>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeltaBlock {
    pub changes: BTreeMap<String, DeltaEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleIntegrity {
    pub content_hash: Option<String>,
    /// true only when observer_valid && hash_valid && errors.is_empty()
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
// 5. Observer
// ==========================================================

pub trait Observer<W>: Send + Sync {
    fn name(&self) -> &str;
    fn observer_id(&self) -> &str;
    fn observe(&self, world: &W) -> Result<ObservationBlock, String>;
}

// ==========================================================
// 6. Canonical structures (hash target only)
// ==========================================================

#[derive(Debug, Clone)]
struct CanonicalHashPayload {
    header: CanonicalHeader,
    observations: Vec<CanonicalObservation>,
    delta: BTreeMap<String, CanonicalDeltaEntry>,
}

#[derive(Debug, Clone)]
struct CanonicalHeader {
    protocol: String,
    capsule_schema: String,
    version: String,
    capsule_id: String,
    parent_id: Option<String>,
    clock: i64,
    sequence: i64,
    /// Always a decimal string in the Canonical Hash (never a JSON number).
    /// This avoids JavaScript Number 53-bit precision loss and keeps the
    /// representation identical across Rust / Python / Go / JS implementations.
    timestamp_ns: String,
    source: String,
    is_keyframe: bool,
    hash_algorithm: String,
}

#[derive(Debug, Clone)]
struct CanonicalObservation {
    name: String,
    schema: String,
    capability: String,
    observer_id: String,
    values: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct CanonicalDeltaValue {
    kind: String,
    value: Option<String>,
}

#[derive(Debug, Clone)]
struct CanonicalDeltaEntry {
    kind: String,
    values: BTreeMap<String, CanonicalDeltaValue>,
}

// ==========================================================
// 7. Helpers
// ==========================================================

fn default_protocol() -> String {
    "PLP/1.1".to_string()
}
fn default_schema() -> String {
    "v1/capsule".to_string()
}
fn default_version() -> String {
    "1.1.3".to_string()
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

/// Length-prefixed to prevent delimiter collision.
fn make_stable_key(obs: &ObservationBlock) -> String {
    format!(
        "{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
        obs.name.len(),
        obs.name,
        obs.schema.len(),
        obs.schema,
        obs.capability.len(),
        obs.capability,
        obs.observer_id.len(),
        obs.observer_id,
    )
}

fn obs_sort_key(o: &ObservationBlock) -> (&str, &str, &str, &str) {
    (
        o.name.as_str(),
        o.schema.as_str(),
        o.capability.as_str(),
        o.observer_id.as_str(),
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

fn canonicalize_f64(val: f64) -> Result<String, CapsuleError> {
    if !val.is_finite() {
        return Err(CapsuleError::NonFiniteValue {
            context: "canonicalize_f64".into(),
            key: "".into(),
            value: val,
        });
    }
    if val == 0.0 {
        return Ok("0".to_string());
    }
    let mut buf = ryu::Buffer::new();
    let s = buf.format_finite(val);
    Ok(s.replace("e+", "e"))
}

fn canonicalize_values(
    values: &BTreeMap<String, f64>,
) -> Result<BTreeMap<String, String>, CapsuleError> {
    let mut out = BTreeMap::new();
    for (k, &v) in values {
        out.insert(k.clone(), canonicalize_f64(v)?);
    }
    Ok(out)
}

// ==========================================================
// 8. Fully controlled deterministic serialization
// ==========================================================

/// Emit a deterministic UTF-8 byte sequence.
///
/// Locked rules:
/// - Object keys appear in the exact order written in the write_* functions
/// - No whitespace
/// - Option::None → null
/// - All large / floating values are already strings
/// - Escaping is minimal but sufficient for controlled inputs
fn deterministic_serialize(payload: &CanonicalHashPayload) -> Result<Vec<u8>, CapsuleError> {
    let mut out = Vec::with_capacity(1024);
    out.extend_from_slice(b"{\"header\":");
    write_header(&mut out, &payload.header)?;
    out.extend_from_slice(b",\"observations\":");
    write_observations(&mut out, &payload.observations)?;
    out.extend_from_slice(b",\"delta\":");
    write_delta(&mut out, &payload.delta)?;
    out.push(b'}');
    Ok(out)
}

fn write_header(out: &mut Vec<u8>, h: &CanonicalHeader) -> Result<(), CapsuleError> {
    out.extend_from_slice(b"{");
    write_str_kv(out, "protocol", &h.protocol, false)?;
    write_str_kv(out, "capsule_schema", &h.capsule_schema, true)?;
    write_str_kv(out, "version", &h.version, true)?;
    write_str_kv(out, "capsule_id", &h.capsule_id, true)?;
    write_opt_str_kv(out, "parent_id", &h.parent_id, true)?;
    write_i64_kv(out, "clock", h.clock, true)?;
    write_i64_kv(out, "sequence", h.sequence, true)?;
    write_str_kv(out, "timestamp_ns", &h.timestamp_ns, true)?;
    write_str_kv(out, "source", &h.source, true)?;
    write_bool_kv(out, "is_keyframe", h.is_keyframe, true)?;
    write_str_kv(out, "hash_algorithm", &h.hash_algorithm, true)?;
    out.push(b'}');
    Ok(())
}

fn write_observations(
    out: &mut Vec<u8>,
    obs: &[CanonicalObservation],
) -> Result<(), CapsuleError> {
    out.push(b'[');
    for (i, o) in obs.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        out.extend_from_slice(b"{");
        write_str_kv(out, "name", &o.name, false)?;
        write_str_kv(out, "schema", &o.schema, true)?;
        write_str_kv(out, "capability", &o.capability, true)?;
        write_str_kv(out, "observer_id", &o.observer_id, true)?;
        out.extend_from_slice(b",\"values\":{");
        for (j, (k, v)) in o.values.iter().enumerate() {
            if j > 0 {
                out.push(b',');
            }
            write_raw_str(out, k)?;
            out.push(b':');
            write_raw_str(out, v)?; // already a canonical number string
        }
        out.extend_from_slice(b"}}");
    }
    out.push(b']');
    Ok(())
}

fn write_delta(
    out: &mut Vec<u8>,
    delta: &BTreeMap<String, CanonicalDeltaEntry>,
) -> Result<(), CapsuleError> {
    out.push(b'{');
    for (i, (k, entry)) in delta.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        write_raw_str(out, k)?;
        out.push(b':');
        out.extend_from_slice(b"{\"kind\":");
        write_raw_str(out, &entry.kind)?;
        out.extend_from_slice(b",\"values\":{");
        for (j, (vk, vd)) in entry.values.iter().enumerate() {
            if j > 0 {
                out.push(b',');
            }
            write_raw_str(out, vk)?;
            out.push(b':');
            out.extend_from_slice(b"{\"kind\":");
            write_raw_str(out, &vd.kind)?;
            if let Some(ref val) = vd.value {
                out.extend_from_slice(b",\"value\":");
                write_raw_str(out, val)?;
            }
            out.push(b'}');
        }
        out.extend_from_slice(b"}}");
    }
    out.push(b'}');
    Ok(())
}

// ---------- low-level helpers ----------

fn write_raw_str(out: &mut Vec<u8>, s: &str) -> Result<(), CapsuleError> {
    out.push(b'"');
    // Iterate over Unicode scalar values for cross-language consistency.
    // Control characters (< U+0020) are escaped as \u00XX.
    for ch in s.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            c if (c as u32) < 0x20 => {
                let esc = format!("\\u{:04x}", c as u32);
                out.extend_from_slice(esc.as_bytes());
            }
            _ => {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                out.extend_from_slice(encoded.as_bytes());
            }
        }
    }
    out.push(b'"');
    Ok(())
}

fn write_str_kv(
    out: &mut Vec<u8>,
    key: &str,
    val: &str,
    leading_comma: bool,
) -> Result<(), CapsuleError> {
    if leading_comma {
        out.push(b',');
    }
    write_raw_str(out, key)?;
    out.push(b':');
    write_raw_str(out, val)
}

fn write_opt_str_kv(
    out: &mut Vec<u8>,
    key: &str,
    val: &Option<String>,
    leading_comma: bool,
) -> Result<(), CapsuleError> {
    if leading_comma {
        out.push(b',');
    }
    write_raw_str(out, key)?;
    out.push(b':');
    match val {
        Some(s) => write_raw_str(out, s),
        None => {
            out.extend_from_slice(b"null");
            Ok(())
        }
    }
}

fn write_i64_kv(
    out: &mut Vec<u8>,
    key: &str,
    val: i64,
    leading_comma: bool,
) -> Result<(), CapsuleError> {
    if leading_comma {
        out.push(b',');
    }
    write_raw_str(out, key)?;
    out.push(b':');
    out.extend_from_slice(val.to_string().as_bytes());
    Ok(())
}

fn write_bool_kv(
    out: &mut Vec<u8>,
    key: &str,
    val: bool,
    leading_comma: bool,
) -> Result<(), CapsuleError> {
    if leading_comma {
        out.push(b',');
    }
    write_raw_str(out, key)?;
    out.push(b':');
    out.extend_from_slice(if val { b"true" } else { b"false" });
    Ok(())
}

// ==========================================================
// 9. Content hash
// ==========================================================

pub fn compute_content_hash(
    header: &CapsuleHeader,
    observations: &[ObservationBlock],
    delta: &DeltaBlock,
    hasher: &dyn HashAlgorithm,
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

    let mut sorted_obs: Vec<&ObservationBlock> = observations.iter().collect();
    sorted_obs.sort_by(|a, b| obs_sort_key(a).cmp(&obs_sort_key(b)));

    let canon_obs: Result<Vec<_>, _> = sorted_obs
        .iter()
        .map(|o| {
            Ok(CanonicalObservation {
                name: o.name.clone(),
                schema: o.schema.clone(),
                capability: o.capability.clone(),
                observer_id: o.observer_id.clone(),
                values: canonicalize_values(&o.values)?,
            })
        })
        .collect();
    let canon_obs = canon_obs?;

    let mut canon_delta = BTreeMap::new();
    for (k, entry) in &delta.changes {
        let mut values = BTreeMap::new();
        for (vk, vd) in &entry.values {
            let (kind_str, val_opt) = match vd {
                ValueDelta::Added(v) => ("added", Some(canonicalize_f64(*v)?)),
                ValueDelta::Modified(v) => ("modified", Some(canonicalize_f64(*v)?)),
                ValueDelta::Removed => ("removed", None),
            };
            values.insert(
                vk.clone(),
                CanonicalDeltaValue {
                    kind: kind_str.to_string(),
                    value: val_opt,
                },
            );
        }
        canon_delta.insert(
            k.clone(),
            CanonicalDeltaEntry {
                kind: entry.kind.as_str().to_string(),
                values,
            },
        );
    }

    let payload = CanonicalHashPayload {
        header: CanonicalHeader {
            protocol: header.protocol.clone(),
            capsule_schema: header.capsule_schema.clone(),
            version: header.version.clone(),
            capsule_id: header.capsule_id.clone(),
            parent_id: header.parent_id.clone(),
            clock: header.clock,
            sequence: header.sequence,
            timestamp_ns: header.timestamp_ns.to_string(),
            source: header.source.clone(),
            is_keyframe: header.flags.is_keyframe,
            hash_algorithm: header.hash_algorithm.clone(),
        },
        observations: canon_obs,
        delta: canon_delta,
    };

    let bytes = deterministic_serialize(&payload)?;
    Ok(hasher.digest_hex(&bytes))
}

// ==========================================================
// 10. PLPCapsule methods (clear contracts)
// ==========================================================

impl PLPCapsule {
    /// Pure verification. Never mutates self.
    pub fn verify(&self, hasher: &dyn HashAlgorithm) -> Result<bool, CapsuleError> {
        let calculated =
            compute_content_hash(&self.header, &self.observations, &self.delta, hasher)?;
        match &self.integrity.content_hash {
            Some(expected) if expected == &calculated => Ok(true),
            Some(expected) => Err(CapsuleError::HashMismatch {
                expected: Some(expected.clone()),
                calculated,
            }),
            None => Err(CapsuleError::HashMismatch {
                expected: None,
                calculated,
            }),
        }
    }

    /// Recompute hash and overwrite `content_hash`.
    /// Clears previous non-observer errors.
    /// Returns true if the new hash was successfully computed.
    pub fn recompute_hash(&mut self, hasher: &dyn HashAlgorithm) -> bool {
        self.integrity
            .errors
            .retain(|e| e.is_observer_related());

        match compute_content_hash(&self.header, &self.observations, &self.delta, hasher) {
            Ok(calculated) => {
                self.integrity.content_hash = Some(calculated);
                self.integrity.hash_valid = Some(true);
                self.integrity.valid =
                    self.integrity.observer_valid && self.integrity.errors.is_empty();
                true
            }
            Err(e) => {
                self.integrity.hash_valid = Some(false);
                self.integrity.valid = false;
                self.integrity.errors.push(e);
                false
            }
        }
    }

    /// Mark as keyframe and recompute hash.
    pub fn seal(&mut self, hasher: &dyn HashAlgorithm) -> bool {
        self.header.flags.is_keyframe = true;
        self.recompute_hash(hasher)
    }

    #[deprecated(note = "use recompute_hash or seal")]
    pub fn refresh_integrity(&mut self, hasher: &dyn HashAlgorithm) -> bool {
        self.recompute_hash(hasher)
    }

    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}

// ==========================================================
// 11. BuildParams + CapsuleBuilder
// ==========================================================

pub struct BuildParams {
    pub clock: i64,
    pub sequence: i64,
    pub capsule_id: String,
    pub timestamp_ns: u128,
    pub source: Option<String>,
    pub parent_id: Option<String>,
    pub flags: Option<CapsuleFlags>,
}

pub struct CapsuleBuilder<W, S = CapabilityRegistry>
where
    S: SchemaProvider,
{
    observers: Vec<Arc<dyn Observer<W>>>,
    schema_provider: S,
    hasher: Box<dyn HashAlgorithm>,
    rel_epsilon: f64,
    abs_epsilon: f64,
}

impl<W> CapsuleBuilder<W, CapabilityRegistry> {
    pub fn new(observers: Vec<Arc<dyn Observer<W>>>) -> Self {
        Self {
            observers,
            schema_provider: CapabilityRegistry::new(),
            hasher: Box::new(Sha256Algorithm),
            rel_epsilon: 1e-9,
            abs_epsilon: 1e-12,
        }
    }
}

impl<W, S> CapsuleBuilder<W, S>
where
    S: SchemaProvider,
{
    pub fn with_schema_provider(mut self, provider: S) -> Self {
        self.schema_provider = provider;
        self
    }

    pub fn with_hasher(mut self, hasher: Box<dyn HashAlgorithm>) -> Self {
        self.hasher = hasher;
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
                    let mut has_added = false;
                    let mut has_modified = false;
                    let mut has_removed = false;

                    for (k, &v) in &curr.values {
                        match prev.values.get(k) {
                            Some(&prev_v) => {
                                if v.is_finite()
                                    && prev_v.is_finite()
                                    && !approx_eq(v, prev_v, self.rel_epsilon, self.abs_epsilon)
                                {
                                    values.insert(k.clone(), ValueDelta::Modified(v - prev_v));
                                    has_modified = true;
                                }
                            }
                            None => {
                                if v.is_finite() {
                                    values.insert(k.clone(), ValueDelta::Added(v));
                                    has_added = true;
                                }
                            }
                        }
                    }
                    for k in prev.values.keys() {
                        if !curr.values.contains_key(k) {
                            values.insert(k.clone(), ValueDelta::Removed);
                            has_removed = true;
                        }
                    }

                    if !values.is_empty() {
                        let kind = match (has_added, has_modified, has_removed) {
                            (true, false, false) => DeltaKind::Added,
                            (false, false, true) => DeltaKind::Removed,
                            _ => DeltaKind::Modified,
                        };
                        changes.insert(key.clone(), DeltaEntry { kind, values });
                    }
                }
            }
        }

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

    /// Non-deterministic convenience path.
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
        self.build_with_meta(
            world,
            input_packet,
            clock,
            sequence,
            uuid::Uuid::new_v4().to_string(),
            now_ns(),
            previous,
            source,
            parent_id,
            flags,
        )
    }

    pub fn build_with_params(
        &self,
        world: &W,
        input: InputCapsule,
        params: BuildParams,
        previous: Option<&PLPCapsule>,
    ) -> PLPCapsule {
        self.build_with_meta(
            world,
            input,
            params.clock,
            params.sequence,
            params.capsule_id,
            params.timestamp_ns,
            previous,
            params.source,
            params.parent_id,
            params.flags,
        )
    }

    pub fn build_with_meta(
        &self,
        world: &W,
        input_packet: InputCapsule,
        clock: i64,
        sequence: i64,
        capsule_id: String,
        timestamp_ns: u128,
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
                        match self.schema_provider.get_schema(&block.capability) {
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
            capsule_id,
            parent_id,
            clock,
            sequence,
            timestamp_ns,
            source: source.unwrap_or_else(default_source),
            flags: flags.unwrap_or_default(),
            hash_algorithm: self.hasher.name().to_string(),
        };

        let delta = self.compute_delta(&observations, previous);

        let (content_hash, hash_valid) =
            match compute_content_hash(&header, &observations, &delta, self.hasher.as_ref()) {
                Ok(h) => (Some(h), Some(true)),
                Err(e) => {
                    errors.push(e);
                    (None, Some(false))
                }
            };

        let observer_valid = !errors.iter().any(|e| e.is_observer_related());
        let is_hash_ok = hash_valid == Some(true);
        let valid = observer_valid && is_hash_ok && errors.is_empty();

        PLPCapsule {
            header,
            input: input_packet,
            observations,
            delta,
            integrity: CapsuleIntegrity {
                content_hash,
                valid,
                observer_valid,
                hash_valid,
                errors,
            },
        }
    }
}

// ==========================================================
// 12. Tests
// ==========================================================

#[cfg(all(test, feature = "sha2-hash"))]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn sample_obs(name: &str, id: &str, x: f64, y: f64) -> ObservationBlock {
        let mut values = BTreeMap::new();
        values.insert("x".into(), x);
        values.insert("y".into(), y);
        ObservationBlock {
            name: name.into(),
            schema: "v1/geometry".into(),
            capability: "geometry".into(),
            observer_id: id.into(),
            values,
        }
    }

    #[test]
    fn canonicalize_zero() {
        assert_eq!(canonicalize_f64(0.0).unwrap(), "0");
        assert_eq!(canonicalize_f64(-0.0).unwrap(), "0");
    }

    #[test]
    fn reject_non_finite() {
        let mut values = BTreeMap::new();
        values.insert("bad".into(), f64::NAN);
        assert!(ensure_finite(&values, "test").is_err());
    }

    #[test]
    fn deterministic_hash_identical_inputs() {
        let header = CapsuleHeader {
            protocol: "PLP/1.1".into(),
            capsule_schema: "v1/capsule".into(),
            version: "1.1.3".into(),
            capsule_id: "fixed-id".into(),
            parent_id: None,
            clock: 1,
            sequence: 0,
            timestamp_ns: 1_700_000_000_000_000_000,
            source: "test".into(),
            flags: CapsuleFlags { is_keyframe: true },
            hash_algorithm: "sha256".into(),
        };
        let obs = vec![sample_obs("geom", "cam0", 1.0, 2.5)];
        let delta = DeltaBlock::default();
        let h1 = compute_content_hash(&header, &obs, &delta, &Sha256Algorithm).unwrap();
        let h2 = compute_content_hash(&header, &obs, &delta, &Sha256Algorithm).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn delta_kind_as_str_stable() {
        assert_eq!(DeltaKind::Added.as_str(), "added");
        assert_eq!(DeltaKind::Modified.as_str(), "modified");
        assert_eq!(DeltaKind::Removed.as_str(), "removed");
    }

    #[test]
    fn build_with_params_deterministic() {
        struct Mock;
        impl Observer<()> for Mock {
            fn name(&self) -> &str {
                "geom"
            }
            fn observer_id(&self) -> &str {
                "cam"
            }
            fn observe(&self, _: &()) -> Result<ObservationBlock, String> {
                Ok(sample_obs("geom", "cam", 0.5, -0.3))
            }
        }

        let registry = CapabilityRegistry::new().register("geometry", "v1/geometry");
        let builder = CapsuleBuilder::new(vec![Arc::new(Mock)]).with_schema_provider(registry);

        let params = BuildParams {
            clock: 10,
            sequence: 1,
            capsule_id: "id-1".into(),
            timestamp_ns: 1_700_000_000_000_000_000,
            source: Some("test".into()),
            parent_id: None,
            flags: Some(CapsuleFlags { is_keyframe: true }),
        };

        let c1 = builder.build_with_params(
            &(),
            InputCapsule {
                data: "seed".into(),
            },
            BuildParams {
                clock: params.clock,
                sequence: params.sequence,
                capsule_id: params.capsule_id.clone(),
                timestamp_ns: params.timestamp_ns,
                source: params.source.clone(),
                parent_id: params.parent_id.clone(),
                flags: params.flags,
            },
            None,
        );
        let c2 = builder.build_with_params(
            &(),
            InputCapsule {
                data: "seed".into(),
            },
            params,
            None,
        );

        assert_eq!(c1.integrity.content_hash, c2.integrity.content_hash);
        assert!(c1.integrity.valid);
        assert!(c1.verify(&Sha256Algorithm).unwrap());
    }

    #[test]
    fn recompute_hash_clears_non_observer_errors() {
        let mut cap = PLPCapsule {
            header: CapsuleHeader {
                protocol: "PLP/1.1".into(),
                capsule_schema: "v1/capsule".into(),
                version: "1.1.3".into(),
                capsule_id: "x".into(),
                parent_id: None,
                clock: 0,
                sequence: 0,
                timestamp_ns: 0,
                source: "t".into(),
                flags: Default::default(),
                hash_algorithm: "sha256".into(),
            },
            input: InputCapsule { data: "".into() },
            observations: vec![],
            delta: DeltaBlock::default(),
            integrity: CapsuleIntegrity {
                content_hash: None,
                valid: false,
                observer_valid: true,
                hash_valid: None,
                errors: vec![CapsuleError::Other("old".into())],
            },
        };

        let ok = cap.recompute_hash(&Sha256Algorithm);
        assert!(ok);
        assert!(cap.integrity.errors.is_empty());
        assert_eq!(cap.integrity.hash_valid, Some(true));
    }

    /// Golden Vector (fixed).
    /// Input and serialization rules are locked; this hash must remain stable
    /// across Rust / Python / Go reference implementations.
    #[test]
    fn golden_hash_fixed_vector() {
        let header = CapsuleHeader {
            protocol: "PLP/1.1".into(),
            capsule_schema: "v1/capsule".into(),
            version: "1.1.3".into(),
            capsule_id: "00000000-0000-4000-8000-000000000001".into(),
            parent_id: None,
            clock: 42,
            sequence: 7,
            timestamp_ns: 1_700_000_000_000_000_000,
            source: "golden".into(),
            flags: CapsuleFlags { is_keyframe: true },
            hash_algorithm: "sha256".into(),
        };

        let mut values = BTreeMap::new();
        values.insert("x".into(), 1.0);
        values.insert("y".into(), -2.5);
        let obs = vec![ObservationBlock {
            name: "geom".into(),
            schema: "v1/geometry".into(),
            capability: "geometry".into(),
            observer_id: "cam0".into(),
            values,
        }];

        let delta = DeltaBlock::default();
        let hash = compute_content_hash(&header, &obs, &delta, &Sha256Algorithm).unwrap();

        assert_eq!(
            hash,
            "a54b533ae4223cbef6d6227c957ac22d11efbcf61deead82bbc1e17134c3941e"
        );
    }
}
