//! Worked example: substrate-first claim on a sepsis-3 patient state (task 11.3).
//!
//! This example is the human-readable companion to the machine-checked property
//! tests in `clinlat/src/proposer.rs` (task 11.2, "Substrate-Invariance Tests")
//! and the discharge argument in `clinlat/docs/obligations/obl-ps-05-proposer-constraint.md`
//! (task 11.1). It closes M2 DoD by demonstrating, on the same sepsis-3 scenario
//! used throughout Phases 9 and 10, that the substrate's *licensed* outcome is
//! identical across proposer architectures even when the proposers' *raw*
//! candidate sets diverge.
//!
//! **Scenario**: Same 64-year-old patient with community-acquired pneumonia
//! (CAP) and respiratory failure as in `sofa_kdigo_proposer.rs` and
//! `llm_proposer_sepsis.rs`. Only the SOFA respiratory operator is registered
//! here (single operator keeps the divergence-then-convergence story legible).
//!
//! **What we demonstrate**:
//! 1. `LatticeSearchProposer` proposes exactly one candidate (the operator-reachable one).
//! 2. `LlmProposer` (mock) proposes a *divergent* raw candidate set: the same
//!    correct candidate, PLUS a plausible-looking but clinically wrong severity
//!    score, PLUS malformed/unknown-system garbage that never survives parsing.
//! 3. Both proposers are routed through the identical `propose_verify` soundness
//!    gate (per-proposer OperatorSet instances constructed independently, per
//!    this repo's test-independence discipline — see CLAUDE.md "Structuring
//!    tests for safety verification").
//! 4. After the gate, `licensed_candidates` is asserted byte-for-byte equal
//!    across both proposers — the substrate-first claim (NOTE.md §3, §5;
//!    OBL-PS-05): substrate behavior is independent of proposer architecture.

use chrono::Utc;
use clinlat::{
    Evidence, Hyp, LatticeSearchProposer, LlmProposer, LlmProposerConfig, Observation, Operator,
    OperatorMetadata, OperatorSet, Outcome, Provenance, ProvenanceOrigin, RefinementProposer,
    SofaRespOperator, Ver, propose_verify,
};
use std::collections::BTreeMap;

