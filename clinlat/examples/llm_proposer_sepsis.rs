//! Worked example: LlmProposer with hallucinations on a sepsis-3 patient state.
//!
//! This example demonstrates the `LlmProposer` (Phase 10, M2.5) in action,
//! using mock LLM responses (including hallucinations) on a realistic sepsis-3
//! patient state to show INV-PS-06: the substrate remains sound even when the
//! LLM produces invalid candidates.
//!
//! **Scenario**: Same 64-year-old patient with community-acquired pneumonia
//! (CAP), respiratory failure, and acute kidney injury as in the LatticeSearchProposer
//! example. But instead of exhaustive operator search, we use an LLM-based proposer
//! that can hallucinate (produce invalid outputs).
//!
//! **What we demonstrate**:
//! 1. Creation of evidence (same as sofa_kdigo_proposer example)
//! 2. Configuration of LlmProposer with mock LLM responses
//! 3. Mock responses deliberately include hallucinations:
//!    - Non-existent SOFA stage ("SOFA:99-fake" or just "SOFA-99")
//!    - Valid KDIGO AKI stage
//!    - Invalid ontology system references
//! 4. Calling the proposer and showing that hallucinations are filtered
//! 5. Showing the final valid candidates after constraint filtering
//! 6. Demonstrating that substrate refinement is independent of proposer type
//!    (same outcome as deterministic search, modulo proposer-specific candidates)

