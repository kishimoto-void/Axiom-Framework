# AXIOM COMMON PROTOCOL v1.0.3

## Golden Test Vectors

This directory contains the official interoperability test vectors for the **AXIOM Common Protocol**.

Every conforming implementation **MUST** produce identical canonical serialization and identical cryptographic identifiers for these vectors.

---

## Directory Structure

```
tests/
└── vectors/
    ├── README.md
    ├── minimal.json
    ├── genesis.json
    ├── transition.json
    ├── merge.json
    ├── extension.json
    ├── proof.json
    ├── replay.json
    ├── invalid_cycle.json
    ├── invalid_duplicate_transition.json
    ├── invalid_vendor_namespace.json
    └── expected/
        ├── minimal.expected.json
        ├── genesis.expected.json
        ├── transition.expected.json
        ├── merge.expected.json
        ├── extension.expected.json
        ├── proof.expected.json
        └── replay.expected.json
```

---

## Vector Categories

### 1. minimal.json

**Purpose**  
Smallest valid AXIOM Frame.

**Verifies**
- Canonical JSON
- Core Hash
- Coordinate ID

**Expected**  
`PASS`

---

### 2. genesis.json

**Purpose**  
Validate Genesis anchoring.

**Verifies**
- Genesis Hash
- Initial State Hash
- Genesis consistency

**Expected**  
`PASS`

---

### 3. transition.json

**Purpose**  
Single transition.

**Verifies**
- Transition serialization
- Lamport sequence
- State linkage

**Expected**  
`PASS`

---

### 4. merge.json

**Purpose**  
Multi-parent DAG merge.

**Verifies**
- `before_states`
- Parent references
- DAG integrity
- Leaf verification

**Expected**  
`PASS`

---

### 5. extension.json

**Purpose**  
Vendor extension.

**Verifies**
- Namespace validation
- Extension isolation
- Canonical serialization

**Expected**  
`PASS`

---

### 6. proof.json

**Purpose**  
Proof envelope.

**Verifies**
- Coordinate target
- Core target
- Proof size limit

**Expected**  
`PASS`

---

### 7. replay.json

**Purpose**  
Replay protection.

**Verifies**
- Lamport ordering
- Sequence monotonicity

**Expected**  
`PASS`

---

## Negative Test Vectors

### invalid_cycle.json

**Expected**  
`FAIL`  
`AxiomCausalError`

**Reason**  
Cycle detected.

---

### invalid_duplicate_transition.json

**Expected**  
`FAIL`  
Duplicate Transition ID

---

### invalid_vendor_namespace.json

**Expected**  
`FAIL`  
Invalid Vendor Namespace

---

## Expected Result Format

Each vector **SHALL** have a corresponding expected result.

Example:

```json
{
  "core_hash": "...",
  "genesis_hash": "...",
  "coordinate_id": "...",
  "canonical_json_sha256": "...",
  "validation": "PASS"
}
```

---

## Conformance Requirements

An implementation is **conformant** if every vector produces:

- identical canonical JSON
- identical Core Hash
- identical Genesis Hash
- identical Coordinate ID
- identical validation result

Independent implementations **MUST NOT** compare against Python object behavior.

Only **serialized protocol output** is normative.

---

## Supported Languages

Reference vectors are intended for:

- Python
- Rust
- Go
- Java
- C#
- TypeScript
- C++
- Swift

Any implementation producing identical outputs is considered protocol compliant.

---

## Future Vectors

Future protocol versions may introduce additional vectors covering:

- Multi-signature proofs
- Distributed validator consensus
- Nested extensions
- Large DAG performance
- Streaming frame serialization
- Partial verification
- Incremental synchronization

Existing vectors **MUST** remain stable to preserve backward compatibility.

---

## Recommended Tooling

In addition to the vectors themselves, a conformance runner is recommended:

```
tests/conformance.py
```

This allows implementers to mechanically verify compliance:

```bash
python tests/conformance.py
# or
cargo test
go test
```

With Golden Test Vectors + automated conformance checks, AXIOM becomes a true language-independent protocol rather than a Python-only library.
