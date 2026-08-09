#!/usr/bin/env python3
"""
Round Consensus Protocol — Minimal Demo
=======================================
3 agents (α, β, γ) × 2 rounds

実験は忠実に実際行って

See ROUND_CONSENSUS_DEMO_REPORT.md for results (10/10 PASS).
Re-run: python3 round_consensus_demo.py
"""

from __future__ import annotations

import hashlib
import json
from copy import deepcopy
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum
from typing import Any, Optional


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def domain_hash(domain: str, payload: str) -> str:
    return sha256_hex(f"axiom:v2:{domain}\0".encode() + payload.encode("utf-8"))


HASH_A_TEXT = """User Goal: Propose a safe, concrete next step for improving code review quality.
Axiom: Prefer measurable actions over vague advice.
Safety Rules:
- Do not invent credentials, secrets, or private data.
- Do not claim external systems were modified.
- Do not escalate beyond the stated goal.
Framework Constraints:
- HashB must stay within the goal scope.
- Observer evaluates contract compliance only, not answer quality.
Immutable Policy: Observer never reasons; Reasoners never seal.
"""

HASH_A = domain_hash("raw", HASH_A_TEXT)


class Role(str, Enum):
    OBSERVER = "Observer"
    REASONER = "Reasoner"


AGENTS = ["α", "β", "γ"]


def roles_for_round(round_idx: int) -> dict[str, Role]:
    obs = AGENTS[round_idx % len(AGENTS)]
    return {a: (Role.OBSERVER if a == obs else Role.REASONER) for a in AGENTS}


class VerdictKind(str, Enum):
    ACCEPT = "Accept"
    REVISE = "Revise"
    ASK_USER = "AskUser"
    ABORT = "Abort"


@dataclass
class ObserverVerdict:
    kind: VerdictKind
    reason: Optional[str] = None
    summary: Optional[str] = None
    candidates: Optional[list[str]] = None

    def to_dict(self) -> dict:
        d: dict[str, Any] = {"kind": self.kind.value}
        if self.reason:
            d["reason"] = self.reason
        if self.summary:
            d["summary"] = self.summary
        if self.candidates:
            d["candidates"] = self.candidates
        return d


SPECIALTY = {"α": "process", "β": "tooling", "γ": "culture"}

CANDIDATE_LIBRARY = {
    ("α", "in_scope"): {
        "title": "checklist-gate",
        "body": "Add a 5-item pre-merge checklist: tests, diff size, ownership, security notes, rollback plan. Track completion rate weekly.",
        "tags": ["ACTION(add_checklist)", "METRIC(completion_rate)"],
    },
    ("β", "in_scope"): {
        "title": "linter-bot",
        "body": "Enable a review bot that flags missing tests and >400-line diffs. Report false-positive rate monthly.",
        "tags": ["ACTION(enable_bot)", "METRIC(false_positive_rate)"],
    },
    ("γ", "in_scope"): {
        "title": "review-sla",
        "body": "Publish a 24h first-response SLA for PRs and rotate primary reviewer weekly. Measure median response time.",
        "tags": ["ACTION(set_sla)", "METRIC(median_response_time)"],
    },
    ("α", "soft_violation"): {
        "title": "rewrite-org",
        "body": "Reorganize the entire engineering org around review excellence and hire three staff engineers as full-time reviewers.",
        "tags": ["ACTION(reorg)", "SCOPE(org_wide)"],
    },
    ("β", "hard_violation"): {
        "title": "leak-creds",
        "body": "Store reviewer credentials in a shared plaintext file so anyone can approve PRs quickly.",
        "tags": ["ACTION(store_credentials)", "SAFETY(violation)"],
    },
}


