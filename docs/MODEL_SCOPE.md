# Formal model scope: what the Lean 4 work will and will not cover

- **Status:** v0.1 (plan), 2026-10-16.
- **Who writes it:** the model is written by a contracted formal-methods engineer from the draft text alone (see CLEAN_ROOM_PROCEDURE.md). This page fixes its scope in advance, so that "proved" cannot drift into "safe".

## What is modelled

- **Policy:** mode, required, roster, distinct_humans, window_sec, ordered_chain and its profile.
- **Member:** role, key, signed context, signature.
- **Trail:** the ordered list of members.
- **Quorum Gate:** a decidable predicate over (policy, action hash, members), following draft section 5.
- **Incremental admission (canAccept):** following section 6, to compare it with the Gate.

## Stated assumptions (axioms, not proved)

These are the assumptions the draft itself states in section 4:

- **Signature unforgeability:** abstract `Verifies(pk, msg, sig)`, assuming EUF-CMA.
- **Hash preimage and collision resistance:** of the completed-signoff link function.

## Planned theorems

1. **Gate soundness.** A satisfied verdict implies every listed check holds. The fail-closed direction: a malformed input gives "not satisfied".
2. **General k-of-n.** A satisfied verdict implies at least `required` members such that:
   - each is in the roster;
   - each verifies;
   - each is bound to the action hash;
   - their keys are pairwise distinct;
   - their approvers satisfy the draft's distinct-human and initiator-exclusion rules.
3. **Ordered mode.** A satisfied ordered verdict implies member i fills roster slot i and the asserted issue times strictly increase.
4. **Causal-chain dependence.** For the completed-signoff profile: under the stated assumptions, a satisfied chain cannot have a predecessor signoff replaced by a different one without a hash collision or a forged signature.
5. **Admission vs Gate.** How canAccept relates to the Gate. Where admission accepts a trail that the Gate rejects, the model exhibits it.

## Checks on the proofs themselves

- **Satisfiability.** For each theorem, a concrete satisfying example, such as a 2-of-3 threshold quorum or a 3-of-3 ordered chain, so the rules are not vacuous.
- **Per-rule counterexamples.** For each check in the Gate, a machine-checked example that violates the target property once that check is removed.
- **Kernel checking.** All proofs are checked by Lean's kernel, with no kernel-skipping options. No `sorry` or `admit` in released proofs.

## Out of scope (stated in every report)

- WebAuthn and authenticator internals.
- Byte-level JCS and JSON parsing. Differential testing of the Rust verifier covers these instead.
- Correctness of the approver directory, and whether enrolled identities are distinct humans.
- Collusion and coercion.
- Wall-clock time and backdating.
- Human attention or comprehension.
- Whether the action itself is safe.

## How the model meets the code

The model exports an executable checker. Every corpus case runs through the Rust verifier, the reference verifiers and this checker. Disagreements are triaged against the draft text.
