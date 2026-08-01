# RFC-AXIOM-0001

# AXIOM Common Protocol (ACP)

**Deterministic State Coordinate & Causal Proof Protocol**

- **Status**: Stable Reference
- **Version**: 1.1.0
- **Date**: 2026-08-01
- **Author**: kishimoto-void
- **License**: Non-Commercial / Non-Military Use Only

---

## Abstract

The AXIOM Common Protocol (ACP) defines a language-neutral, deterministic mechanism for identifying immutable states, representing causal transitions as a Directed Acyclic Graph (DAG), and attaching cryptographic proofs.

ACP is intentionally free of reasoning logic and AI model assumptions. Its sole responsibility is to provide a common coordinate system and integrity layer for state exchange across heterogeneous implementations.

Any system that can produce and consume canonical JSON can implement ACP.

### Conventions and Terminology

The key words "**MUST**", "**MUST NOT**", "**REQUIRED**", "**SHALL**", "**SHALL NOT**", "**SHOULD**", "**SHOULD NOT**", "**RECOMMENDED**", "**MAY**", and "**OPTIONAL**" in this document are to be interpreted as described in [RFC 2119] and [RFC 8174] when, and only when, they appear in all capitals, as shown here.

---

## 1. Introduction

### 1.1 Motivation

Distributed AI systems, multi-agent workflows, and cross-model pipelines require a shared way to answer:

- What is the current state?
- How did we arrive here?
- Who authorized this transition?
- Can this history be replayed and verified independently?

Existing approaches either embed these concerns inside proprietary frameworks or leave them undefined. ACP extracts them into a minimal, immutable protocol layer.

### 1.2 Design Goals

1. **Determinism** — Identical inputs MUST produce identical hashes and Coordinate IDs across all conforming implementations.
2. **Language Neutrality** — No dependency on any particular programming language or runtime.
3. **Intelligence Neutrality** — No assumption about reasoning engines or model architectures.
4. **Immutability of Core** — Protocol state is append-only and content-addressed.
5. **Minimal Surface** — The protocol defines only what is necessary for identification, causality, and proof.
6. **Extensibility** — Vendor-specific metadata is permitted without polluting the core namespace.
7. **Algorithm Agility** — Hash and signature algorithms can evolve without breaking the coordinate model.

### 1.3 Non-Goals

ACP does **not** define:

- How reasoning is performed
- How models are invoked
- How mutable runtime data is stored (see Capsule)
- How physical or semantic state is represented (see PLP or other profiles)

---

## 2. Glossary

| Term | Definition |
|------|------------|
| **ACP** | AXIOM Common Protocol — the State Integrity Layer defined by this document. |
| **Frame** | The top-level protocol object (Header + Genesis + Core + Transitions + Proofs + Extension). |
| **Coordinate ID** | Deterministic digest that uniquely identifies a complete Frame. |
| **Core** | Immutable state payload (Identity, State, Invariant, Constraint). |
| **Core Hash** | Digest of the canonical Core under the active hash algorithm and encoding. |
| **Genesis** | Origin record of a state universe. |
| **Genesis Hash** | Self-hash of the Genesis record. |
| **Transition** | Single causal step from one or more parent states to a new state. |
| **DAG** | Directed Acyclic Graph formed by the parent relations of Transitions. |
| **Proof** | Cryptographic envelope binding a signature to a typed target. |
| **Capsule** | Mutable runtime payload living *outside* ACP. |
| **PLP** | Particle Language Protocol — one possible State Representation profile. ACP does not depend on it. |
| **State Integrity** | The concern of identity, ordering, causality and proof (ACP’s sole responsibility). |
| **State Representation** | The concern of *what the state is* (PLP or any other profile). |

---

## 3. Protocol Layers

```
Applications
         │
  LRP / PSS / DCK
         │
      Capsule          ← Mutable Runtime Payload
         │
  ┌──────┴──────┐
  ▼             ▼
 ACP           PLP     ← Sibling layers
State Integrity   State Representation
  │             │
  └──────┬──────┘
         │
    Runtime / Reality
```

ACP answers: *When, from where, how did the state change, and who proved it?*  
PLP (or any other representation) answers: *What is the state?*

---

## 4. Canonical Serialization

### 4.1 Requirement

All protocol objects MUST be serialized using **RFC 8785 JSON Canonicalization Scheme (JCS)** before hashing (JSON profile).

### 4.2 Rules (Summary)