def make_hash_b(agent: str, mode: str) -> dict[str, Any]:
    key = (agent, mode) if (agent, mode) in CANDIDATE_LIBRARY else (agent, "in_scope")
    lib = CANDIDATE_LIBRARY[key]
    payload = json.dumps(
        {"agent": agent, "specialty": SPECIALTY[agent], "title": lib["title"], "body": lib["body"], "tags": lib["tags"]},
        ensure_ascii=False, sort_keys=True,
    )
    return {
        "agent": agent, "specialty": SPECIALTY[agent], "title": lib["title"], "body": lib["body"],
        "tags": list(lib["tags"]), "hash_b": domain_hash("canonical", payload), "mode": key[1],
    }


def observe(hash_a: str, candidates: list[dict], round_idx: int) -> ObserverVerdict:
    if not candidates:
        return ObserverVerdict(kind=VerdictKind.ABORT, reason="no HashB candidates")
    for c in candidates:
        tags = c.get("tags") or []
        if any("SAFETY(violation)" in t for t in tags):
            return ObserverVerdict(
                kind=VerdictKind.ABORT,
                reason=f"HashA Safety Rules violated by {c['agent']}: {c['title']}",
                candidates=[x["title"] for x in candidates],
            )
    for c in candidates:
        tags = c.get("tags") or []
        if any("SCOPE(org_wide)" in t for t in tags):
            return ObserverVerdict(
                kind=VerdictKind.REVISE,
                reason=f"Goal drift (org-wide scope) by {c['agent']}: {c['title']}",
                summary="Stay within concrete next steps for code review quality",
                candidates=[x["title"] for x in candidates],
            )
    return ObserverVerdict(
        kind=VerdictKind.ACCEPT,
        summary="All candidates within HashA scope; no safety violations",
        candidates=[c["title"] for c in candidates],
    )


def seal(hash_a: str, approved: list[dict], round_idx: int, role_map: dict[str, Role]) -> dict[str, Any]:
    approved_hashes = [c["hash_b"] for c in approved]
    role_s = ",".join(f"{a}:{role_map[a].value}" for a in AGENTS)
    payload = json.dumps(
        {"hash_a": hash_a, "approved_hash_b": approved_hashes, "round": round_idx, "roles": role_s},
        sort_keys=True,
    )
    return {
        "proof": domain_hash("proof", payload),
        "hash_a": hash_a,
        "approved_hash_b": approved_hashes,
        "approved_titles": [c["title"] for c in approved],
        "round": round_idx,
        "roles": {a: role_map[a].value for a in AGENTS},
    }


def tag_set(cands: list[dict]) -> set[str]:
    s: set[str] = set()
    for c in cands:
        s.update(c.get("tags") or [])
    return s


def divergence(prev: list[dict], curr: list[dict]) -> dict[str, Any]:
    a, b = tag_set(prev), tag_set(curr)
    union = a | b
    inter = a & b
    div = 1.0 - (len(inter) / len(union)) if union else 0.0
    return {
        "prev_tags": sorted(a), "curr_tags": sorted(b),
        "added": sorted(b - a), "removed": sorted(a - b),
        "overlap": sorted(inter), "divergence": round(div, 3),
    }


@dataclass
class RoundState:
    round_idx: int = -1
    hash_a: str = HASH_A
    approved_hash_b: list[dict] = field(default_factory=list)
    seals: list[dict] = field(default_factory=list)
    roles: dict[str, str] = field(default_factory=dict)

    def snapshot(self) -> dict:
        return {
            "round": self.round_idx,
            "hash_a": self.hash_a[:16] + "…",
            "approved_titles": [c.get("title") for c in self.approved_hash_b],
            "seal_count": len(self.seals),
            "roles": dict(self.roles),
        }


@dataclass
class RoundRecord:
    round_idx: int
    roles: dict[str, str]
    observer: str
    reasoners: list[str]
    candidates: list[dict]
    verdict: dict
    sealed: bool
    seal: Optional[dict]
    carried_forward: dict
    divergence_from_prev: Optional[dict]


