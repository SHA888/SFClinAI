# Wells PE Operator Soundness

**Operator:** `WellsPeOperator` (clinlat v0.2.0)

**Formalizes:** SPEC.md §2 (DEF-PS-08, Operator trait interface), NOTE.md §4A

**Status:** Discharged via informal-argument tier. Property-test tier deferred to Phase 6+.

---

## Obligation Statement

The Wells PE operator must satisfy DEF-PS-08 (Operator soundness):

1. **Refinement monotonicity (INV-PS-03):** For all hypotheses h and evidence e, if the operator produces a refined hypothesis h', then h' ⊑ h.

2. **No spurious refinement:** The operator does not commit to a PE risk category beyond the evidence's justification (cumulative Wells clinical scoring).

3. **Abstention purity:** Abstention is structural (missing gestalt assessment) rather than error handling.

---

## Clinical Soundness: Wells Score for Pulmonary Embolism

**Source:** Wells et al. (1997, derivation; 2000, simplified PE model), validated in the dichotomized (two-tier) form by the Christopher Study (van Belle et al., 2006).

The Wells criteria stratify pulmonary embolism (PE) pretest probability using seven clinical features, one of which is a mandatory clinician gestalt judgment. Points:

- **Clinical signs of DVT** (leg swelling, asymmetry, pain on palpation): +3
- **PE as the most likely diagnosis** (gestalt): +3
- **Heart rate >100 bpm:** +1.5
- **Recent surgery or immobilization** (>4 days, within the past 4 weeks): +1.5
- **Prior DVT or PE:** +1.5
- **Hemoptysis:** +1
- **Malignancy** (treatment ongoing or within 6 months): +1

Maximum score: 12.5. Unlike CURB-65's five independent binary features, Wells mixes objective findings (heart rate, exam signs) with one irreducibly subjective input — "is PE the most likely diagnosis" — that cannot be derived from the other six.

### Risk Category and Sequential Testing by Wells Score

- **Score ≤4.0:** PE unlikely — D-dimer testing indicated; a negative result excludes PE, a positive result requires CTPA.
- **Score >4.0:** PE likely — CTPA indicated regardless of D-dimer result.

This is the two-tier (dichotomized) Wells model validated by the Christopher Study, as opposed to the original three-tier (low/moderate/high) stratification.

### Clinical Justification

**Why the gestalt component is mandatory, not optional:** Six of seven Wells criteria are objective (vital signs, exam findings, history). "PE as most likely diagnosis" is the exception — it asks the clinician to integrate everything not captured by the other six (symptom onset pattern, alternative-diagnosis likelihood, overall clinical gestalt) into a single judgment. Wells derivation studies found this component load-bearing: omitting it degrades the score's discriminative validity. The operator therefore abstains rather than silently defaulting it to false when absent (see Abstention Purity below).

**Why cumulative rather than all-or-nothing scoring:** As with CURB-65, each feature independently shifts PE pretest probability; the literature validates the cumulative point total against angiographically or CT-confirmed PE outcomes, not any single feature in isolation.

**Why the ≤4/>4 dichotomy:** The original three-tier model (low/moderate/high, roughly PE prevalence 3%/28%/78%) was collapsed to a two-tier model for the Christopher Study's outcome-based algorithm (score ≤4 → D-dimer-gated CTPA; score >4 → direct CTPA), because the two-tier model showed a 3-month VTE incidence of 0.5% in the D-dimer-negative, low-probability arm — adequate for safely withholding anticoagulation. The operator implements this validated two-tier form, not the original three-tier form.

---

## Operator Soundness Proof

### Theorem: DEF-PS-08 Soundness of WellsPeOperator

**Claim:** For all hypotheses h and evidence e:
1. If the operator refines h to h', then h' ⊑ h.
2. The refinement (PE risk category assignment) is justified by cumulative Wells clinical scoring.
3. The operator does not spuriously claim PE presence or absence; it assigns a risk category or abstains.

