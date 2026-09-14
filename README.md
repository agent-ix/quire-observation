# quire-observation

Private Rust implementation of qualified observation bindings and
transport-independent valuations for Quire (OB01).

The library admits records only when their selected package, producer,
binding, source/schema, subject, relationship, population/window, clock range,
and resource limits are explicit and compatible. It returns `Available`,
`Incomplete`, or `Refused`; it never treats absent evidence as Boolean false.
It also publishes nine canonical bounded owner contracts for observation,
population, position, clock, capture, progress, closure, completeness, and
result availability. Consumers receive only strict-read validated views.

It accepts a selected `native-linked-package/1` descriptor and consumes the
constructor-private `AdmittedStaticBundle` from the FCD Producer interface 1.2
Rust crate pinned at `404288282402d60de007295ccbafa960532b955e`. It deliberately
has no compiler dependency, FCD decoder, telemetry adapter, temporal evaluator,
or protocol-result mapper. It does not claim production monitor execution,
protocol conformance interpretation, or evidence-store ownership.

## Owner contracts

| Module | Contract | Schema SHA-256 | Public reader |
| --- | --- | --- | --- |
| `authority::observation` | `quire.observation.record/v2` | `8737683a5971aabcb39bbc5f0842fa1d7c1db5c8f283b8b2544c6364c682f882` | `observation::read` |
| `authority::population` | `quire.observation.population/v2` | `bceba1a2a69d150af05a7f7bea52769560849fd24a8582cb35b0937eee263c01` | `population::read` |
| `authority::position` | `quire.observation.position-ledger/v1` | `aac2fd7fc1b129e24afec397900647f3fe8977341b9907662a3852ac5430211a` | `position::read` |
| `authority::clock` | `quire.observation.clock-binding/v1` | `c9a7b154a2c2d775ba71ba001842e6f0595e123c07b23a2bb37215c1f05de38d` | `clock::read` |
| `authority::capture` | `quire.observation.capture-environment/v1` | `6d9260d3c4de65b5c3304debdb9baa7816aaea55949839924182d1db4c24c516` | `capture::read` |
| `authority::progress` | `quire.observation.progress-assertion/v1` | `91223fb982b68d4fb7c66b3737370429bd4841321f5b561ebc6c1e9691bd7f1c` | `progress::read` |
| `authority::closure` | `quire.observation.closure-assertion/v1` | `8638fb9f4dd5a26d44ae9975388fc734670992914422e23c27eaae82149245f4` | `closure::read` |
| `authority::completeness` | `quire.observation.completeness-assertion/v1` | `b7c3c48d5f907cfb5bd7b4450e009b6c95f7d558095ece8b81d2ae1d5821bcf0` | `completeness::read` |
| `authority::availability` | `quire.observation.result-availability/v1` | `2e6c00d6dc94a00b0859b346dc12a14c7142735a565e35d15b85895d311e8416` | `availability::read` |

Population admission additionally strict-reads
`quire.observation.explicit-members/v1` through
`population::read_membership`; its schema digest is
`408c9d2908c3660d657cea574f52d8531e9ab749ec78c8d058fe5f163df23cce`.

The immutable record/population v1 schema files remain committed at their
original digests (`2922c6ad…` and `493b4c10…`). Their active qualified owner
documents are v2 because FR-287 adds authority, revision, digest-domain and
opaque-byte members that v1 cannot represent without an in-place schema break.

## Local checks

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test --locked
```
