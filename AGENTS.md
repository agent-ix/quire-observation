# Contributor guidance

Read this file, `README.md`, `LICENSE-DECISION.md`, and the owning issues
before editing. This is the implementation for creation ticket
`agent-ix/quire-research#32` and OB01 (`#44`) under OBSERVATION (`#37`).

## Hash / digest / pin antipattern: present, slated for removal

This repository still contains hashes, digests, SHAs, pins, checksum catalogs and
records that track files, versions or tools. That is an antipattern and it is slated
for removal. Do not introduce any new use of it. When you touch code or specs that
contain one, remove it as part of the change. The only hash that stays is a canonical
identity digest that binds a proof to the exact content it proved. Package versions
live in Cargo.toml / package.json and their lockfiles only; reports name the app
version they ran.

## Rules

- Keep the semantic core transport independent. FCD and telemetry adapters are
  not part of OB01.
- Preserve exact selected identities, incomplete causes, and typed
  refusals. Do not infer relationships, completeness, values, clocks, or
  defaults from ambient state.
- New production and test code is Rust. Hosted workflows must be
  `workflow_dispatch` only; do not dispatch one without explicit direction.
- Run `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
  `cargo test --locked`, and
  `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` before requesting
  review.
- Do not claim timed settlement, late-data replay, or result/evidence mapping:
  those belong to OB02 and OB03.