- Object keys sorted lexicographically by Unicode code point.
- No insignificant whitespace.
- Numbers in shortest accurate decimal representation.
- Strings **MUST** be Unicode NFC normalized; other normalization forms MUST be rejected or converted to NFC before hashing.
- Arrays preserve order.
- **Empty array vs omitted key**: For every field whose schema type is an array, an empty array `[]` and the complete absence of the key are **not** equivalent. Implementations that produce Frames SHOULD emit empty arrays explicitly. Consumers MUST treat a missing array key as a schema violation (ACP-001) unless the field is marked OPTIONAL and the prose explicitly permits omission.

### 4.3 Decimal Representation

All **non-integer** numeric values inside `Core.state` MUST be encoded using the Decimal Representation object:

```json
{ "$type": "decimal", "$value": "1.23" }
```

**Normative rules for `$value`:**

- MUST be a string matching the grammar:
  ```
  decimal   = [ "-" ] 1*DIGIT [ "." 1*DIGIT ]
  ```
- MUST NOT use scientific notation (`1e-5`, `1E3`, etc.).
- MUST NOT contain a leading `+`.
- MUST NOT contain leading zeros (except the single zero required for values with absolute value < 1, e.g. `"0.5"` is legal, `"00.5"` is not).
- MUST NOT contain trailing zeros after the decimal point (`"1.230"` is illegal; `"1.23"` is legal).
- MUST NOT be an integer written with a decimal point (`"1.0"`, `"1."` are illegal); integers MUST be emitted as JSON numbers.
- MUST NOT be the strings `""`, `"."`, `"-"`, `"+"` or any non-numeric content.
- Negative zero (`"-0"`, `"-0.0"`) is illegal; zero MUST be the integer `0`.

Floating-point literals written directly as JSON numbers inside `Core.state` MUST be rejected (ACP-001 or implementation-defined numeric error).  
Integer values MAY be written as JSON numbers; if an integer exceeds the safe range of IEEE-754 binary64 (≈ 2^53), implementations SHOULD encode it as a Decimal object instead.

### 4.4 Hash Algorithms

| Algorithm | Status | Notes |
|-----------|--------|-------|
| `sha256` | **Required** | Default and mandatory for v1.x |
| `sha3-256` | Optional | Recommended for new implementations |
| `blake3` | Optional | High-performance alternative |

Implementations MUST reject unknown `hash_algorithm` values unless they explicitly support them.

### 4.5 Collected ABNF

```
; Timestamp (UTC only)
date-fullyear  = 4DIGIT
date-month     = 2DIGIT
date-mday      = 2DIGIT
time-hour      = 2DIGIT
time-minute    = 2DIGIT
time-second    = 2DIGIT
time-secfrac   = "." 3DIGIT
timestamp      = date-fullyear "-" date-month "-" date-mday
                 "T" time-hour ":" time-minute ":" time-second
                 [ time-secfrac ] "Z"

; Vendor namespace under $ext
vendor-label   = 1*( DIGIT / %x61-7A / "-" )   ; [a-z0-9-]+
vendor-ns      = vendor-label 1*( "." vendor-label )

; Proof algorithm identifier
alg-token      = %x61-7A *(%x61-7A / DIGIT / "-" )  ; [a-z][a-z0-9-]*

; Coordinate URI (informational)
coord-uri      = "axiom://frame/"  64HEXDIG
               / "axiom://core/"   64HEXDIG
               / "axiom://transition/" 1*VCHAR
               / "axiom://genesis/" 1*VCHAR
```

---

## 5. Domain Separation

Every hash computation is prefixed with a domain separation tag that includes algorithm **and** encoding:

```
AXIOM:<DOMAIN>:v1:<algorithm>:<encoding>:
```

| Domain | Example Tag (JSON / SHA-256) |
|--------|------------------------------|
| STATE | `AXIOM:STATE:v1:sha256:jcs:` |
| GENESIS | `AXIOM:GENESIS:v1:sha256:jcs:` |
| TRANSITION | `AXIOM:TRANSITION:v1:sha256:jcs:` |
| PROOF | `AXIOM:PROOF:v1:sha256:jcs:` |
| FRAME | `AXIOM:FRAME:v1:sha256:jcs:` |

When a future CBOR profile is defined, the encoding component becomes `cbor`, preventing cross-encoding collisions.

---

## 6. Data Structures

### 6.1 Header

```json
{
  "protocol": "AXIOM",
  "protocol_id": "acp",
  "version": "1.1.0",
  "encoding": "rfc8785-jcs",
  "hash_algorithm": "sha256"
}
```