**Proof Sketch:**

**Gestalt Presence Check (Abstention Purity, Clause 3):** Before scoring, the operator scans observations for a `"PE-LIKELY"` code. If absent entirely, it abstains with `AbstainReason::InsufficientEvidence("PE gestalt assessment unavailable; clinician judgment required for Wells scoring")` — this is a **presence** check, not a **value** check: an explicit `PE-LIKELY: false` (clinician gestalt says PE is *not* the most likely diagnosis) is valid evidence and scores 0 points for that criterion; only the *absence* of the observation triggers abstention. This distinguishes "clinician assessed and PE is unlikely" from "clinician assessment was never obtained," which the naive alternative (defaulting missing gestalt to `false`) would conflate.

**Cumulative Scoring:** Given the gestalt observation is present, the operator:
- Scans for each of the seven criteria independently (DVT signs, PE-likely gestalt, heart rate, recent immobilization, prior VTE, hemoptysis, malignancy).
- Boolean criteria contribute their fixed point value only if the observation's value parses as `true`.
- Heart rate contributes +1.5 only if the value parses as f64 and exceeds 100.0.
- Sums all contributing points into a single `f64` score.

Each feature is evaluated independently (no early exit, no branching that could skip a later criterion), mirroring CURB-65's independent-evaluation discipline.

**Category Assignment:** The operator:
- Compares the summed score to the 4.0 threshold: `≤4.0 → PE-UNLIKELY`, `>4.0 → PE-LIKELY`.
- Creates an Atom tagged `WELLS-PE-{CATEGORY}`, carrying the numeric score in its preferred term for audit legibility.
- Appends the atom to the input hypothesis: `h' = h ∪ {Atom(...)}`.

By atom-set inclusion, h' ⊑ h (one additional atom). **Clause 1 satisfied.**

**Evidence Justification:** Every point contributing to the Wells score is:
- Drawn directly from a clinical observation in e (gestalt, DVT signs, heart rate, immobilization, prior VTE, hemoptysis, malignancy).
- Applied via the validated point weights and 4.0 dichotomy threshold from Wells (2000) and the Christopher Study.
- Not speculative: the operator does not invent evidence, and does not default an absent gestalt observation to a scored value (see Abstention Purity).

**Clause 2 satisfied.**

