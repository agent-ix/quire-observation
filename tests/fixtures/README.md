# Test fixture provenance

`fcd-static-bundle-1.2.json` is a locally owned test fixture shaped like a
Producer interface 1.2 static bundle document. It exercises this crate's own
local admission code in `src/producer.rs`, which decoupled from
`agent-ix/filament-core-data` and no longer depends on it. The document shape
is kept for realism and to preserve this crate's existing test coverage; it is
not a copy of, or a dependency on, anything outside this repository.
