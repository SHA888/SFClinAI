# Substrate-First Clinical AI: Architecture Diagrams

**Version:** v0.2.0-draft
**Date:** 2026-05-25
**Status:** Working draft for scrutiny. Companion to `NOTE.md` v0.12.0-draft and `SPEC.md` v0.3.0-draft.
**License:** CC BY 4.0

---

## Purpose

These diagrams visualize the architecture formalized in `SPEC.md` (v0.3.0-draft) and described in clinical prose in `NOTE.md` (v0.12.0-draft). They are not a replacement for either; they are an aid to comprehension.

**Resolution order for conflicts** (per `CLAUDE.md` and `SPEC.md` §0.1, §9.3): `NOTE.md` wins if `SPEC.md` and `NOTE.md` disagree; `SPEC.md` wins if a diagram and `SPEC.md` disagree. The diagrams render `SPEC.md`'s typed structures (Galois connections, operator sets, joint-licensing decomposition, re-review events for both substrates) rather than re-rendering `NOTE.md`'s prose-level commitments. Where a diagram and either document disagree, the diagram is wrong.

Five diagrams, each addressing a primary view the others cannot show:

1. **Component decomposition** — what's in the system, organized by the four architectural axes (4A, 4B, 4C, 4D).
2. **Data and event flow** — how a clinical input traverses the system and produces an output.
3. **Substrate-learned-component boundary** — the interface contract that makes learned components safe by construction.
4. **Temporal-evolution lifecycle** — how substrate components evolve over deployment time (months to years).
5. **Learned-component composition (expansion of 3)** — the enumerated set of task-specific learned and symbolic-proposer components that plug into the substrate's interfaces.

---

## Visual conventions (apply across all five diagrams)

| Visual element | Architectural meaning |
| --- | --- |
| Rectangle | Component (substrate element, learned component, infrastructure) |
| Rounded rectangle | External system (EHR, clinical ontology, guideline source, regulatory infrastructure) |
| Diamond | Decision point where abstention can fire |
| Solid arrow | Synchronous call or required relationship |
| Dashed arrow | Event or asynchronous notification |
| Bold border | Component that enforces an architectural safety property (substrate-as-safety-boundary) |
| Dashed border | Research-question component, not yet ready to build |
| Color: blue | Substrate (symbolic infrastructure) |
| Color: orange | Learned components (probabilistic, ML/AI models) |
| Color: green | Symbolic proposers (operations-research solvers, deterministic algorithms — propose moves like learned components but are not learned) |
| Color: gray | External systems and infrastructure |
| Color: red (border) | Abstention output or safety-critical gate |
| Label on arrow | What flows along that relationship (data type, event type, return type) |

Mermaid's class/style support is used to encode color. Bold and dashed borders are encoded via stroke-width and stroke-dasharray respectively.

---

## Principle-to-formalization map

Each diagram below visualizes structures that `SPEC.md` v0.3.0 formalizes against `NOTE.md` v0.12.0's eighteen load-bearing principles. The table records, per `NOTE.md` §4 principle: criticality tier (P/S/F from `NOTE.md` §4 intro), the `SPEC.md` formal anchors (DEF/INV/OBL), and the diagrams in which the structure is visible.

| `NOTE.md` principle | Criticality | `SPEC.md` anchors | Visible in |
| --- | --- | --- | --- |
| 4A.1 Persistent patient state | P | DEF-PS-01, DEF-PS-02, DEF-PS-05, DEF-PS-06; OBL-PS-02 | D1, D2 |
| 4A.2 Justified reversible transitions; ontology-bounded | S | DEF-PS-03/04/07/08/09; INV-PS-01/02/03; OBL-PS-01/03 | D1, D2, D3 |
| 4A.3 First-class abstention | S | DEF-PS-10/11; INV-PS-04 | D1, D2, D3 |
| 4A.4 Auditable provenance | S | DEF-PS-12/13; INV-PS-05; OBL-PS-04 | D1, D2 |
| 4A.5 Constrained refinement proposer | S | DEF-PS-14/15; INV-PS-06; OBL-PS-05 | D3, D5 |
| 4B.1 Persistent institutional state | P | DEF-IS-01/02/04/05/06; OBL-IS-03 | D1, D2 |
| 4B.2 Justified capacity transitions; resource-bounded | S | DEF-IS-03/07/08/09; INV-IS-01/02/03; OBL-IS-01/02/04 | D1, D2, D3 |
| 4B.3 Allocation abstention | S | DEF-IS-10/11; INV-IS-04 | D1, D2 |
| 4B.4 Auditable allocation provenance | S | DEF-IS-12/13; INV-IS-05; OBL-IS-05 | D1, D2 |
| 4B.5 Constrained capacity-learned proposers | S | DEF-IS-14/15; INV-IS-06; OBL-IS-06 | D3, D5 |
| 4C.1 Joint licensing | P | DEF-IX-03/04/05/06/07; INV-IX-02; OBL-IX-02 | D1, D2 |
| 4C.2 Cross-layer events | F | DEF-IX-01/02; INV-IX-01; OBL-IX-01 | D1, D2 |
| 4C.3 Joint abstention (incl. structured diff DEF-IX-09) | P | DEF-IX-08/09/10; INV-IX-03; OBL-IX-03 | D1, D2 |
| 4D.1 Versioned substrate components | F | DEF-TE-01; INV-TE-01 | D1, D4 |
| 4D.2 Evidence currency | F | DEF-TE-02/03; INV-TE-02 | D1, D4 |
| 4D.3 Sound evolution under operator-set change | S | DEF-TE-04/05; INV-TE-03; OBL-TE-01 | D4 |
| 4D.4 Clinician-mediated propagation (+ institutional symmetry) | P | DEF-TE-06, **DEF-TE-06b**; INV-TE-04; OBL-TE-02 | D4 |
| 4D.5 Evolution-aware provenance | S | DEF-TE-07; INV-TE-05; OBL-TE-03 | D1, D4 |

