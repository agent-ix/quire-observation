// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Canonical, bounded observation-owner artifacts.
//!
//! These modules deliberately expose validated views rather than parsers or
//! protocol decisions. A view can only be obtained by deriving owner bytes or
//! by strict-reading bytes against independently supplied qualified state.

mod common;

pub mod availability;
pub mod capture;
pub mod clock;
pub mod closure;
pub mod completeness;
pub mod observation;
pub mod partial;
pub mod population;
pub mod position;
pub mod progress;

pub use common::{
    AuthoritySelection, BoundaryRef, Context, Document, Error, ErrorCode, History,
    IncrementalHistory, Limits, OpenClosed, Result, SubjectSelection, TemporalBoundary, Usage,
};