- `protocol` MUST be `"AXIOM"`.
- `protocol_id` MUST be `"acp"` for this specification.  
  (Reserved for future siblings: `"plp"`, `"pss"`, `"lrp"`, …)
- `version` follows Semantic Versioning.
- `encoding` MUST be `"rfc8785-jcs"` for the JSON profile.
- `hash_algorithm` MUST be a supported algorithm (§4.4).

### 6.2 Genesis

```json
{
  "genesis_id": "string",
  "created_by": "string",
  "initial_state_hash": "hex-digest",
  "timestamp": "ISO-8601-UTC",
  "genesis_hash": "hex-digest"
}
```

`genesis_hash` is computed over the canonical Genesis **excluding** the `genesis_hash` field itself, using the GENESIS domain tag.

### 6.3 Core

```json
{
  "identity": {
    "scheme": "urn | uuid | did | url | string",
    "entity": "string",
    "scope": "string",
    "domain": "string",
    "boundary": ["string"]
  },
  "state": {
    "current": { ... },
    "initial": { ... },
    "target": { ... },
    "transition": ["string"]
  },
  "invariant": {
    "must_hold": ["string"],
    "forbidden": ["string"],
    "conservation": ["string"]
  },
  "constraint": {
    "hard": ["string"],
    "soft": ["string"],
    "resource": ["string"],
    "limit": ["string"]
  }
}
```

- `identity.scheme` is OPTIONAL. If absent, the identity is treated as an opaque string tuple.
- Implementations MUST accept and preserve unknown scheme values; they MUST NOT reject a Frame solely because the scheme is unrecognized. This permits future schemes without a protocol revision.
- The **Core Hash** is the digest of the domain-tagged canonical Core.

### 6.4 Transition

```json
{
  "transition_id": "string",
  "sequence_number": integer,
  "before_states": ["hex-digest"],
  "after": "hex-digest",
  "operation": "string",
  "actor": "string",
  "timestamp": "ISO-8601-UTC",
  "parent_transitions": ["string"],
  "reason": "string (optional)"
}
```

#### Causal Rules

1. **Single Root** — Exactly one Transition MUST have an empty `parent_transitions` list.
2. **Acyclicity** — The graph MUST be a DAG.
3. **Lamport Ordering** —  
   `T.sequence_number ≥ max(parent.sequence_number) + 1`  
   (no parents → `sequence_number ≥ 1`).
4. **Uniqueness** — All `transition_id` values within a Frame MUST be unique.
5. **Leaf Consistency** (normative)

   A Transition is a **leaf** if and only if no other Transition in the same Frame lists its `transition_id` in that Transition’s `parent_transitions` array.

   - If the `transitions` array is **non-empty**:
     The Core Hash (STATE domain tag) MUST equal the `after` value of **at least one** leaf Transition.
     (Multiple leaves with different `after` values are permitted; this supports legitimate multi-parent merges. It is NOT required that every leaf share the same `after`.)
   - If the `transitions` array is **empty**:
     Let `H` be the digest obtained by applying the STATE domain tag to the canonical serialization of `Core.state.initial`.
     `H` MUST equal `Genesis.initial_state_hash`.

   Frames that violate this rule MUST be rejected with **ACP-105**.
   Core Hash (STATE domain) and Genesis fields are never compared as raw strings; comparison is always performed on values that share the same domain tag.

### 6.5 Proof

```json
{
  "algorithm": "ed25519",
  "signer": "string",
  "signature": "string",
  "target_type": "frame | core | transition | genesis",
  "target_hash": "hex-digest"
}
```

`algorithm` MUST be a lowercase ASCII token matching `^[a-z][a-z0-9-]*$` (e.g. `ed25519`, `ecdsa-p256`).  
Upper-case or mixed-case identifiers (`Ed25519`, `ED25519`) MUST be rejected.

**Normative values** (MUST be accepted):

- `frame`
- `core`
- `transition`
- `genesis`

**Reserved values** (MUST NOT be used by applications in v1.x):

- `extension`
- `capsule`

Any `target_type` that is not one of the four normative values (including the reserved values and any unknown string) MUST cause the implementation to reject the Frame with error **ACP-202**.

Maximum proof size MUST be enforced (recommended default: 64 KiB).

#### Signature Construction (Normative)

The byte sequence that is signed (and later verified) SHALL be constructed as follows:

```
SignPayload = UTF-8(
    "AXIOM:PROOF:v1:" +
    hash_algorithm + ":" +
    encoding + ":" +
    target_type + ":" +
    target_hash
)
```