**Criticality vocabulary** (`NOTE.md` §4 intro):

- **P** — Position-critical. Violation falsifies a substrate/layer/coupling claim the position itself depends on.
- **S** — Safety-property. Violation breaks a stated safety property within an existing substrate.
- **F** — Foundation. Violation breaks supporting infrastructure other principles depend on.

Tier counts in §4: 5 P, 10 S, 3 F (matches `SPEC.md` §6.5 obligation distribution at 5 P / 10 S / 2 F, with the F-asymmetry on 4D.1/4D.2 documented as OQ-X-01).

`INV-IX-04` (substrate-local soundness preserved under coupling) is a cross-cutting structural invariant that does not anchor to a single principle; it formalizes the §4C intro claim that "the two substrates are coupled but not collapsed." Visible implicitly in Diagrams 1 and 3 as the architectural separation of the patient-substrate and institutional-substrate columns.

---

## Diagram 1: Component decomposition

The architectural skeleton, organized by the four axes from position note Section 4 plus cross-cutting concerns. Top-level only — sub-components within the "Learned components zone" are expanded in Diagram 5.

```mermaid
graph TB
    subgraph EXT["External systems"]
        EHR["EHR / clinical data sources"]
        ONT["Clinical ontologies<br/>SNOMED CT, RxNorm, LOINC, ICD-11"]
        GUIDE["Guideline sources<br/>SSC, KDIGO, IDSA/ATS, CHEST"]
        STD["Standards infrastructure<br/>HL7 FHIR CR, CQL, MAGICapp"]
        REG["Regulatory infrastructure<br/>FDA PCCP, EU AI Act Art. 72"]
    end

    subgraph A4["4A — Patient-state substrate"]
        PL["Patient lattice<br/>(candidate sets, partial order)"]
        PDO["Patient deduction operators<br/>(SOFA, Wells, KDIGO, CURB-65, ...)"]
        PSB["Search & backtrack procedure"]
        PAB["Patient abstention semantics"]
        PPR["Patient provenance ledger"]
        PLC_ZONE["Patient learned components zone<br/>(see Diagram 5 for enumeration)"]
    end

    subgraph B4["4B — Institutional-state substrate"]
        IL["Institutional lattice<br/>(resource state, feasibility partial order)"]
        IDO["Capacity-update operators<br/>(admit, discharge, dispense, deliver, ...)"]
        IAP["Allocation policy layer<br/>(priority, equity, scarcity ethics)"]
        IAB["Allocation abstention semantics"]
        IPR["Institutional provenance ledger"]
        ILC_ZONE["Institutional learned + symbolic-proposer zone<br/>(see Diagram 5 for enumeration)"]
    end

    subgraph C4["4C — Interaction semantics"]
        JL["Joint licensing gate"]
        XE["Cross-layer event bus<br/>(typed, provenanced, bidirectional)"]
        JAB["Joint abstention output"]
    end

    subgraph D4["4D — Temporal evolution"]
        VR["Version registry<br/>(operator-version pairs, effective period, status)"]
        CC["Currency monitor<br/>(staleness signal, advisory/inactive downgrades)"]
        SE["Sound-evolution checker<br/>(add/modify/retire semantics)"]
        CMP["Clinician-mediated propagation<br/>(re-review event generator)"]
        EAP["Evolution-aware provenance"]
    end

    subgraph CC_CONCERNS["Cross-cutting concerns"]
        AUD["Audit infrastructure<br/>(unified across 4A, 4B, 4D)"]
        OB["Ontology binding<br/>(used across 4A, 4B, 4D)"]
        CI["Clinician interface<br/>(both layers, diff display, override capture)"]
        CMI["Capacity-management interface<br/>(administrator-facing)"]
        EH["Evaluation harness<br/>(patient + institutional + joint metrics)"]
        RA["Regulatory artifact production<br/>(PCCP submissions, Art. 72 reports)"]
    end

    EHR ==> PL
    ONT ==> OB
    OB ==> PL
    OB ==> IL
    GUIDE ==> VR
    STD ==> VR
    REG ==> RA

    PL <==> PDO
    PDO <==> PSB
    PSB --> PAB
    PL --> PPR
    PLC_ZONE -.proposes refinement.-> PL

    IL <==> IDO
    IDO <==> IAP
    IAP --> IAB
    IL --> IPR
    ILC_ZONE -.proposes refinement.-> IL

    PL <-.cross-layer events.-> XE
    IL <-.cross-layer events.-> XE
    XE --> JL
    JL --> JAB

    VR --> PDO
    VR --> IDO
    CC --> VR
    SE --> VR
    CMP -.re-review events.-> CI
    EAP --> PPR
    EAP --> IPR

    PPR --> AUD
    IPR --> AUD
    JL --> CI
    JAB --> CI
    IAB --> CMI
    AUD --> RA

    classDef substrate fill:#cfe2ff,stroke:#0d6efd,stroke-width:2px,color:#000
    classDef learnedZone fill:#ffe5b4,stroke:#fd7e14,stroke-width:2px,stroke-dasharray:5 5,color:#000
    classDef external fill:#e2e3e5,stroke:#6c757d,stroke-width:1px,color:#000
    classDef safetyGate fill:#cfe2ff,stroke:#dc3545,stroke-width:3px,color:#000
    classDef crossCutting fill:#d1e7dd,stroke:#198754,stroke-width:1px,color:#000

    class PL,PDO,PSB,PPR,IL,IDO,IAP,IPR,XE,VR,CC,SE,EAP substrate
    class PAB,IAB,JL,JAB,CMP safetyGate
    class PLC_ZONE,ILC_ZONE learnedZone
    class EHR,ONT,GUIDE,STD,REG external
    class AUD,OB,CI,CMI,EH,RA crossCutting
```

