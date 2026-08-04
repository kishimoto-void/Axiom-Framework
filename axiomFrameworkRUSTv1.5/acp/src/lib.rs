//! AXIOM Common Protocol (ACP) v1.1.0 — Crate entry
//!
//! **Setup (one-time):** copy the full reference implementation into this file:
//!
//! ```bash
//! # from repo root
//! cp axiomFrameworkRUSTv1.5/acp_v1_1_0_reference.rs axiomFrameworkRUSTv1.5/acp/src/lib.rs
//! # then strip the leading markdown commentary if present, keeping from the first `//!` / `use` line
//! cargo test --manifest-path axiomFrameworkRUSTv1.5/acp/Cargo.toml
//! ```
//!
//! The normative full source currently lives at:
//! `axiomFrameworkRUSTv1.5/acp_v1_1_0_reference.rs`
//!
//! Dependencies are declared in `Cargo.toml` (chrono, regex, serde, serde_json, sha2, sha3, blake3, ryu).

// Temporary empty module so the crate path exists until the full lib is copied.
#[cfg(test)]
mod setup_note {
    #[test]
    fn remember_to_copy_full_source() {
        // After copying acp_v1_1_0_reference.rs → src/lib.rs, this test module is replaced.
        assert!(true);
    }
}