where:

- `hash_algorithm` is the value from the Frame Header,
- `encoding` is the encoding identifier (`jcs` for the JSON profile),
- `target_type` and `target_hash` are taken from the Proof object.

No other bytes (including the rest of the Proof object or the Frame) participate in the signature payload.  
This definition is mandatory for Conformance Level L3.

#### Target Hash Binding (Normative)

In addition to verifying the signature over `SignPayload`, a Level-L3 implementation MUST verify that `target_hash` actually equals the digest of the referenced object:

| target_type | Required match |
|-------------|----------------|
| `frame` | Coordinate ID of the Frame |
| `core` | Core Hash |
| `transition` | digest of the referenced Transition (TRANSITION domain) |
| `genesis` | Genesis Hash |

Failure of this binding check MUST produce **ACP-203**.

#### Size Limits (Normative)

| Limit | Value | Error |
|-------|-------|-------|
| Maximum size of a single Proof object (canonical form) | 64 KiB | ACP-201 |
| Maximum number of Proofs in a Frame | 32 | ACP-201 |
| Maximum length of `signature` string | 16 KiB | ACP-201 |
| Maximum length of any single string field in a Proof | 4 KiB | ACP-201 |

### 6.6 Extension

```json
{
  "geometry": { ... },
  "$ext": {
    "vendor.example": { ... }
  },
  "$sys": { }
}
```

- Vendor keys under `$ext` MUST match: `^[a-z0-9-]+(\.[a-z0-9-]+)+$`
- Keys under `$ext` MUST be unique within the object.
- `$sys` is reserved for future protocol-level extensions. Implementations MUST NOT write application data under `$sys`; detection MUST produce **ACP-005**.

### 6.7 Frame & Coordinate ID

A Frame is the aggregation of Header + Genesis + Core + optional Transitions + optional Proofs + optional Extension.

`Header`, `Genesis` and `Core` are **mandatory**. A Frame missing any of them MUST be rejected with ACP-001.

**Coordinate ID computation (normative):**

1. Start with the complete Frame object.
2. Recursively remove every key named `coordinate_id` from every object at any depth (including inside `$ext`, nested state objects, etc.).
3. If removal leaves an empty object where a non-empty object was required by the schema, the Frame is malformed and MUST be rejected (ACP-001).
4. Canonicalize the resulting object with JCS.
5. Prefix the canonical bytes with the FRAME domain tag.
6. Compute the digest with the algorithm declared in the Header.

The field `coordinate_id` never participates in its own hash.  
Implementations that materialise the Coordinate ID inside a stored Frame MUST strip it before steps 4–6.

A concrete calculation example MUST appear in the Golden Test Vectors for any Frame that embeds its own Coordinate ID.

---

## 7. Timestamp Policy

All timestamps MUST:

1. Be expressed in **UTC** with the `Z` suffix (no local offsets).
2. Follow one of the two strictly permitted formats:
   - Without fractional seconds: `YYYY-MM-DDThh:mm:ssZ`
   - With millisecond precision: `YYYY-MM-DDThh:mm:ss.sssZ` (exactly three digits)
3. MUST NOT emit other fractional-second lengths (e.g. `.0`, `.00`, `.1234`).
4. On **input**, an implementation MUST normalise any accepted timestamp into one of the two forms above before canonicalisation or hashing. Numeric (epoch) comparison is forbidden; only the normalised string is significant.
5. Leap-second handling is left to the platform; protocol comparison is performed on the canonical string after normalisation.

Rationale: differing JSON libraries emit `2026-08-01T07:00:00Z` versus `2026-08-01T07:00:00.000Z`. Because JCS requires exact string equality, any variation breaks Coordinate ID determinism.

---

## 8. Coordinate URI

URI scheme:

```
axiom://frame/<coordinate-id>          (canonical)
axiom://core/<core-hash>
axiom://transition/<transition-id>
axiom://genesis/<genesis-id>
axiom://state/<coordinate-id>          (deprecated alias of frame)
```

`axiom://frame/` is the canonical form.  
`axiom://state/` is retained as a deprecated alias for compatibility; new implementations SHOULD emit `axiom://frame/` only.  
These URIs are informational; the protocol operates on raw digests.

---

## 9. Media Types

| Media Type | Usage |
|------------|-------|
| `application/axiom+json` | Generic ACP document |
| `application/axiom-frame+json` | Complete Frame |
| `application/axiom-core+json` | Core only |

