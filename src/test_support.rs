// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Owned test fixtures for downstream test code, behind the `test-support`
//! feature. Not part of the production surface.

/// The Producer interface 1.2 static bundle document this crate's own tests
/// use, embedded from its locally owned fixture file.
pub const FCD_STATIC_BUNDLE_1_2: &[u8] =
    include_bytes!("../tests/fixtures/fcd-static-bundle-1.2.json");