**Reading this diagram.** Five zones, four architectural axes plus cross-cutting concerns. The patient-state substrate (4A, blue) and institutional-state substrate (4B, also blue) are the two coupled lattices the position note's Section 5 names. The interaction semantics (4C, red borders) sit between them, carrying joint licensing, cross-layer events, and joint abstention. Temporal evolution (4D, blue, with red borders on the propagation event generator) sits to the side and feeds versioning information into both substrates' deduction operators. External systems (gray, rounded) flow into the architecture from the top. Cross-cutting concerns (green) span all four axes and produce outputs to clinicians, administrators, evaluators, and regulators.

Learned components are deliberately shown as opaque "zones" at this level (orange, dashed-border-for-research-question). The expansion is in Diagram 5. The position note (4A.5, 4B.5) supports this multi-component reading; Diagram 5 enumerates the specific learned components and symbolic proposers that plug into each substrate's interface.

---

## Diagram 2: Data and event flow

How a clinical input traverses the system and produces an output. Dynamic view, single-encounter timescale (minutes to hours). The substrate-learned-component boundary is visible but not zoomed; Diagram 3 zooms in on it.

```mermaid
flowchart TB
    CIN["Clinical input<br/>(new lab, vital, finding, override)"]
    EHR_IN["From EHR / data source"]

    OG["Ontology binding gate<br/>(SNOMED/RxNorm/LOINC/ICD-11)"]
    PL_UPD["Patient lattice<br/>state update"]
    PROPOSE_P["Patient learned component(s)<br/>propose refinement(s)"]
    SOUND_P["Soundness verification<br/>(deduction operator licensing)"]
    REJ_P{"Sound?"}
    AB_P{"Abstention<br/>triggered?"}
    PROV_P["Provenance ledger entry<br/>(operator-version pair, evidence, authority)"]

    EVT_OUT["Cross-layer event<br/>'patient needs X'"]
    IL_UPD["Institutional lattice<br/>state update"]
    PROPOSE_I["Institutional learned/symbolic<br/>proposer(s)"]
    FEAS_I["Feasibility verification<br/>(capacity-update operator licensing)"]
    REJ_I{"Feasible?"}
    AB_I{"Allocation<br/>abstention?"}
    PROV_I["Institutional provenance ledger entry"]

    JL_GATE["Joint licensing gate<br/>(patient sound AND institutional feasible)"]
    DIFF["Diff generator<br/>(unconstrained-optimal vs achievable)"]
    JABS{"Joint abstention?"}

    CIN_OUT["Recommendation pair<br/>(unconstrained-optimal + achievable, with diff)"]
    PABS_OUT["Patient abstention output<br/>(escalation)"]
    IABS_OUT["Allocation abstention output<br/>(capacity manager)"]
    JABS_OUT["Joint abstention output<br/>(human-only judgment)"]

    AUD["Audit trail<br/>(unified)"]

    EHR_IN --> CIN
    CIN --> OG
    OG --> PL_UPD
    PL_UPD --> PROPOSE_P
    PROPOSE_P --> SOUND_P
    SOUND_P --> REJ_P
    REJ_P -- "rejected" --> AB_P
    REJ_P -- "accepted" --> PROV_P
    AB_P -- "yes" --> PABS_OUT
    AB_P -- "no, retry" --> PROPOSE_P

    PL_UPD -.event.-> EVT_OUT
    EVT_OUT --> IL_UPD
    IL_UPD --> PROPOSE_I
    PROPOSE_I --> FEAS_I
    FEAS_I --> REJ_I
    REJ_I -- "rejected" --> AB_I
    REJ_I -- "accepted" --> PROV_I
    AB_I -- "yes" --> IABS_OUT
    AB_I -- "no, retry" --> PROPOSE_I

    PROV_P --> JL_GATE
    PROV_I --> JL_GATE
    JL_GATE --> DIFF
    DIFF --> JABS
    JABS -- "yes" --> JABS_OUT
    JABS -- "no" --> CIN_OUT

    PROV_P --> AUD
    PROV_I --> AUD
    PABS_OUT --> AUD
    IABS_OUT --> AUD
    JABS_OUT --> AUD
    CIN_OUT --> AUD

    classDef substrate fill:#cfe2ff,stroke:#0d6efd,stroke-width:2px,color:#000
    classDef learned fill:#ffe5b4,stroke:#fd7e14,stroke-width:2px,color:#000
    classDef gate fill:#cfe2ff,stroke:#dc3545,stroke-width:3px,color:#000
    classDef output fill:#fff,stroke:#0d6efd,stroke-width:2px,color:#000
    classDef external fill:#e2e3e5,stroke:#6c757d,stroke-width:1px,color:#000
    classDef absOutput fill:#fff,stroke:#dc3545,stroke-width:3px,color:#000

    class PL_UPD,IL_UPD,EVT_OUT,DIFF,PROV_P,PROV_I,AUD substrate
    class PROPOSE_P,PROPOSE_I learned
    class OG,SOUND_P,FEAS_I,JL_GATE,REJ_P,REJ_I,AB_P,AB_I,JABS gate
    class CIN_OUT output
    class PABS_OUT,IABS_OUT,JABS_OUT absOutput
    class CIN,EHR_IN external
```