Formal IANA registration is deferred; implementations SHOULD already use these strings.

---

## 10. Validation Order

A conforming implementation SHOULD perform validation in the following order.  
Early failure is permitted; the order reduces implementation divergence.

1. **Deserialize** — Parse JSON, reject malformed documents (ACP-001).
2. **Header** — Check protocol, protocol_id, version, encoding, hash_algorithm (ACP-002, ACP-003).
3. **Canonicalize** — Produce JCS form of each component.
4. **Genesis** — Verify self-hash and timestamp policy.
5. **Core** — Compute and record Core Hash.
6. **DAG / Transitions** —  
   - uniqueness of transition_id (ACP-102)  
   - single root (ACP-103)  
   - acyclicity (ACP-101)  
   - Lamport ordering (ACP-104)  
   - leaf consistency (ACP-105)
7. **Proofs** — size limits, target_type, signature verification (ACP-201–203).
8. **Coordinate ID** — compute final Frame digest and compare if a stored value is present.
9. **Extension** — vendor namespace regex, $sys reservation (ACP-004, ACP-005).

---

## 11. Error Codes

| Code | Condition | Class |
|------|-----------|-------|
| ACP-001 | Missing or malformed Header / JSON | Deserialization |
| ACP-002 | Unsupported or unknown version / protocol_id | Deserialization |
| ACP-003 | Unknown hash_algorithm | Deserialization |
| ACP-004 | Invalid vendor namespace under `$ext` | Deserialization |
| ACP-005 | Write attempted under reserved `$sys` | Deserialization |
| ACP-101 | Cycle detected in Transition DAG | Causal |
| ACP-102 | Duplicate `transition_id` | Causal |
| ACP-103 | Root count ≠ 1 | Causal |
| ACP-104 | Lamport sequence violation | Causal |
| ACP-105 | Core Hash does not match any leaf / Genesis | Causal |
| ACP-201 | Proof exceeds size limit | Proof |
| ACP-202 | `target_type` is not one of the four normative values (includes reserved and unknown values) | Proof |
| ACP-203 | Invalid or unverifiable signature / target_hash binding failure | Proof |

---

## 12. Conformance Levels

| Level | Name | Requirements |
|-------|------|--------------|
| **L0** | Read | Parse Frames, extract fields |
| **L1** | Validate | L0 + enforce all causal & structural rules, emit correct error codes |
| **L2** | Produce | L1 + generate Frames whose digests match the Golden Test Vectors |
| **L3** | Sign & Verify | L2 + create and verify Proofs with a declared signature algorithm |

An implementation claiming a level MUST satisfy all lower levels.

Official vectors: `tests/vectors/`.  
Comparison MUST be performed on serialized protocol output only.

---

## 13. Relationship to Other Layers

### 13.1 Capsule

Capsule holds mutable runtime information.  
It MUST NOT be mixed into ACP Core or Transitions when determinism is required.

### 13.2 PLP

PLP is a State Representation profile.  
ACP MAY carry PLP states inside Core.state but does not require PLP.  
Any canonically serializable structured state is valid.

---

## 14. Security Considerations

### 14.1 Threat Model (Informative)

ACP is designed to resist:

| Threat | Mitigation |
|--------|------------|
| Cross-implementation hash divergence | RFC 8785 JCS + strict Decimal + timestamp normalisation |
| Type-confusion / cross-protocol hash reuse | Domain separation tags (algorithm + encoding) |
| DAG injection / cycles | Kahn topological check + single-root rule |
| Replay of old transitions | Unique transition_id + Lamport clocks |
| Oversized proof DoS | Concrete size limits (ACP-201) |
| Signature stripping / target substitution | SignPayload binding + mandatory target_hash match (ACP-203) |

ACP does **not** protect against compromised signers, side-channel leakage in signature implementations, or social engineering of `created_by` / `actor` fields.

### 14.2 Concrete Requirements

- **Hash Strength** — `sha256` mandatory; `sha3-256` / `blake3` optional.
- **Domain Separation** — Includes algorithm and encoding.
- **Replay Protection** — Unique transition_id + Lamport clocks. Optional nonces MAY be placed under `$ext`.
- **Proof Size Limits** — See §6.5.
- **Canonicalization** — RFC 8785 + NFC + Decimal rules.
- **Signature Choice** — Left to implementers; see Appendix E.

---

## 15. IANA Considerations

No formal IANA request in v1.1.0.  
Future versions may register the media types of §9 and the `axiom://` URI scheme.

---

## 16. Versioning Policy