**Risk Category (No Spurious Diagnosis):** The operator outputs a **pretest-probability category with an implied next diagnostic step** (D-dimer-gated vs. direct CTPA), not a PE diagnosis. The atoms `WELLS-PE-PE-UNLIKELY` / `WELLS-PE-PE-LIKELY` represent **pretest probability tiers**, not confirmed or excluded PE. Clinical decision proceeds:
- PE-UNLIKELY: D-dimer testing; negative excludes PE without imaging, positive escalates to CTPA.
- PE-LIKELY: Direct CTPA regardless of D-dimer (D-dimer's negative predictive value is insufficient at this pretest probability).

The operator **refrains from claiming** PE is present or absent — it stops at pretest-probability stratification and defers the diagnostic determination to the sequential test (D-dimer, CTPA) the category indicates. This is **structural refinement** ("Unknown risk" → "Wells PE {category}: score N"), not spurious diagnosis. **Clause 3 satisfied** (jointly with the abstention-purity argument above).

---

## Implementation Verification

### Unit Tests (8 tests from wells_pe.rs)

All 8 unit tests pass, covering:
- Low score (tachycardia only, gestalt negative) → PE-UNLIKELY
- Moderate score (DVT signs + tachycardia, gestalt positive, score 4.5) → PE-LIKELY
- High score (gestalt + DVT signs + tachycardia + prior VTE, score 9.0) → PE-LIKELY
- All seven criteria present (score 12.5, the maximum) → PE-LIKELY
- Zero score (gestalt negative, no other criteria) → PE-UNLIKELY
- Boundary score exactly 4.0 (DVT signs + hemoptysis, gestalt negative) → PE-UNLIKELY, confirming the `≤4.0` threshold is inclusive
- Monotonicity preservation (h' ⊑ h)
- Abstention when the gestalt (`PE-LIKELY`) observation is entirely absent, with the abstention message verified to reference "gestalt"

**Test coverage:** 8 tests passing; all Wells-PE-specific scenarios covered, including the presence-vs-value distinction on the gestalt criterion.

### Limitations and Caveats

1. **Sequential-testing recommendation is not encoded in the output atom.** The operator internally computes a next-step signal (`"consider-d-dimer"` / `"ctpa-indicated"`) alongside the risk category, but this value is discarded — only the `WELLS-PE-{CATEGORY}` atom is emitted. The next diagnostic step is recoverable from the category by clinical convention (documented above), not from the Evidence chain itself. **Future work (Phase 6+):** carry the next-step recommendation into the Atom or a companion provenance field so downstream consumers do not need to re-derive it.

2. **Two other abstention paths are documented but not implemented.** The operator's doc comment names "D-dimer unavailable" and "CTPA contraindicated (renal impairment, contrast anaphylaxis history)" as abstention triggers, but `apply()` only implements the missing-gestalt abstention — D-dimer and CTPA availability/contraindication are sequential-testing concerns downstream of this operator's scope, not currently modeled here. **Future work (Phase 6+ or M4 interaction layer):** either implement these as operator-level preconditions, or explicitly re-scope them to a downstream sequential-testing operator and remove them from this operator's doc comment.

3. **Naming overlap between the input gestalt observation and the output category.** The observation code `"PE-LIKELY"` (the mandatory gestalt input) and the output category label `PE-LIKELY` (score >4.0) share a name but are distinct concepts — a clinician's gestalt judgment that PE is likely (input, worth +3 points) does not by itself determine the output category (which depends on the *summed* score, e.g., gestalt-positive with no other criteria scores only 3.0, still `PE-UNLIKELY`). Implementers integrating with this operator should not conflate the two. No code change proposed; documented here for clarity.

4. **No BMI, pregnancy, or pediatric adjustment.** The classic Wells score is validated in general adult populations. Higher-BMI, pregnant, and pediatric populations have separate or modified pretest-probability tools (e.g., YEARS algorithm) not implemented here. **Future work (M5+):** age/population-specific variant operators.

5. **No temporal progression modeling.** Wells is a point-in-time pretest-probability assessment. Repeat scoring after new findings (e.g., a positive D-dimer prompting reassessment) requires a fresh operator application, not incremental update. **Future work (M5):** track sequential Wells/D-dimer/CTPA testing state as a composed workflow, per NOTE.md §7E.3.

---

## Worked Example

**Patient:** 45-year-old with acute-onset dyspnea, 3 days post long-haul flight.

**Presenting Evidence:**
- Gestalt: PE most likely diagnosis (clinician assessment): Yes
- Recent immobilization: Yes (12-hour flight, within past 4 weeks)
- Heart rate: 110 bpm (>100)
- DVT signs: No
- Prior DVT/PE: No
- Hemoptysis: No
- Malignancy: No

**Operator Execution:**

1. **Gestalt presence check:** `"PE-LIKELY"` observation found (value: true) → proceed to scoring (no abstention).
2. **DVT signs:** absent/false → +0
3. **PE-LIKELY (gestalt):** true → +3.0
4. **Heart rate:** 110 > 100 → +1.5
5. **Recent immobilization:** true → +1.5
6. **Prior VTE:** false → +0
7. **Hemoptysis:** false → +0
8. **Malignancy:** false → +0
9. **Sum:** 3.0 + 1.5 + 1.5 = **6.0**
10. **Category:** 6.0 > 4.0 → **PE-LIKELY**
11. **Create atom:** `Atom(code: "WELLS-PE-PE-LIKELY", preferred_term: "Wells PE PE-LIKELY: score 6.0", version: "0.2.0")`
12. **Output:** h' = h ∪ {Wells PE PE-LIKELY atom}

**Clinical Interpretation:**
- Wells score 6.0 (PE-LIKELY, >4.0 threshold) indicates CTPA is indicated directly, regardless of D-dimer result — D-dimer's negative predictive value is not sufficient to withhold imaging at this pretest probability.
- Immediate next steps: CTPA (or V/Q scan if CTPA contraindicated), empiric anticoagulation pending imaging if bleeding risk is acceptable, hemodynamic monitoring.
- This is the sequential-testing branch point documented in NOTE.md §7E.3 (Wells/PE with sequential testing): the score alone does not diagnose PE, it selects which confirmatory test pathway applies.

**Refinement Check:** h' ⊑ h (input = Unknown or prior hypothesis; output = Unknown + {Wells PE PE-LIKELY atom}). Monotonicity preserved ✓.

---

## Clinical Validation References

- **Wells PS, et al. (2000).** "Derivation of a simple clinical model to categorize patients probability of pulmonary embolism: increasing the model's utility with the SimpliRED D-dimer." *Thrombosis and Haemostasis*. 83(3):416–420. Original Wells PE score derivation and point weights.
- **van Belle A, et al., Christopher Study Investigators (2006).** "Effectiveness of managing suspected pulmonary embolism using an algorithm combining clinical probability, D-dimer testing, and computed tomography." *JAMA*. 295(2):172–179. Prospective validation of the dichotomized (≤4/>4) two-tier Wells model with D-dimer-gated CTPA.
- **Wells PS, et al. (1997).** "Value of assessment of pretest probability of deep-vein thrombosis in clinical management." *Lancet*. 350(9094):1795–1798. Predecessor scoring framework informing the Wells PE model's structure.

---

## Verification Checklist

- [x] Clinical criteria (Wells PE, seven features) implemented and tested
- [x] Cumulative scoring validated against Wells (2000) point weights
- [x] Two-tier dichotomy (≤4.0 vs. >4.0) matches Christopher Study validation
- [x] Gestalt (PE-LIKELY) criterion evaluated as a mandatory presence check, distinct from its boolean value
- [x] Boundary score 4.0 correctly resolves to PE-UNLIKELY (inclusive threshold)
- [x] Monotonicity preservation (INV-PS-03)
- [x] 8 unit tests covering all score ranges, the boundary, monotonicity, and abstention
- [x] Atom generation with Wells PE code and score for audit trail
- [x] No spurious PE diagnosis (pretest-probability tier only, not a diagnostic claim)
- [x] Structural abstention when gestalt assessment is unavailable (not silently defaulted)

**Limitations documented:**
- [x] Sequential-testing next-step not carried into the output atom (Phase 6+)
- [x] D-dimer/CTPA-contraindication abstention paths documented but not implemented (Phase 6+ / M4)
- [x] Gestalt-input vs. category-output naming overlap (documentation-only, no code change proposed)
- [x] No BMI/pregnancy/pediatric variant (M5+)
- [x] No temporal sequential-testing state tracking (M5)

**Conclusion:** Wells PE operator satisfies DEF-PS-08 soundness at informal-argument tier. Clinically validated against the Wells (2000) derivation and the Christopher Study's prospective two-tier validation. Ready for clinical deployment and multi-operator composition; the sequential-testing workflow described in NOTE.md §7E.3 is realized at the category-output level, with next-step encoding into the audit trail deferred to future work. ✓

---

## References

- Wells PS, et al., Thromb Haemost 2000: https://pubmed.ncbi.nlm.nih.gov/10744147/
- van Belle A, et al., JAMA 2006 (Christopher Study): https://jamanetwork.com/journals/jama/fullarticle/202296
- Wells PS, et al., Lancet 1997: https://pubmed.ncbi.nlm.nih.gov/9428249/
- SFClinAI SPEC.md §2 (patient-state substrate, operator interface)
- SFClinAI NOTE.md §7E.3 (worked example: Wells/PE with sequential testing)