/// Builds the shared sepsis-3 evidence: PaO₂/FiO₂ = 250 (SOFA respiratory score 2),
/// tagged with the provenance version the SofaRespOperator (v0.2.0) expects.
fn build_evidence() -> Evidence {
    let observations = vec![
        Observation::new("LOINC:2703-7", serde_json::json!(150.0))
            .with_unit("mmHg")
            .with_source("Radiology PACS – ABG from 14:30"),
        Observation::new("LOINC:3150-0", serde_json::json!(0.60))
            .with_unit("fraction")
            .with_source("Ventilator settings"),
        Observation::new("SNOMED:243144002", serde_json::json!(true))
            .with_source("Ventilator active"),
    ];

    let provenance = Provenance::new(
        ProvenanceOrigin::new("epic_lms", "LOINC", "2703-7"),
        Utc::now(),
        Ver::new("clinlat", "sofa_resp", "0.2.0"),
        BTreeMap::new(),
    );

    Evidence::new(observations, provenance)
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("Worked Example: Substrate-First Claim on Sepsis-3 (Task 11.3)");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("PATIENT PRESENTATION:");
    println!("  Name: Patient A (64-year-old)");
    println!("  Chief complaint: Fever, cough, shortness of breath × 2 days");
    println!("  Admission diagnosis: Community-acquired pneumonia (CAP)");
    println!("  Current status: Intubated on mechanical ventilation\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 1: Shared evidence, both proposers see the identical input
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 1: Construct Evidence (identical for both proposers)");
    println!("────────────────────────────────────────────────────────\n");

    let evidence = build_evidence();

    println!("  • PaO₂ = 150 mmHg, FiO₂ = 0.60 → ratio = 250 (SOFA respiratory input)");
    println!("  • On mechanical ventilation: YES\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 2: Ground-truth operator output (what the SOFA operator actually
    // licenses for this evidence). Derived by calling the operator directly,
    // not hardcoded, so this example stays correct if SofaRespOperator's
    // internal atom encoding ever changes.
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 2: Ground-Truth Operator Output");
    println!("─────────────────────────────────────\n");

    let ground_truth_op = SofaRespOperator::default_v0_2();
    let ground_truth_atom = match ground_truth_op.apply(&Hyp::unknown(), &evidence) {
        Outcome::Refined(refined) => refined
            .atoms()
            .first()
            .cloned()
            .expect("SofaRespOperator must produce exactly one atom for this evidence"),
        other => panic!(
            "expected Outcome::Refined for this evidence, got {:?}",
            other
        ),
    };

    println!(
        "  SofaRespOperator licenses: {}:{}@{} ({})\n",
        ground_truth_atom.system,
        ground_truth_atom.code,
        ground_truth_atom.version,
        ground_truth_atom.preferred_term
    );

    // ─────────────────────────────────────────────────────────────────────
    // STEP 3: LatticeSearchProposer — exhaustive, trivially-sound search.
    // Its own OperatorSet instance (used only for candidate generation).
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 3: LatticeSearchProposer (Task 9.1) — Raw Candidates");
    println!("───────────────────────────────────────────────────────\n");

    let ops_for_lattice_search = OperatorSet::new().register(
        Box::new(SofaRespOperator::default_v0_2()),
        OperatorMetadata {
            name: "sofa_resp".to_string(),
            version: "clinlat-v0.2.0".to_string(),
        },
    );
    let lattice_proposer = LatticeSearchProposer::new(ops_for_lattice_search);

    let lattice_raw_candidates = lattice_proposer.propose(&Hyp::unknown(), &evidence);

    println!(
        "  Raw candidate count: {} (exhaustive search over 1 operator)",
        lattice_raw_candidates.len()
    );
    for c in &lattice_raw_candidates {
        println!(
            "    • {:?}",
            c.atoms()
                .iter()
                .map(|a| format!("{}:{}@{}", a.system, a.code, a.version))
                .collect::<Vec<_>>()
        );
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // STEP 4: LlmProposer (mock) — divergent raw candidates.
    // Line 1: the same correct candidate as the lattice search (derived from
    //         ground_truth_atom, so it stays in sync with the operator).
    // Line 2: a plausible-looking but WRONG severity score (hallucination that
    //         survives ontology parsing — will be caught at the LICENSING
    //         stage, not the parse stage).
    // Line 3: malformed garbage — dropped SILENTLY at the PARSE stage, never
    //         reaches propose_and_filter's candidate set at all.
    // Line 4: unknown ontology system — also dropped SILENTLY at parse stage.
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 4: LlmProposer (Task 10.2, mock) — Divergent Raw Candidates");
    println!("──────────────────────────────────────────────────────────────\n");

    let correct_line = format!(
        "{}:{}@{}",
        ground_truth_atom.system, ground_truth_atom.code, ground_truth_atom.version
    );
    // Same code family as the ground-truth atom, but the trailing digit is
    // altered to name a different (wrong) SOFA severity score — syntactically
    // valid, ontology-bounded, but not what this evidence actually licenses.
    let wrong_severity_line = correct_line.replace("resp-2", "resp-4");

    let response_text = format!(
        "{}\n{}\nNOT_AN_ATOM_AT_ALL\nFHIR:99999@0.2.0",
        correct_line, wrong_severity_line
    );
    // Two identical copies: mock responses are a FIFO queue, and this example
    // calls .propose() directly (below) AND indirectly via propose_verify
    // (Step 5) — each call pops one entry.
    let mock_response = vec![response_text.clone(), response_text];

    println!("  Mock LLM response (4 lines):");
    println!(
        "    1. {}   (correct — matches operator output)",
        correct_line
    );
    println!(
        "    2. {}   (hallucination: wrong severity, ontology-valid)",
        wrong_severity_line
    );
    println!("    3. NOT_AN_ATOM_AT_ALL             (hallucination: no ':', fails parse_atom)");
    println!("    4. FHIR:99999@0.2.0               (hallucination: unknown ontology system)\n");

    let config = LlmProposerConfig::mock("test-model", mock_response, "0.1.0");
    let llm_proposer = LlmProposer::new(config);

    let llm_raw_candidates = llm_proposer.propose(&Hyp::unknown(), &evidence);

    println!(
        "  Raw candidate count: {} (lines 3–4 dropped silently at parse stage; never reach here)",
        llm_raw_candidates.len()
    );
    for c in &llm_raw_candidates {
        println!(
            "    • {:?}",
            c.atoms()
                .iter()
                .map(|a| format!("{}:{}@{}", a.system, a.code, a.version))
                .collect::<Vec<_>>()
        );
    }
    println!();

    println!(
        "  ⚠ Raw candidate sets already DIVERGE: LatticeSearchProposer produced {}, LlmProposer produced {}.\n",
        lattice_raw_candidates.len(),
        llm_raw_candidates.len()
    );

    // ─────────────────────────────────────────────────────────────────────
    // STEP 5: Route both through the identical soundness-verification gate.
    // Per this repo's test-independence discipline, each propose_verify call
    // gets its OWN independently-constructed OperatorSet — this validates the
    // invariant across independently-built objects, not merely that a single
    // shared instance behaves consistently with itself.
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 5: Soundness-Verification Gate (propose_verify, INV-PS-06)");
    println!("─────────────────────────────────────────────────────────────\n");

    let ops_for_lattice_verify = OperatorSet::new().register(
        Box::new(SofaRespOperator::default_v0_2()),
        OperatorMetadata {
            name: "sofa_resp".to_string(),
            version: "clinlat-v0.2.0".to_string(),
        },
    );
    let ops_for_llm_verify = OperatorSet::new().register(
        Box::new(SofaRespOperator::default_v0_2()),
        OperatorMetadata {
            name: "sofa_resp".to_string(),
            version: "clinlat-v0.2.0".to_string(),
        },
    );

    let lattice_verify = propose_verify(
        &lattice_proposer,
        &ops_for_lattice_verify,
        &Hyp::unknown(),
        &evidence,
    )
    .expect("LatticeSearchProposer candidate must be licensed");
    let llm_verify = propose_verify(
        &llm_proposer,
        &ops_for_llm_verify,
        &Hyp::unknown(),
        &evidence,
    )
    .expect("LlmProposer's correct candidate must be licensed");

    println!(
        "  LatticeSearchProposer: {} licensed, {} unlicensed",
        lattice_verify.licensed_candidates.len(),
        lattice_verify.unlicensed_candidates.len()
    );
    println!(
        "  LlmProposer:           {} licensed, {} unlicensed",
        llm_verify.licensed_candidates.len(),
        llm_verify.unlicensed_candidates.len()
    );
    println!(
        "\n  LlmProposer's wrong-severity hallucination (line 2) parsed fine but was\n  rejected HERE — at licensing, not at parsing — because its atom is not a\n  subset of what SofaRespOperator.apply_set() actually returned.\n"
    );

    // ─────────────────────────────────────────────────────────────────────
    // STEP 6: The substrate-first claim, mechanically checked.
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("SUBSTRATE-FIRST CLAIM (OBL-PS-05, NOTE.md §3 / §5):");
    println!("═══════════════════════════════════════════════════════════════\n");

    let identical = lattice_verify.licensed_candidates == llm_verify.licensed_candidates;

    println!(
        "  licensed_candidates identical across proposer swap: {}\n",
        if identical { "✓ YES" } else { "✗ NO" }
    );

    assert!(
        identical,
        "substrate-invariance violated: licensed_candidates differ across proposer architectures"
    );

    println!(
        "  Despite divergent raw candidate sets ({} vs. {} candidates,",
        lattice_raw_candidates.len(),
        llm_raw_candidates.len()
    );
    println!("  including hallucinated wrong-severity and malformed output), the");
    println!("  substrate's licensed outcome is byte-for-byte identical:\n");
    for c in &lattice_verify.licensed_candidates {
        println!(
            "    {:?}",
            c.atoms()
                .iter()
                .map(|a| format!("{}:{}@{}", a.system, a.code, a.version))
                .collect::<Vec<_>>()
        );
    }
    println!();
    println!("  This is the empirical demonstration referenced in M2 DoD:");
    println!("  \"substrate behavior is independent of proposer architecture\" —");
    println!("  safety and correctness are enforced by the deduction substrate");
    println!("  (operators + soundness gate), not by which proposer generated");
    println!("  the candidates in the first place.\n");

    println!("  See also:");
    println!("    • clinlat/docs/obligations/obl-ps-05-proposer-constraint.md (Task 11.1)");
    println!("    • clinlat/src/proposer.rs, \"Substrate-Invariance Tests\" (Task 11.2)");
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!("END OF EXAMPLE");
    println!("═══════════════════════════════════════════════════════════════");
}
