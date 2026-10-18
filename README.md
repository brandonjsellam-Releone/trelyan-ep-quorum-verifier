# trelyan-ep-quorum-verifier

Work toward an independent Rust verifier, built under a clean-room procedure, for the IETF Internet-Draft
[draft-schrock-ep-quorum-04](https://datatracker.ietf.org/doc/draft-schrock-ep-quorum/),
"Multi-Party Quorum Authorization for High-Risk Agent Actions" (EP-QUORUM), and the parts of its
normative base, draft-schrock-ep-authorization-receipts-12, that it depends on.

> **Status: early work in progress. Not for production use.** Only the two building blocks listed
> below are implemented. There is no quorum verdict yet.

## Why this exists

EP-QUORUM defines a check for "two-person rules" on AI agents: several distinct, accountable people
must sign before a high-risk agent action may run. Its authors write in section 9 that their three
reference verifiers are "a consistency check rather than clean-room independent implementations",
and that "Independent implementations remain future interoperability evidence." This project
aims to provide that evidence: an implementation written from the draft text alone, outside the
authors' team. Today it is written at TRELYAN Inc. by one engineer working with AI coding agents,
with every change reviewed and tested. It is a single independent implementation, not yet the plural
interoperability evidence the draft calls for. Other implementers are welcome.

This project is not affiliated with the draft's authors (EMILIA Protocol, Inc.).

## What is implemented

| Crate | What it does | Tests |
|---|---|---|
| `epq-json` | Strict JSON parse boundary and RFC 8785 (JCS) canonicalization, under the profile of receipts-12 section 13.12. It rejects duplicate member names (after escape decoding), unpaired surrogates, non-integer numbers, integers above 2^53-1 and nesting deeper than 64. | RFC 8785 sorting, string and byte known-answer tests, plus profile rejection tests |
| `epq-chain` | The `EP-QUORUM-SIGNOFF-CHAIN-v1` completed-signoff link from the draft's section 4. | Known answers computed with an independent SHA-256 implementation; a negative test pinning the forbidden backslash-zero separator |

Planned: the signoff model, WebAuthn (ES256 and Ed25519) assertion verification, the section 5 Quorum
Gate, section 6 incremental admission, and a frozen corpus of cases derived from the text. See
[docs/MODEL_SCOPE.md](docs/MODEL_SCOPE.md) for the formal-model plan, which a separate person will
author.

## How independence is kept

See [docs/CLEAN_ROOM_PROCEDURE.md](docs/CLEAN_ROOM_PROCEDURE.md).

- Only the draft texts are read. Their hashes are in [docs/SOURCES.sha256](docs/SOURCES.sha256).
- The authors' reference code and vector files are not opened before our behaviour is frozen and
  tagged.
- Disagreements are triaged against the draft text, never by majority vote.

## Disclosure

Findings about the draft go privately to its authors first, under the authors' published disclosure
policy where one exists. We publish only when that policy allows it: once a fix or clarified text is
available, or once its disclosure window ends without one. Without such a policy, we publish no earlier
than 45 days after first contact.

## What passing does not show

A future "satisfied" verdict from this verifier will mean that the approval evidence meets the
draft's rules under the inputs given. It will not show:
- that the approver directory is correct;
- that the approvers are distinct humans, read what they signed, or did not collude;
- that any timestamp is true;
- that the action was safe.

## Building

Rust 1.85 or later.

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo deny check
```

## License

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

## Contact

TRELYAN Inc.: fondation@trelyan.ch