- **PATCH** — Bug fixes that do not change external behaviour.
- **MINOR** — Backward-compatible extensions.
- **MAJOR** — Breaking changes to required structures or semantics.

Golden Test Vectors for a given major.minor series MUST remain stable.

---

## 17. Reference Implementation

```
src/axiom/common_protocol.py
```

Authoritative for version 1.1.0 behaviour where the implementation has been updated to match this specification.  
Other language implementations are expected to match the Golden Test Vector outputs exactly.

---

## 18. References

- RFC 8785 — JSON Canonicalization Scheme (JCS)
- RFC 2119 — Key words for use in RFCs to Indicate Requirement Levels
- RFC 8174 — Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words
- FIPS 180-4 — SHA-256
- FIPS 202 — SHA-3
- BLAKE3 specification
- Kahn’s Algorithm
- Lamport, L. (1978). Time, Clocks, and the Ordering of Events in a Distributed System

---

## Appendix A. Golden Test Vectors

See `tests/vectors/README.md` and `tests/vectors/CONFORMANCE_REPORT.md`.

### A.1 Positive Vectors (summary)

| Vector | core_hash (prefix) | coordinate_id (prefix) | Result |
|--------|--------------------|------------------------|--------|
| minimal | `64b35aeda8bcf99d…` | `361090f3fb4e5a08…` | PASS |
| genesis | `60db5169aa10c30c…` | `c99f37a8952e91d4…` | PASS |
| transition | `64ef83d9c4dcdfbe…` | `9d748f4869dea6fc…` | PASS |
| merge | `ec069060c4d76c43…` | `2035d4c99af0c521…` | PASS |
| extension | `64b35aeda8bcf99d…` | `67888a2951bd169c…` | PASS |
| proof | `64b35aeda8bcf99d…` | `a5edbf249a3255c9…` | PASS |
| replay | `ab6e1126da4e88bd…` | `3576e35a4ae7728a…` | PASS |

### A.2 Negative Vectors

| Vector | Expected Error |
|--------|----------------|
| invalid_cycle | ACP-101 (AxiomCausalError) |
| invalid_duplicate_transition | ACP-102 (AxiomCausalError) |
| invalid_vendor_namespace | ACP-004 (AxiomDeserializationError) |

Full digests and canonical forms are authoritative in the `tests/vectors/expected/` directory.

---

## Appendix B. Canonical Examples

- Minimal Frame → `tests/vectors/minimal.json`
- Single Transition → `tests/vectors/transition.json`
- Branch + Merge → `tests/vectors/merge.json`
- Vendor Extension → `tests/vectors/extension.json`

---

## Appendix C. Capability Negotiation (Informative)

A simple advertisement object that nodes MAY exchange:

```json
{
  "hash": ["sha256", "sha3-256"],
  "proof": ["ed25519", "ecdsa-p256"],
  "encoding": ["jcs"]
}
```

Formal negotiation messages are left to future work.

---

## Appendix D. Future Work (Non-Normative)

- Canonical CBOR profile (`encoding: cbor`)
- Merkle-root optional field for large DAGs
- Formal Replay Window (timestamp tolerance + nonce)
- Streaming / partial verification
- Additional Conformance Levels

---

## Appendix E. Recommended Signature Algorithms (Informative)

| Algorithm | Status | Notes |
|-----------|--------|-------|
| `ed25519` | Recommended | Fast, small signatures, widely available |
| `ecdsa-p256` | Acceptable | Broad hardware support |
| `rsa-3072` | Acceptable | Legacy interoperability |

Implementations claiming Conformance Level L3 SHOULD document which of the above (or other) algorithms they support.

---

## Appendix F. Change Log

| Version | Date | Notes |
|---------|------|-------|
| **1.1.0** | 2026-08-01 | First-class specification. Normative additions since 1.0.x code: `protocol_id`, `target_type` + SignPayload, strict Decimal grammar, recursive `coordinate_id` removal, Leaf definition, timestamp normalisation, size limits, target_hash binding, RFC 8174 keywords, ABNF, algorithm lowercase rule, unknown-scheme ignore, `axiom://frame/` canonical URI, empty-array policy, NFC MUST, threat model, Golden Vector summary table. SemVer minor bump because normative surface expanded. |
| 1.0.3 | 2026-08-01 | Reference implementation + Golden Test Vectors (code) |
| 1.0.2 | 2026-07 | Initial structured Frame |
| 1.0.0 | 2026-06 | Conceptual draft |

---

*End of Specification*
