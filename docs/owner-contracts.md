# Owner contracts

Besides admission, quire-observation publishes fourteen canonical, bounded
owner contracts. Each one has a JSON Schema in [`schemas/`](../schemas/) and a
module under `authority::` that either derives the canonical bytes or reads
bytes back against independently supplied state. A reader returns a validated
view; there is no parser that hands back unchecked data.

| Module | Contract | Public reader |
| --- | --- | --- |
| `authority::activation` | `quire.observation.activation-scope-authority/v1` | `activation::read` |
| `authority::observation` | `quire.observation.record/v2` | `observation::read` |
| `authority::population` | `quire.observation.population/v2` | `population::read` |
| `authority::position` | `quire.observation.position-ledger/v1` | `position::read` |
| `authority::clock` | `quire.observation.clock-binding/v1` | `clock::read` |
| `authority::partial` | `quire.observation.partial-fact/v1` | `partial::read` |
| `authority::capture` | `quire.observation.capture-environment/v1` | `capture::read` |
| `authority::progress` | `quire.observation.progress-assertion/v1` | `progress::read` |
| `authority::closure` | `quire.observation.closure-assertion/v1` | `closure::read` |
| `authority::completeness` | `quire.observation.completeness-assertion/v1` | `completeness::read` |
| `authority::availability` | `quire.observation.result-availability/v1` | `availability::read` |
| `authority::bundle` | `quire.observation-authority/v1` | `bundle::read` / `bundle::read_revision` |
| `authority::bundle` | `quire.observation-authority/v2` | `bundle::read_v2` / `bundle::read_revision_v2` |
| `authority::bundle` | `quire.observation-authority-lineage/v1` | `bundle::read_lineage` |

Population admission additionally strict-reads
`quire.observation.explicit-members/v1` through
`population::read_membership`.

The record/population v1 schema files remain committed for their original
contracts. Their active qualified owner documents are v2 because the owner
contracts add authority, revision and opaque-byte members.
The authority-bundle v1 schema likewise remains immutable. Its v2 successor is
restricted to the exact seven-role structurally empty population profile; it
does not manufacture record-dependent sentinel facts.