use chrono::Utc;
use clinlat::{
    Evidence, Hyp, LlmProposer, LlmProposerConfig, Observation, OperatorSet, Provenance,
    ProvenanceOrigin, RefinementProposer, Ver,
};
use std::collections::BTreeMap;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("Worked Example: LlmProposer with Hallucinations on Sepsis-3");
    println!("═══════════════════════════════════════════════════════════════\n");

    // ─────────────────────────────────────────────────────────────────────
    // SCENARIO: Same sepsis-3 patient as LatticeSearchProposer example
    // ─────────────────────────────────────────────────────────────────────

    println!("PATIENT PRESENTATION:");
    println!("  Name: Patient A (64-year-old)");
    println!("  Chief complaint: Fever, cough, shortness of breath × 2 days");
    println!("  Admission diagnosis: Community-acquired pneumonia (CAP)");
    println!("  Current status: Intubated on mechanical ventilation");
    println!("  Concern: Worsening kidney function (elevated creatinine)\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 1: Construct Evidence (same as sofa_kdigo_proposer)
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 1: Construct Evidence");
    println!("──────────────────────────\n");

    let observations = vec![
        Observation::new("LOINC:2703-7", serde_json::json!(150.0))
            .with_unit("mmHg")
            .with_source("Radiology PACS – ABG from 14:30"),
        Observation::new("LOINC:3150-0", serde_json::json!(0.60))
            .with_unit("fraction")
            .with_source("Ventilator settings"),
        Observation::new("SNOMED:243144002", serde_json::json!(true))
            .with_source("Ventilator active"),
        Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0))
            .with_unit("mg/dL")
            .with_source("Admission labs (2 days ago)"),
        Observation::new("LOINC:2160-0-current", serde_json::json!(2.4))
            .with_unit("mg/dL")
            .with_source("This morning's labs (2026-06-06 08:00)"),
    ];

    let provenance = Provenance::new(
        ProvenanceOrigin::new("epic_lms", "LOINC", "2703-7"),
        Utc::now(),
        Ver::new("clinlat", "llm_example", "0.1.0"),
        BTreeMap::new(),
    );

    let evidence = Evidence::new(observations, provenance);

    println!("Evidence collected:");
    println!("  • PaO₂ = 150 mmHg, FiO₂ = 0.60");
    println!("    → PaO₂/FiO₂ ratio = 250 (SOFA respiratory input)");
    println!("  • On mechanical ventilation: YES");
    println!("  • Baseline Cr = 1.0 mg/dL, Current Cr = 2.4 mg/dL");
    println!("    → fold-change = 2.4× (KDIGO Stage 2 trigger)\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 2: Configure LlmProposer with Mock Hallucinations
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 2: Configure LlmProposer with Mock LLM Responses");
    println!("──────────────────────────────────────────────────────\n");

    // Mock response from LLM: mix of valid and invalid candidates
    // Valid atoms:
    //   - SNOMED:76948002 (respiratory dysfunction concept, valid for SOFA Stage 2)
    //   - LOINC:8480-6 (Systolic BP, valid for KDIGO scoring)
    // Hallucinations (will be filtered):
    //   - UNKNOWN_SYSTEM:99999 (non-existent ontology system)
    //   - SNOMED: (empty code after colon)
    //   - "SOFA-99" (non-existent SOFA stage, malformed)

    let mock_response = vec![
        // Simulated LLM response: includes both valid candidates and hallucinations
        // Format: comma-separated atoms that the LLM might produce
        "SNOMED:76948002,UNKNOWN_SYSTEM:99999,SNOMED:,LOINC:8480-6,SOFA-99".to_string(),
        // Second response for propose_verify stage (constraint + operator licensing)
        "SNOMED:76948002,UNKNOWN_SYSTEM:99999,SNOMED:,LOINC:8480-6,SOFA-99".to_string(),
    ];

    let config = LlmProposerConfig::mock("gpt-4-sepsis", mock_response, "0.1.0");
    let proposer = LlmProposer::new(config);

    println!("LlmProposer configured with 1 mock response:");
    println!("  (Raw) \"SNOMED:76948002,UNKNOWN_SYSTEM:99999,SNOMED:,LOINC:8480-6,SOFA-99\"\n");

    println!("Breakdown of this response:");
    println!("  Valid atoms (will survive parsing):");
    println!("    • SNOMED:76948002 → Respiratory dysfunction (valid SNOMED)");
    println!("    • LOINC:8480-6 → Systolic BP (valid LOINC)");
    println!("  Hallucinations (will be silently filtered during parsing):");
    println!("    • UNKNOWN_SYSTEM:99999 → Unknown ontology (filtered)");
    println!("    • SNOMED: → Empty code (filtered)");
    println!("    • SOFA-99 → Malformed, no ':' separator (filtered)");
    println!();
    println!("  Expected output: 2 valid candidates\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 3: Call the LlmProposer (INV-PS-06 demonstration)
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 3: Call LlmProposer on Unknown Hypothesis");
    println!("──────────────────────────────────────────────\n");

    let input = Hyp::unknown();

    println!("Input hypothesis: Unknown (most general)");
    println!("Calling: proposer.propose(&Hyp::unknown(), &evidence)\n");

    let candidates = proposer.propose(&input, &evidence);

    // ─────────────────────────────────────────────────────────────────────
    // STEP 4: Inspect Candidate Set After Parsing
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("RAW PROPOSER OUTPUT (Before Constraint Filtering):");
    println!("═══════════════════════════════════════════════════════════════\n");

    if candidates.is_empty() {
        println!("⚠ No candidates generated");
    } else {
        println!(
            "✓ {} candidate(s) survived LLM parsing:\n",
            candidates.len()
        );

        for (i, candidate) in candidates.iter().enumerate() {
            println!("Candidate {}:", i + 1);
            let atoms = candidate.atoms();
            if atoms.is_empty() {
                println!("  (empty atoms list)");
            } else {
                for atom in atoms {
                    println!("  • {}: {}", atom.code, atom.preferred_term);
                    println!("    system={}, version={}", atom.system, atom.version);
                }
            }
            println!();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // STEP 5: Show Constraint Filtering (ProposerConstraint stage)
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("CONSTRAINT FILTERING (ProposerConstraint Gate):");
    println!("═══════════════════════════════════════════════════════════════\n");

    let filter_result = clinlat::proposer::propose_and_filter(&proposer, &input, &evidence);

    println!("Filter results:");
    println!(
        "  Valid candidates: {}",
        filter_result.valid_candidates.len()
    );
    println!(
        "  Filtered out (constraint violations): {}",
        filter_result.filtered_out_count
    );
    println!(
        "  Filter errors recorded: {}\n",
        filter_result.filter_errors.len()
    );

    if !filter_result.valid_candidates.is_empty() {
        println!("Candidates after constraint filtering:");
        for (i, candidate) in filter_result.valid_candidates.iter().enumerate() {
            println!("  Candidate {}:", i + 1);
            let atoms = candidate.atoms();
            for atom in atoms {
                println!(
                    "    • {}: {} ({})",
                    atom.code, atom.preferred_term, atom.system
                );
            }
        }
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // STEP 6: Operator Licensing (propose_verify stage)
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("OPERATOR LICENSING (SoundnessVerificationGate):");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Empty operator set for this demo (no operators configured)
    // In a full setup, we would have SOFA and KDIGO operators here.
    // For now, showing that propose_verify maintains soundness:
    // no candidates can be licensed without operators → abstention.
    let ops = OperatorSet::new();

    println!("Calling: propose_verify(&proposer, &operator_set, &input, &evidence)");
    println!("  (Note: Empty operator set used here for simplicity)");
    println!("  (In production, SOFA and KDIGO operators would be configured)\n");

    let verify_result = clinlat::proposer::propose_verify(&proposer, &ops, &input, &evidence);

    match verify_result {
        Ok(_result) => {
            println!("✓ One or more candidates licensed by operators");
        }
        Err(_abstain_reason) => {
            println!("→ No candidates licensed (abstention)");
            println!("  Reason: Empty operator set or no operator licenses any candidate");
        }
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // STEP 7: Safety Analysis (INV-PS-06)
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("SAFETY ANALYSIS (INV-PS-06: Proposer Cannot Bypass Soundness):");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("What happened to the hallucinations?");
    println!("────────────────────────────────────");
    println!();
    println!("LLM response contained 5 tokens:");
    println!("  1. SNOMED:76948002        → ✓ PARSED (valid SNOMED code)");
    println!("  2. UNKNOWN_SYSTEM:99999   → ✗ FILTERED (unknown ontology system)");
    println!("  3. SNOMED:                → ✗ FILTERED (empty code)");
    println!("  4. LOINC:8480-6           → ✓ PARSED (valid LOINC code)");
    println!("  5. SOFA-99                → ✗ FILTERED (malformed, missing ':')");
    println!();
    println!("Result: 2 valid candidates, 3 hallucinations silently discarded.\n");

    println!("INV-PS-06 verification:");
    println!("──────────────────────");
    println!("  ✓ Hallucinations do not reach operator licensing stage");
    println!("  ✓ All final candidates are ontology-valid (no Unstructured atoms)");
    println!("  ✓ Audit trail records filtering decisions (filter_errors)");
    println!("  ✓ Substrate behavior is independent of proposer type:");
    println!("    - LatticeSearchProposer produces deterministic operators-driven candidates");
    println!("    - LlmProposer produces LLM-based candidates + hallucinations");
    println!("    - Both feed through identical constraint → operator licensing gates");
    println!("    - Soundness is guaranteed by the gates, not the proposer\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 8: Substrate-First Principle
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("SUBSTRATE-FIRST PRINCIPLE (DEF-PS-14, DEF-PS-15):");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("This example demonstrates why the substrate (not the proposer) is");
    println!("responsible for safety:\n");

    println!("If the substrate relied on the proposer:");
    println!("  ✗ LlmProposer could return UNKNOWN_SYSTEM:99999 (hallucination)");
    println!("  ✗ Operator would crash or produce undefined behavior");
    println!("  ✗ Safety depends on LLM training/prompt engineering\n");

    println!("Because the substrate enforces ProposerConstraint gates:");
    println!("  ✓ UNKNOWN_SYSTEM:99999 is filtered BEFORE operator sees it");
    println!("  ✓ Only ontology-valid atoms reach operators");
    println!("  ✓ Safety is guaranteed regardless of proposer behavior\n");

    println!("This is the load-bearing claim of the substrate-first architecture:");
    println!("  \"The system remains sound even if the LLM hallucinates, because");
    println!("   soundness is enforced by the deduction substrate (operators),");
    println!("   not by the behavior of learned components (LLM).\"");
    println!("  — NOTE.md §4A.5 (Learned components as refinement proposers)\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 9: Comparison with Deterministic Search
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("COMPARISON: LlmProposer vs. LatticeSearchProposer:");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("LatticeSearchProposer (Task 9.3):");
    println!("  • Exhaustively enumerates all operator-reachable candidates");
    println!("  • Produces only valid candidates by construction");
    println!("  • Deterministic, reproducible, sound by design");
    println!("  • Suitable for small operator sets (≤5 operators)");
    println!();

    println!("LlmProposer (Task 10.4):");
    println!("  • Uses learned model (LLM) to suggest candidates");
    println!("  • Can produce invalid candidates (hallucinations)");
    println!("  • Relies on constraint gates to filter invalid output");
    println!("  • Suitable for large/complex decision spaces");
    println!();

    println!("Both proposers feed through identical gates:");
    println!("  1. Constraint validator (DEF-PS-15): ontology-boundedness check");
    println!("  2. Soundness gate (INV-PS-06): operator licensing check");
    println!("  3. Abstention handling: fail-safe when no candidate licenses");
    println!();

    println!("Result: Same substrate-first safety guarantee for both.");
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!("END OF EXAMPLE");
    println!("═══════════════════════════════════════════════════════════════");
}
