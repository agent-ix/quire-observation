// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-014: the `test-support` export of the owned static-bundle fixture.
#![cfg(feature = "test-support")]

use ix_trace_rs::trace;
use quire_observation::{test_support::FCD_STATIC_BUNDLE_1_2, StaticProducerBundle};

#[trace("TC-014", "FR-012-AC-1")]
#[test]
fn fr_012_ac_1_exported_bundle_is_the_owned_fixture_and_admits() {
    assert_eq!(
        FCD_STATIC_BUNDLE_1_2,
        include_bytes!("fixtures/fcd-static-bundle-1.2.json").as_slice()
    );
    StaticProducerBundle::admit_json(FCD_STATIC_BUNDLE_1_2).expect("exported bundle is admitted");
}
