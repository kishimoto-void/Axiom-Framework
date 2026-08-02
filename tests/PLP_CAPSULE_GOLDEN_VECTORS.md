# PLP Capsule Golden Test Vectors (Rust reference)

These vectors are produced by the Rust implementation (`plp_capsule_production.rs`) under the exact serialization rules of:

- `BTreeMap` key ordering
- `serde_json::to_vec` (default)
- SHA-256 → lowercase hex

They are intended to be shared across Rust / Python / Go implementations for cross-language determinism verification.

## Fixed Header used in all hash vectors

```json
{
  "protocol": "PLP/1.0",
  "capsule_schema": "v1/capsule",
  "version": "1.0.0",
  "capsule_id": "00000000-0000-4000-8000-000000000001",
  "parent_id": null,
  "clock": 42,
  "sequence": 7,
  "timestamp_ns": 1700000000000000000,
  "source": "golden_test",
  "is_keyframe": true
}
```

## 1. Empty observations + empty delta

**Hash (SHA-256 hex):**
```
74a5d13e37f4355a294a61e79ba36ea293a1007fa2d83a9fb922600ac3eca588
```

## 2. Two observations (geom + thermo) + empty delta

Observations (sorted by name/schema/capability/observer_id):

- `geom` / `v1/geometry` / `geometry` / `camera/front`
  - `radius`: 1.7
  - `x`: 1.0
  - `y`: 2.5

- `thermo` / `v1/thermal` / `thermal` / `sensor/0`
  - `energy`: -0.1234
  - `temp`: 0.0065

**Hash:**
```
4bf640c5cdd6780e9f25f2b074d3624855573be28b04aa0a4ef236837c5641e6
```

## 3. One observation + non-trivial delta

Observation: same as geom above.

Delta entry key: `geom.v1/geometry.geometry.camera/front`

```json
{
  "kind": "Modified",
  "values": {
    "old_key": "Removed",
    "x": { "Modified": 0.1 },
    "z": { "Added": 3.14 }
  }
}
```

**Hash:**
```
1674386e2d69a90a9fbfc3d5ec3532f3fb64e9c00837b0bf681810f18dc69f8a
```

## Delta semantics (ValueDelta enum)

| Variant     | Meaning                          |
|-------------|----------------------------------|
| `Added(v)`  | Key newly appeared with value v  |
| `Modified(d)` | Value changed by delta d (= new − old) |
| `Removed`   | Key disappeared                  |

Stable observation key format:  
`{name}.{schema}.{capability}.{observer_id}`

## Test coverage (10 tests, all green)

- `golden_content_hash_empty`
- `golden_content_hash_with_observations`
- `golden_content_hash_with_delta`
- `golden_delta_first_capsule_all_added`
- `golden_delta_value_modified`
- `golden_delta_key_removed_and_added`
- `golden_delta_observation_removed`
- `golden_full_build_and_verify`
- `golden_reject_non_finite`
- `golden_observer_related_flag`

## Notes for other language ports

1. Serialize the HashPayload with **sorted keys** (BTreeMap equivalent).
2. Use the exact same field names and enum variant names as Rust serde.
3. Do **not** include `integrity` or `input` in the hash payload.
4. Floats must be serialized in a way that matches `serde_json` default (shortest representation that round-trips).
5. Future improvement: switch to RFC 8785 Canonical JSON for true byte-for-byte identity across languages.

Generated: 2026-08-02 (Rust 1.75 / sha2 0.10 / serde_json 1.x)