**Reading this diagram.** A clinical input enters from the EHR, passes through the ontology-binding gate (rejected if outside SNOMED/RxNorm/LOINC/ICD-11), updates the patient lattice state, and triggers learned-component proposal of a refinement. The refinement is verified against deduction operators (with currency tag from 4D's version registry); unsound proposals trigger retry or abstention. Sound proposals are logged to provenance with operator-version pair and evidence. A cross-layer event fires to the institutional lattice, which runs an analogous flow on the institutional side: capacity-update operator licensing, allocation abstention if infeasible, provenance with attribution. Both provenance entries reach the joint-licensing gate; the diff generator produces the unconstrained-optimal-versus-achievable pair if both substrates license, or joint abstention fires if neither does. All outputs feed the unified audit trail. Three abstention types (patient, allocation, joint) are first-class outputs with distinct escalation semantics per Section 4A.3, 4B.3, 4C.

The diagram does not show the temporal-evolution lifecycle (Diagram 4 covers that, on a different timescale) nor the internal structure of the learned-component zones (Diagram 5 covers that). The learned components appear here as single boxes; the boundary contract they satisfy is what Diagram 3 zooms in on.

---

## Diagram 3: Substrate-learned-component boundary

The interface contract that makes the substrate-first commitment work. This is the diagram that most directly captures the architecture's central claim: the substrate's safety properties do not depend on which specific model sits at the interface, as long as the interface contract is satisfied.

Shown here for one substrate-side interface; the same contract shape applies to every substrate-learned-component boundary in the system.

```mermaid
flowchart LR
    subgraph SUB["Substrate side (4A or 4B)"]
        STATE["Lattice state<br/>(current candidate sets,<br/>partial order, history)"]
        CTX["Context bundle<br/>(active operators, currency tags,<br/>patient or institutional metadata)"]
    end

    subgraph LCB["Substrate-learned-component boundary"]
        REQ["Refinement request<br/>(typed input contract)"]
        OG_IN["Input-side ontology gate<br/>(only ontology-bounded candidates<br/>are visible to learned component)"]
        RESP["Refinement proposal<br/>(typed output contract)"]
        OG_OUT["Output-side ontology gate<br/>(proposal must name only<br/>ontology-bounded candidates)"]
        SV["Soundness verification<br/>(deduction operator licensing<br/>against cited evidence)"]
        PG["Provenance generator<br/>(operator-version, evidence,<br/>authority, currency-at-decision)"]
        AT["Abstention trigger<br/>(no licensed refinement applies)"]
    end

    subgraph LC["Learned component (one of many — see Diagram 5)"]
        MODEL["Model implementation<br/>(LDT-style / LLM-class / specialized /<br/>operations-research solver / future architecture)"]
    end

    subgraph OUT["Outputs back to substrate"]
        ACCEPT["Accepted refinement<br/>(applied to lattice with provenance)"]
        REJECT["Rejected proposal<br/>(logged, retry or abstention)"]
        ABSTAIN["Abstention<br/>(first-class output)"]
    end

    STATE --> REQ
    CTX --> REQ
    REQ --> OG_IN
    OG_IN --> MODEL
    MODEL --> RESP
    RESP --> OG_OUT
    OG_OUT --> SV
    SV --> ACCEPT
    SV --> REJECT
    SV --> AT
    AT --> ABSTAIN
    ACCEPT --> PG
    PG --> STATE
    REJECT -. retry .-> MODEL

    classDef substrate fill:#cfe2ff,stroke:#0d6efd,stroke-width:2px,color:#000
    classDef learned fill:#ffe5b4,stroke:#fd7e14,stroke-width:2px,color:#000
    classDef boundary fill:#cfe2ff,stroke:#dc3545,stroke-width:3px,color:#000
    classDef output fill:#fff,stroke:#0d6efd,stroke-width:2px,color:#000
    classDef absOutput fill:#fff,stroke:#dc3545,stroke-width:3px,color:#000

    class STATE,CTX substrate
    class MODEL learned
    class REQ,RESP,OG_IN,OG_OUT,SV,PG,AT boundary
    class ACCEPT output
    class REJECT,ABSTAIN absOutput
```

**Reading this diagram.** The substrate sends the learned component a typed request containing the current lattice state and context (active operators, currency tags, patient or institutional metadata). The request passes through an input-side ontology gate: the learned component only sees candidates that are within the substrate's ontology (SNOMED CT, RxNorm, LOINC, ICD-11). The model — which can be LDT-style, LLM-class, specialized for a specific task, or even an operations-research solver on the institutional side — produces a typed proposal. The proposal passes through the output-side ontology gate (proposal must name only ontology-bounded candidates) and then through soundness verification (does any deduction operator with cited evidence license this proposal?). Three outcomes: accepted (refinement applied to lattice with provenance), rejected (logged, may trigger retry or abstention), or abstention (first-class output emitted).

**Why this is the load-bearing diagram.** The substrate's safety properties are encoded in the gates and the verification, not in the model. Swap the model — LDT-style today, LLM-class tomorrow, future architecture in five years — and the safety properties hold as long as the interface contract is satisfied. The architecture's central claim, in visual form: *the substrate constrains the learned component, not the other way around, and this is what allows the learned component to be wrong without the system being unsafe* (position note 4A.5, 4B.5).

The same boundary shape applies to every learned-component slot in the system. Diagram 5 enumerates the slots; this diagram defines what each slot's contract looks like.

---

## Diagram 4: Temporal-evolution lifecycle

How substrate components evolve over deployment time. Different timescale from Diagrams 2 and 3 — months and years rather than minutes and hours. The 7E.6 worked example in the position note instantiates this lifecycle for the SSC 2021 → SSC 2026 transition; this diagram is the general shape that instantiation follows.

```mermaid
flowchart TB
    SRC["External source updates<br/>(SSC 2021 → SSC 2026, KDIGO 2024,<br/>local antibiogram weekly, formulary daily)"]
    LSR["Living systematic review<br/>(Cochrane LSR, MAGICapp,<br/>Australian Living Evidence)"]

    VR["Version registry<br/>(operator-version pairs:<br/>source, version, effective period, status)"]
    CC["Currency monitor<br/>(staleness signal,<br/>next-check-by tracking)"]

    STATE_A["Operator status: active"]
    STATE_ADV["Operator status: advisory<br/>(licenses with currency caveat)"]
    STATE_INACT["Operator status: inactive<br/>(cannot license without re-verification)"]

    SE["Sound-evolution checker<br/>(DEF-TE-04, DEF-TE-05;<br/>add: no rejected→accepted without policy;<br/>modify: track per-recommendation diff;<br/>retire: flag outstanding for re-review;<br/>category: refines / generalizes / incomparable)"]

    AR_PAT["Active patient recommendations<br/>(licensed under prior Δ_PS version)"]
    AR_INST["Active institutional allocations<br/>(licensed under prior Δ_IS version)"]

    CMP_PAT["Clinician-mediated propagation<br/>(DEF-TE-06: ReReviewEvent;<br/>NOTE §4D.4 ¶1)"]
    CMP_INST["Institutional-authority propagation<br/>(DEF-TE-06b: InstReReviewEvent;<br/>NOTE §4D.4 ¶2 — institutional symmetry)"]

    RR_PAT["Patient re-review event<br/>per active hypothesis<br/>(status: Pending → ResolvedKeep / ResolvedReplace)"]
    RR_INST["Institutional re-review event<br/>per active capacity hypothesis<br/>(authority_class: CapacityManager /<br/>EthicsCommittee / FormularyCommittee / ...)"]

    CHOICE_PAT{"Clinician<br/>decision"}
    CHOICE_INST{"Institutional<br/>authority<br/>decision"}

    TRANS["Transition to new operator-version<br/>(ResolvedReplace, logged)"]
    CONT["Continue under prior version<br/>(ResolvedKeep, documented justification)"]
    REEVAL["Explicit re-evaluation<br/>at next assessment"]

    EAP["Evolution-aware provenance<br/>(DEF-TE-07; operator-version, currency-status,<br/>propagation-decision, resolving principal)"]

    REG_OUT["Regulatory artifact<br/>(FDA PCCP submission,<br/>EU AI Act Art. 72 monitoring report)"]
    INST_LEARN["Institutional retrospective<br/>(outcome correlation by<br/>operator-version transition)"]

    SRC ==> VR
    LSR ==> CC
    CC --> VR

    VR --> STATE_A
    STATE_A -. currency threshold crossed .-> STATE_ADV
    STATE_ADV -. configured-period elapsed .-> STATE_INACT

    SRC -. operator update available .-> SE
    SE --> VR
    SE -. Δ_PS transitioned .-> AR_PAT
    SE -. Δ_IS transitioned .-> AR_INST

    AR_PAT --> CMP_PAT
    AR_INST --> CMP_INST

    CMP_PAT -.one per active hypothesis.-> RR_PAT
    CMP_INST -.one per active capacity hypothesis.-> RR_INST

    RR_PAT --> CHOICE_PAT
    RR_INST --> CHOICE_INST

    CHOICE_PAT --> TRANS
    CHOICE_PAT --> CONT
    CHOICE_PAT --> REEVAL
    CHOICE_INST --> TRANS
    CHOICE_INST --> CONT
    CHOICE_INST --> REEVAL

    TRANS --> EAP
    CONT --> EAP
    REEVAL --> EAP

    EAP --> REG_OUT
    EAP --> INST_LEARN

    classDef external fill:#e2e3e5,stroke:#6c757d,stroke-width:1px,color:#000
    classDef substrate fill:#cfe2ff,stroke:#0d6efd,stroke-width:2px,color:#000
    classDef state fill:#cfe2ff,stroke:#0d6efd,stroke-width:2px,stroke-dasharray:5 5,color:#000
    classDef safetyGate fill:#cfe2ff,stroke:#dc3545,stroke-width:3px,color:#000
    classDef decision fill:#fff,stroke:#dc3545,stroke-width:3px,color:#000

    class SRC,LSR external
    class VR,CC,SE,AR_PAT,AR_INST,EAP,REG_OUT,INST_LEARN substrate
    class STATE_A,STATE_ADV,STATE_INACT state
    class CMP_PAT,CMP_INST,RR_PAT,RR_INST safetyGate
    class CHOICE_PAT,CHOICE_INST decision
    class TRANS,CONT,REEVAL substrate
```

**Reading this diagram.** External sources update at their own cadences (SSC every 4-5 years per 4D.2, KDIGO decadal with focused updates, antibiogram weekly, formulary daily). Living systematic review infrastructure (Cochrane LSR, MAGICapp, Australian Living Evidence Collaboration) feeds currency signals to the substrate's currency monitor. The version registry (DEF-TE-01) tracks every operator-version pair with effective period and status. Operators progress through three states: active (default), advisory (currency threshold crossed, licenses with explicit caveat per DEF-TE-03), inactive (configured-period elapsed past threshold, cannot license without re-verification).

When an operator update is available, the sound-evolution checker (DEF-TE-04) enforces the semantics of `NOTE.md` §4D.3: adding a new operator must not silently license previously-rejected recommendations; modifying tracks per-recommendation licensing differences; retiring flags outstanding recommendations for re-review. Each transition is categorized as **refines / generalizes / incomparable** (DEF-TE-05); incomparable transitions carry an additional justification burden per §4D.3 ¶2.

The diagram makes explicit the **symmetric re-review pathway** that `NOTE.md` §4D.4 commits to and `SPEC.md` v0.3.0 formalizes:

- **Patient-side propagation** (DEF-TE-06): when `Δ_PS` transitions, the patient substrate emits a `ReReviewEvent` for every active hypothesis whose provenance includes the retired operator. The responsible clinician resolves it as `ResolvedReplace`, `ResolvedKeep` (continue under prior version with documented justification), or holds for re-evaluation at next assessment.
- **Institutional-side propagation** (DEF-TE-06b — new in v0.3.0 / NOTE.md §4D.4 ¶2): when `Δ_IS` transitions (queue-priority policy revision, scarcity-allocation framework revision, formulary-restriction modification), the institutional substrate emits an `InstReReviewEvent` for every active capacity hypothesis whose provenance includes the retired operator. The event carries an `authority_class` field that routes resolution to the appropriate institutional authority — capacity manager, ethics committee, or formulary committee depending on the operator class. The lifecycle is structurally identical to the patient path.

This symmetry is load-bearing: `INV-TE-04` (no automatic replacement) and `OBL-TE-02` (no silent drift) both apply to `Δ_PS` and `Δ_IS` and hold **unconditionally on transition type** — even when the new operator strictly refines the old, automatic propagation is forbidden. The "obviously safer" judgment is reserved for human authority.

Evolution-aware provenance (DEF-TE-07, `NOTE.md` §4D.5) supports two downstream uses: regulatory artifact production (FDA PCCP submissions for change-control records, EU AI Act Article 72 post-market monitoring reports) and institutional retrospective analysis (outcome correlation by operator-version transition — did patients transitioned at point X have different outcomes than those continuing on the prior operator?). `OBL-TE-03` requires that audit queries filter on component _versions_ (not merely identities).

The 7E.6 worked example in the position note traces this lifecycle for the SSC 2021 → SSC 2026 transition concretely on the patient side. The general shape this diagram shows — including the institutional symmetric path — is what 7E.6 instantiates and what NOTE.md §4D.4 ¶2 mandates for institutional operator revisions.

---

## Diagram 5: Learned-component composition (expansion of Diagram 3)

The enumerated set of task-specific learned components and symbolic proposers that plug into the substrate's interfaces, drawn from position note Sections 4A.5 and 4B.5. Side-by-side columns separate patient-substrate components from institutional-substrate components, since the substrate distinction is architecturally significant (different lattice structures, different deduction-operator libraries, different abstention semantics).

Each component is shown at its interface position with: (a) the task it performs, (b) the architectural-options labels for the model that implements it (the architecture is agnostic about which option is chosen as long as the interface contract from Diagram 3 is satisfied), and (c) a maturity tag indicating whether the component slot is "plug in a known model" or "still requires architecture research."

```mermaid
flowchart TB
    subgraph SUB_P["Patient-substrate side"]
        PL_IF["Patient lattice<br/>interface contract<br/>(see Diagram 3)"]
    end

    subgraph SUB_I["Institutional-substrate side"]
        IL_IF["Institutional lattice<br/>interface contract<br/>(see Diagram 3)"]
    end

    subgraph PLC["Patient learned components (note §4A.5)"]
        RP["Refinement proposer<br/>(lattice-search component)<br/>Options: LDT-style / LLM-class /<br/>HRM-class / new architecture<br/>Maturity: research question"]
        IMG["Imaging recognizer(s)<br/>(radiology, pathology, dermatology)<br/>Options: vision transformer / specialized CNN<br/>Maturity: production-ready in literature"]
        PATH["Pathology morphology recognizer<br/>Options: specialized vision model<br/>Maturity: production-ready in literature"]
        ICU["ICU trajectory forecaster<br/>Options: state-space model /<br/>Temporal Fusion Transformer / RNN<br/>Maturity: production-ready in literature"]
        RPC["Rare phenotype clustering<br/>Options: contrastive embedding /<br/>specialized clustering<br/>Maturity: active research area"]
        MMI["Multimodal signal integrator<br/>(structured EHR + notes + imaging + time-series)<br/>Options: multimodal transformer / fusion model<br/>Maturity: active research area"]
        AR["Ambiguity ranker<br/>(prioritize candidate for next-test selection)<br/>Options: learned ranker / Bayesian<br/>Maturity: well-established"]
        DE["Differential expander<br/>(suggest candidates substrate has not considered)<br/>Options: LLM-class / specialized retrieval<br/>Maturity: active research area"]
        NS["Note synthesizer<br/>(narrative output from substrate state)<br/>Options: LLM-class under substrate constraints<br/>Maturity: active research area"]
        PSC["Prognostic scorer(s)<br/>(mortality, readmission, deterioration)<br/>Options: gradient boosting / specialized DL<br/>Maturity: well-established per outcome"]
    end

    subgraph ILC_LEARNED["Institutional learned components (note §4B.5)"]
        DF["Demand forecaster<br/>(anticipated admissions 6/24/72h)<br/>Options: state-space / Temporal Fusion Transformer /<br/>specialized time-series<br/>Maturity: well-established"]
        QDP["Queue dynamics predictor<br/>(expected wait times)<br/>Options: queueing-theory + ML / specialized<br/>Maturity: well-established"]
        LOS["Length-of-stay estimator<br/>Options: survival analysis / gradient boosting / DL<br/>Maturity: well-established"]
        SDP["Supply depletion projector<br/>(drug, blood product, consumable inventory)<br/>Options: time-series forecasting / ARIMA / DL<br/>Maturity: well-established"]
    end

    subgraph ILC_SYMBOLIC["Institutional symbolic proposers (note §5)"]
        ORS["Operations-research solver(s)<br/>(scheduling, assignment, routing)<br/>Options: MILP / constraint programming / heuristic<br/>Maturity: well-established"]
        APL["Allocation policy logic<br/>(priority, equity, scarcity ethics)<br/>Options: hand-specified rules<br/>Maturity: production by design"]
    end

    PL_IF --- RP
    PL_IF --- IMG
    PL_IF --- PATH
    PL_IF --- ICU
    PL_IF --- RPC
    PL_IF --- MMI
    PL_IF --- AR
    PL_IF --- DE
    PL_IF --- NS
    PL_IF --- PSC

    IL_IF --- DF
    IL_IF --- QDP
    IL_IF --- LOS
    IL_IF --- SDP
    IL_IF --- ORS
    IL_IF --- APL

    classDef substrate fill:#cfe2ff,stroke:#0d6efd,stroke-width:2px,color:#000
    classDef learnedWellEst fill:#ffe5b4,stroke:#fd7e14,stroke-width:2px,color:#000
    classDef learnedActiveResearch fill:#ffe5b4,stroke:#fd7e14,stroke-width:2px,stroke-dasharray:3 3,color:#000
    classDef learnedResearchQ fill:#ffe5b4,stroke:#fd7e14,stroke-width:2px,stroke-dasharray:8 4,color:#000
    classDef symbolic fill:#d1e7dd,stroke:#198754,stroke-width:2px,color:#000

    class PL_IF,IL_IF substrate
    class IMG,PATH,ICU,AR,PSC,DF,QDP,LOS,SDP learnedWellEst
    class RPC,MMI,DE,NS learnedActiveResearch
    class RP learnedResearchQ
    class ORS,APL symbolic
```

**Reading this diagram.** Two columns separate patient-substrate learned components from institutional-substrate learned and symbolic-proposer components.

**Patient-substrate side (orange, learned components).** Ten task-specific components, every one named in position note Section 4A.5 ("imaging pattern recognition, pathology morphology, ICU trajectory forecasting, rare phenotype clustering, multimodal signal integration, ambiguity ranking, differential expansion, note synthesis") plus the refinement proposer that does lattice-search and operator selection (4A.5's "the transformer (or whatever the learned component is)") plus prognostic scorers that the note's 7E.1 sepsis example explicitly invokes. Each component lists its architectural options without committing to any single choice: the refinement proposer slot can be filled by LDT-style, LLM-class, HRM-class, or a future architecture entirely. The substrate doesn't care which, as long as the Diagram 3 interface contract is satisfied.

**Institutional-substrate side, learned components (orange).** Four task-specific components, every one named in 4B.5 ("demand forecasting models, queue dynamics predictors, supply availability estimators, length-of-stay predictors"). Each with its architectural-options labels.

**Institutional-substrate side, symbolic proposers (green).** The note's Section 5 explicitly names operations-research solvers as a component class on the institutional side: "the institutional lattice typically a mix of forecasting models, queue dynamics estimators, and operations-research solvers." These propose moves to the institutional substrate just like learned components do — the substrate licenses or rejects them through the same interface — but they are not learned. They are deterministic algorithms (MILP, constraint programming, hand-specified allocation policies). The visual distinction (green vs. orange) reflects this architectural significance: same interface position, different epistemic basis.

**Maturity asymmetry is shown by border style.** Solid borders are well-established components where the architecture decision is "plug in a known model" — vision transformers for imaging, gradient boosting for prognostic scoring, time-series models for demand forecasting. Short-dashed borders are active research areas where multiple plausible architectures exist and the right choice is empirically open — multimodal integration, rare phenotype clustering, differential expansion, note synthesis under substrate constraints. Long-dashed borders are research questions where the architecture itself is the open problem — the refinement proposer is the headline example. The position note's Section 7 timeline predictions (2-4 years for narrow applications, 5-8 years for broader diagnostic reasoning, 8-12 years for novelty-handling) reflect this asymmetry directly.

**What the diagram is not committing to.** Specific model architectures at each slot. The substrate-first architecture's claim is that the safety property holds across model choices at each interface position. The labels list options; they do not pick.

**The refinement proposer slot is highlighted as the headline research question.** The position note's Section 5 line 155 names it as the "transformer-branching" point: "The most plausible learned component, given current technical maturity, is a recurrent transformer of the LDT type" *[inferred]*. Most plausible per current maturity is not a commitment; it's a working assumption. If a different architecture turns out to be better suited to the refinement-proposing task under clinical lattice constraints, the substrate's safety properties hold under the swap.

---

## Cross-diagram traceability to `NOTE.md` and `SPEC.md`

| Diagram | `NOTE.md` v0.12.0 sections | `SPEC.md` v0.3.0 sections |
| --- | --- | --- |
| 1: Component decomposition | §4 (4A, 4B, 4C, 4D), §5 (system identity), §7A/B/C (build description) | §2 (patient substrate), §3 (institutional substrate), §4 (interaction layer), §5 (temporal evolution) |
| 2: Data and event flow | §4A.1–4A.4, §4B.1–4B.4, §4C, plus §7E.1 (sepsis worked example for concrete instantiation) | §2.3 (Galois connection), §2.4–§2.5 (operators + abstention), §3.3–§3.5 (institutional operators + abstention), §4.1–§4.4 (cross-layer + joint licensing + joint abstention with structured diff DEF-IX-09) |
| 3: Substrate-learned-component boundary | §4A.5, §4B.5, §4C, plus §5 (regulatory positioning under UNDCS taxonomy) | §2.7 (constrained proposer DEF-PS-14, INV-PS-06, OBL-PS-05), §3.7 (constrained capacity-learned proposer DEF-IS-14, INV-IS-06, OBL-IS-06) |
| 4: Temporal-evolution lifecycle | §4D (all five principles), §6 temporal-evolution prior art subsection, §7E.6 worked example | §5.1 (versioning DEF-TE-01), §5.2 (currency DEF-TE-02/03), §5.3 (sound evolution DEF-TE-04/05, OBL-TE-01), §5.4 (re-review **DEF-TE-06 + DEF-TE-06b**, INV-TE-04, OBL-TE-02), §5.5 (evolution-aware provenance DEF-TE-07, OBL-TE-03) |
| 5: Learned-component composition (expansion of 3) | §4A.5 (enumerated patient learned components), §4B.5 (enumerated institutional learned components), §5 (operations-research solvers, transformer-branching) | §2.7 (DEF-PS-14 black-box constraint), §3.7 (DEF-IS-14 capacity-learned proposer); v0.3.0 deliberately leaves model-architecture taxonomy out of SPEC.md scope — see SPEC.md §0.5 |

If a future revision of `NOTE.md` adds, removes, or substantively modifies any principle in §4, the relevant diagram(s) above need updating to match. Similarly, if `SPEC.md` adds, removes, or restructures the formal anchors (`DEF-*`, `INV-*`, `OBL-*`), the diagrams and the principle-to-formalization map at the top of this document need updating. The diagrams version independently of `NOTE.md` and `SPEC.md`; this v0.2.0 reflects `NOTE.md` v0.12.0 and `SPEC.md` v0.3.0.

---

## Open decisions

1. **Whether to add a 6th diagram for cross-cutting concerns specifically.** Currently audit infrastructure, ontology binding, clinician interface, capacity-management interface, evaluation harness, and regulatory artifact production are visible in Diagram 1 but not zoomed. A 6th diagram showing how each cross-cutting concern threads through the four axes would add legibility at the cost of one more diagram to maintain. Deferred to v0.2.0 of the diagrams pending feedback on whether the current set is sufficient.

2. **Whether Diagram 5 should be split further by maturity.** Currently maturity is encoded in border style, which is subtle. An alternative would be three sub-diagrams in Diagram 5: well-established components, active-research components, research-question components. Subtle border encoding is more compact; explicit sub-diagrams are more legible. Lean toward keeping it as one diagram for v0.1.0; revisit if border styles prove too subtle in actual reading.

3. **Whether the visual conventions should be moved into a shared style file.** Mermaid supports `init` blocks for shared themes. A shared theme would keep colors and stroke widths consistent across diagrams without repetition in each diagram's classDef. Deferred to v0.2.0; v0.1.0 uses inline classDef for clarity.

4. **Whether to produce an executive-summary single-diagram view** for audiences who will only look at one picture. Single-diagram views encode trade-offs explicitly (some detail must be hidden), so the question is what gets hidden. Deferred pending feedback on whether the audience for an executive-summary view exists.

---

## Changelog

- **v0.2.0-draft (2026-05-25):** Re-anchored to `SPEC.md` v0.3.0-draft per `SPEC.md` §0.1 and §9.3 (ARCHITECTURE.md draws its diagrams from `SPEC.md`, not from `NOTE.md` directly). Header references bumped to `NOTE.md` v0.12.0-draft and `SPEC.md` v0.3.0-draft. Purpose section now states the three-document resolution order explicitly (`NOTE.md` > `SPEC.md` > diagrams). New **principle-to-formalization map** added between visual conventions and Diagram 1, listing all eighteen `NOTE.md` §4 principles with their criticality tier (P/S/F per `NOTE.md` §4 intro: 5 P / 10 S / 3 F), their `SPEC.md` formal anchors (`DEF-*`, `INV-*`, `OBL-*`), and the diagrams each principle is visible in. Cross-cutting invariant `INV-IX-04` (substrate-local soundness preserved under coupling) called out explicitly. **Diagram 4 (temporal-evolution lifecycle) restructured to render the institutional symmetric re-review path** mandated by `NOTE.md` §4D.4 ¶2 and formalized as `DEF-TE-06b` in `SPEC.md` §5.4: when `Δ_IS` transitions (queue-priority, scarcity-allocation, formulary-restriction), `InstReReviewEvent`s carry an `authority_class` field routing resolution to capacity manager, ethics committee, or formulary committee rather than to a bedside clinician. The lifecycle is structurally identical to the patient path (DEF-TE-06) — same `ReReviewStatus` lifecycle, same provenance discipline, same prohibition on automatic replacement per `INV-TE-04`. Operator-transition categorization (refines / generalizes / incomparable per DEF-TE-05) called out in the sound-evolution checker node. Cross-diagram traceability table extended to include `SPEC.md` v0.3.0 section anchors alongside `NOTE.md` v0.12.0 section anchors. Diagram 4 reading prose rewritten to make the symmetry, `INV-TE-04` unconditional-on-transition-type property, and `OBL-TE-02 / OBL-TE-03` audit-query obligation visible. Diagrams 1, 2, 3, 5 are structurally unchanged from v0.1.0; their content remains consistent with the new anchors. Four open decisions from v0.1.0 carried forward unchanged (6th diagram for cross-cutting concerns, splitting Diagram 5 by maturity, shared style file, executive-summary single-diagram view).

- **v0.1.0-draft (2026-05-24):** Initial draft. Five Mermaid diagrams covering component decomposition (1), data and event flow (2), substrate-learned-component boundary (3), temporal-evolution lifecycle (4), and learned-component composition as expansion of 3 (5). Visual conventions stated once at the top and applied across all five. Cross-diagram traceability table mapping each diagram to position note sections. Maturity asymmetry encoded via border style (solid / short-dashed / long-dashed) in Diagram 5. Side-by-side columns separate patient-substrate components from institutional-substrate components in Diagram 5. Symbolic proposers (operations-research solvers, hand-specified allocation policies) shown with green coloring to distinguish from learned components (orange) while preserving the architectural truth that they sit at the same interface position. Refinement proposer slot in Diagram 5 highlighted as the headline research question per Section 5 line 155's *[inferred]* "transformer-branching" claim, with architectural options listed (LDT-style / LLM-class / HRM-class / new architecture entirely) and no commitment to any single option. The diagrams reflect the multi-model heterogeneity reading the position note's hedges in 4A.5, 4B.5, and Section 5 support; this reading was clarified during the design discussion that produced this companion document and is recorded here as the architectural truth the diagrams encode. Four open decisions named explicitly for future versions: 6th diagram for cross-cutting concerns, splitting Diagram 5 by maturity, shared style file, executive-summary single-diagram view.
