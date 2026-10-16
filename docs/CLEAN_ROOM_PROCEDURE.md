# Clean-room procedure: independent verification of EP-QUORUM

- **Status:** v0.1, 2026-10-16.
- **Owner:** TRELYAN Inc.
- **Target:** draft-schrock-ep-quorum-04, "Multi-Party Quorum Authorization for High-Risk Agent Actions", and its normative base, draft-schrock-ep-authorization-receipts-12.

## Why

The draft's authors state that their three reference verifiers "share one repository and are therefore a consistency check rather than clean-room independent implementations" and that "Independent implementations remain future interoperability evidence" (section 9). This procedure defines how we produce that independent evidence, and how anyone can check that we did.

## 1. Allowed inputs

| Allowed | Not allowed before the freeze (step 4) |
|---|---|
| The two draft texts, pinned by SHA-256 (see `SOURCES.sha256`) | The authors' reference verifier source code, in any language |
| Normative references: RFC 8785 (JCS), RFC 8174/2119, W3C WebAuthn Level 2, RFC 3339 | The authors' corpus vector files |
| Our own notes and the spec-reading log | Implementation discussions in the authors' issue tracker |

## 2. Two separate readings

- **Rust verifier.** Written by TRELYAN from the text alone.
- **Lean 4 model.** Written by a different person (a contracted formal-methods engineer), also from the text alone.
- **Isolation.** Neither author sees the other's work until both are frozen. Agreement between the two is therefore not same-author agreement.

## 3. Spec-reading log

- Every ambiguity, inconsistency or unstated rule is logged with its section and line reference and the interpretation we chose.
- Security-relevant entries stay private until disclosure (step 6). Editorial entries are published with each release.

## 4. Freeze before comparison

Before any reference verifier is run, we tag our verifier and the cases we derived from the text, and publish the tag's commit hash. Only after the freeze do we run the reference verifiers, as black boxes, on the same inputs.

## 5. Differential testing and triage

Each case runs through:
- our verifier;
- each reference verifier;
- the Lean model's executable checker.

Every disagreement in verdict or reason code is classified by reading the draft text, never by majority vote:

- (a) a bug in our verifier;
- (b) a bug in a reference verifier;
- (c) a spec ambiguity, where the text supports both readings.

## 6. Disclosure

- Findings go privately to the draft's authors first. We publish after they respond, or after 45 days.
- Cases that would harm deployed systems wait until a fix or a clarified text exists.

## 7. AI use

AI coding agents help write code and tests, and AI-assisted proving helps with Lean. Every accepted case and every proof step is checked mechanically: by the verifiers, by the Lean kernel, or by a human against the text.

Unpublished, embargoed findings are never sent to third-party AI services. Attack generation over embargoed material runs on local, private inference.

## 8. Records

Each release publishes:
- the draft text hashes;
- toolchain versions (rustc, Lean, Mathlib);
- the frozen tag;
- the triage table;
- a statement of what passing does and does not show.
