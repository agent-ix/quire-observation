# Contributor guidance

Read this file, `README.md`, `LICENSE-DECISION.md`, and the owning issues
before editing. This is the implementation for creation ticket
`agent-ix/quire-research#32` and OB01 (`#44`) under OBSERVATION (`#37`).

- Keep the semantic core transport independent. FCD and telemetry adapters are
  not part of OB01.
- Preserve exact selected identities, digests, incomplete causes, and typed
  refusals. Do not infer relationships, completeness, values, clocks, or
  defaults from ambient state.
- New production and test code is Rust. Hosted workflows must be
  `workflow_dispatch` only; do not dispatch one without explicit direction.
- Run `cargo fmt --check`, `cargo clippy -- -D warnings`,
  `cargo test --locked`, and
  `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` before requesting
  review.
- Do not claim timed settlement, late-data replay, or result/evidence mapping:
  those belong to OB02 and OB03.
