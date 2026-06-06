//! Worked example: SOFA + KDIGO proposer on a sepsis-3 patient state.
//!
//! This example demonstrates the `LatticeSearchProposer` (Phase 9, M2.4) in action,
//! using both the SOFA respiratory and KDIGO AKI operators to generate candidate
//! refinements for a realistic sepsis-3 patient state.
//!
//! **Scenario**: A 64-year-old patient with community-acquired pneumonia (CAP)
//! presenting with respiratory failure and acute kidney injury. The substrate
//! progressively refines hypotheses as evidence accumulates.
//!
//! **What we demonstrate**:
//! 1. Creation of evidence (ABG + creatinine observations) typical in sepsis-3
//! 2. Instantiation of SOFA respiratory and KDIGO AKI operators
//! 3. Building an OperatorSet and LatticeSearchProposer
//! 4. Calling the proposer on the unknown hypothesis to generate candidates
//! 5. Inspecting the candidate set to show refinement diversity

use chrono::Utc;
use clinlat::{
    Evidence, Hyp, KdigoAkiOperator, LatticeSearchProposer, Observation, OperatorMetadata,
    OperatorSet, Provenance, ProvenanceOrigin, RefinementProposer, SofaRespOperator, Ver,
};
use std::collections::BTreeMap;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("Worked Example: SOFA + KDIGO Proposer on Sepsis-3 Patient");
    println!("═══════════════════════════════════════════════════════════════\n");

    // ─────────────────────────────────────────────────────────────────────
    // SCENARIO: Sepsis-3 patient with respiratory failure and AKI
    // ─────────────────────────────────────────────────────────────────────

    println!("PATIENT PRESENTATION:");
    println!("  Name: Patient A (64-year-old)");
    println!("  Chief complaint: Fever, cough, shortness of breath × 2 days");
    println!("  Admission diagnosis: Community-acquired pneumonia (CAP)");
    println!("  Current status: Intubated on mechanical ventilation");
    println!("  Concern: Worsening kidney function (elevated creatinine)\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 1: Construct Evidence (Observations + Provenance)
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 1: Construct Evidence");
    println!("──────────────────────────\n");

    // Blood gas observations (SOFA respiratory input)
    let observations = vec![
        // Arterial blood gas: PaO₂ 150 mmHg on FiO₂ 0.60
        // → PaO₂/FiO₂ ratio = 150 / 0.60 = 250 mmHg
        // → SOFA respiratory score 2 (200–299 mmHg range)
        Observation::new("LOINC:2703-7", serde_json::json!(150.0))
            .with_unit("mmHg")
            .with_source("Radiology PACS – ABG from 14:30"),
        Observation::new("LOINC:3150-0", serde_json::json!(0.60))
            .with_unit("fraction")
            .with_source("Ventilator settings"),
        // Mechanical ventilation flag
        Observation::new("SNOMED:243144002", serde_json::json!(true))
            .with_source("Ventilator active"),
        // Kidney function observations (KDIGO input)
        // Baseline Cr = 1.0 mg/dL (normal)
        // Current Cr = 2.4 mg/dL
        // → fold-change = 2.4 / 1.0 = 2.4×
        // → KDIGO AKI Stage 2 (2.0–2.9× baseline)
        Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0))
            .with_unit("mg/dL")
            .with_source("Admission labs (2 days ago)"),
        Observation::new("LOINC:2160-0-current", serde_json::json!(2.4))
            .with_unit("mg/dL")
            .with_source("This morning's labs (2026-06-06 08:00)"),
        // Note: Urine output data is available but we exclude it here because
        // the KDIGO operator requires validation of the temporal window (6-24h).
        // In clinical practice, the substrate would either:
        // (a) Include metadata about when the UO window was collected, or
        // (b) Let the operator abstain and allow a different evidence capture pathway
    ];

    let provenance = Provenance::new(
        ProvenanceOrigin::new("epic_lms", "LOINC", "2703-7"),
        Utc::now(),
        Ver::new("clinlat", "sofa_resp", "0.2.0"),
        BTreeMap::new(),
    );

    let evidence = Evidence::new(observations, provenance);

    println!("Evidence collected:");
    println!("  • PaO₂ = 150 mmHg, FiO₂ = 0.60");
    println!("    → PaO₂/FiO₂ ratio = 250 (SOFA respiratory input)");
    println!("  • On mechanical ventilation: YES");
    println!("  • Baseline Cr = 1.0 mg/dL, Current Cr = 2.4 mg/dL");
    println!("    → fold-change = 2.4× (KDIGO Stage 2 trigger)");
    println!("  • [UO data available but excluded per operator temporal constraints]\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 2: Create Operators
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 2: Instantiate Operators");
    println!("─────────────────────────────\n");

    let sofa_op = SofaRespOperator::default_v0_2();
    let kdigo_op = KdigoAkiOperator::new("0.2.0");

    println!("Operators configured:");
    println!("  1. SofaRespOperator v0.2.0");
    println!("     Maps PaO₂/FiO₂ ratio to respiratory severity (0–4)");
    println!("  2. KdigoAkiOperator v0.2.0");
    println!("     Maps Cr fold-change + urine output to AKI stage (0–3)\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 3: Build OperatorSet and Proposer
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 3: Build OperatorSet and LatticeSearchProposer");
    println!("─────────────────────────────────────────────────────\n");

    let op_set = OperatorSet::new()
        .register(
            Box::new(sofa_op),
            OperatorMetadata {
                name: "sofa_resp".to_string(),
                version: "clinlat-v0.2.0".to_string(),
            },
        )
        .register(
            Box::new(kdigo_op),
            OperatorMetadata {
                name: "kdigo_aki".to_string(),
                version: "clinlat-v0.2.0".to_string(),
            },
        );

    let proposer = LatticeSearchProposer::new(op_set);

    println!("OperatorSet built with 2 operators:");
    println!("  • sofa_resp (respiratory dysfunction scoring)");
    println!("  • kdigo_aki (acute kidney injury staging)\n");
    println!("LatticeSearchProposer ready to generate candidates.\n");

    // ─────────────────────────────────────────────────────────────────────
    // STEP 4: Call the Proposer on the Unknown Hypothesis
    // ─────────────────────────────────────────────────────────────────────

    println!("STEP 4: Call Proposer on Unknown Hypothesis");
    println!("──────────────────────────────────────────\n");

    let input = Hyp::unknown();

    println!("Input hypothesis: Unknown (most general)");
    println!("Calling: proposer.propose(&Hyp::unknown(), &evidence)\n");

    let candidates = proposer.propose(&input, &evidence);

    // ─────────────────────────────────────────────────────────────────────
    // STEP 5: Inspect Candidate Set
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("CANDIDATES GENERATED:");
    println!("═══════════════════════════════════════════════════════════════\n");

    if candidates.is_empty() {
        println!("⚠ No candidates generated (operators abstained or returned no refinements)");
    } else {
        println!("✓ {} candidate(s) generated:\n", candidates.len());

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
    // STEP 6: Interpret Results
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("INTERPRETATION:");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("What the proposer found:");
    println!("──────────────────────────");
    println!();
    println!("Expected outcomes (given evidence):");
    println!();
    println!("  1. SOFA Respiratory Score 2:");
    println!("     PaO₂/FiO₂ = 250 falls in the 200–299 range");
    println!("     → Moderate respiratory dysfunction");
    println!("     → This is a valid refinement from Unknown");
    println!();
    println!("  2. KDIGO AKI: Creatinine-based staging");
    println!("     Current Cr = 2.4 mg/dL, Baseline = 1.0 mg/dL");
    println!("     Fold-change = 2.4× (2.0–2.9× range → Stage 2)");
    println!("     Urine output excluded due to temporal window constraints");
    println!();
    println!("Why both operators produce candidates:");
    println!("────────────────────────────────────");
    println!("  • LatticeSearchProposer applies each operator to the input");
    println!("  • Both operators find sufficient evidence to refine");
    println!("  • The proposer returns the UNION of all refinements");
    println!("  • Each candidate represents a single-step refinement");
    println!();
    println!("Soundness property (DEF-PS-15):");
    println!("──────────────────────────────");
    println!("  Each candidate is trivially sound by construction:");
    println!("  • It was produced by exactly one operator");
    println!("  • When passed through the soundness gate (propose_verify),");
    println!("    the gate will recognize it via OperatorSet.apply_set()");
    println!("  • The proposer cannot generate invalid candidates because");
    println!("    it only collects operator-produced refinements");
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // STEP 7: Show Multi-Step Refinement (Propagation)
    // ─────────────────────────────────────────────────────────────────────

    println!("═══════════════════════════════════════════════════════════════");
    println!("MULTI-STEP REFINEMENT (Future Iteration):");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("If we continued the refinement process:");
    println!("  1. Take a candidate (e.g., SOFA Score 2 hypothesis)");
    println!("  2. Call proposer again with the same evidence");
    println!("  3. The proposer would attempt further refinements");
    println!("     (though in this case, operators typically produce");
    println!("      a single stage/score and return self-identity,");
    println!("      which LatticeSearchProposer filters out)");
    println!();
    println!("The substrate ensures:");
    println!("  • No spurious candidates (proposer returns operator-reachable only)");
    println!("  • Monotone refinement (each step refines or stays same)");
    println!("  • Exhaustiveness (for bounded operator sets, all reachable");
    println!("    hypotheses are eventually enumerated)");
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!("END OF EXAMPLE");
    println!("═══════════════════════════════════════════════════════════════");
}