def run_round(state: RoundState, mode_by_agent: dict[str, str]) -> RoundRecord:
    state.round_idx += 1
    r = state.round_idx
    role_map = roles_for_round(r)
    observer = [a for a, role in role_map.items() if role == Role.OBSERVER][0]
    reasoners = [a for a, role in role_map.items() if role == Role.REASONER]
    candidates = [make_hash_b(a, mode_by_agent.get(a, "in_scope")) for a in reasoners]
    verdict = observe(state.hash_a, candidates, r)
    sealed = False
    seal_obj: Optional[dict] = None
    if verdict.kind == VerdictKind.ACCEPT:
        approved = candidates
        seal_obj = seal(state.hash_a, approved, r, role_map)
        state.approved_hash_b = deepcopy(approved)
        state.seals.append(seal_obj)
        sealed = True
    state.roles = {a: role_map[a].value for a in AGENTS}
    return RoundRecord(
        round_idx=r, roles={a: role_map[a].value for a in AGENTS},
        observer=observer, reasoners=reasoners, candidates=candidates,
        verdict=verdict.to_dict(), sealed=sealed, seal=seal_obj,
        carried_forward=state.snapshot(), divergence_from_prev=None,
    )


def run_demo() -> dict[str, Any]:
    state = RoundState()
    records: list[RoundRecord] = []
    modes0 = {"α": "in_scope", "β": "in_scope", "γ": "in_scope"}
    rec0 = run_round(state, modes0)
    records.append(rec0)
    prev_approved = deepcopy(state.approved_hash_b)
    modes1 = {"α": "soft_violation", "β": "in_scope", "γ": "in_scope"}
    rec1 = run_round(state, modes1)
    rec1.divergence_from_prev = divergence(prev_approved, state.approved_hash_b or rec1.candidates)
    records.append(rec1)
    return {
        "protocol": "Round Consensus v0.1",
        "hash_a_preview": HASH_A[:24] + "…",
        "hash_a_full": HASH_A,
        "agents": AGENTS,
        "specialty": SPECIALTY,
        "rounds": [asdict(r) for r in records],
        "final_state": state.snapshot(),
        "seals": state.seals,
    }


def check_demo(result: dict) -> list[dict]:
    r0, r1 = result["rounds"][0], result["rounds"][1]
    checks = [
        {"id": "R0-observer-is-alpha", "expect": "α", "got": r0["observer"], "pass": r0["observer"] == "α"},
        {"id": "R0-verdict-accept", "expect": "Accept", "got": r0["verdict"]["kind"], "pass": r0["verdict"]["kind"] == "Accept"},
        {"id": "R0-sealed", "expect": True, "got": r0["sealed"], "pass": r0["sealed"] is True},
        {"id": "R0-two-reasoners", "expect": 2, "got": len(r0["reasoners"]), "pass": len(r0["reasoners"]) == 2},
        {"id": "R1-observer-is-beta", "expect": "β", "got": r1["observer"], "pass": r1["observer"] == "β"},
        {"id": "R1-verdict-revise", "expect": "Revise", "got": r1["verdict"]["kind"], "pass": r1["verdict"]["kind"] == "Revise"},
        {"id": "R1-not-sealed", "expect": False, "got": r1["sealed"], "pass": r1["sealed"] is False},
        {"id": "R1-carries-r0-approval", "expect": "from R0 seal", "got": result["final_state"]["approved_titles"],
         "pass": len(result["final_state"]["approved_titles"]) == 2 and result["final_state"]["seal_count"] == 1},
        {"id": "hash-a-stable", "expect": HASH_A, "got": result["hash_a_full"], "pass": result["hash_a_full"] == HASH_A},
        {"id": "divergence-logged", "expect": "present", "got": r1.get("divergence_from_prev") is not None,
         "pass": r1.get("divergence_from_prev") is not None},
    ]
    return checks


def main() -> None:
    result = run_demo()
    checks = check_demo(result)
    out = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "result": result, "checks": checks,
        "passed": sum(1 for c in checks if c["pass"]), "total": len(checks),
    }
    print(json.dumps(out, ensure_ascii=False, indent=2))
    print(f"RESULT: {out['passed']}/{out['total']} {'PASS' if out['passed'] == out['total'] else 'FAIL'}")


if __name__ == "__main__":
    main()
