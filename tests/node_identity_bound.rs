// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-004: structural node identity encoding is bounded before any canonical
//! text is retained.

use ix_trace_rs::trace;
use quire_exact::{NodeKey, PathSegment};
use quire_observation::authority::{progress::NodeIdentity, ErrorCode, Limits};

/// Each level names the previous level twice, so the canonical text doubles
/// per level while the key shares its storage and stays small.
fn doubling_key(levels: usize) -> NodeKey {
    let mut key = NodeKey::new("Leaf", vec![PathSegment::Text("x".to_owned())], 0).expect("leaf");
    for _ in 0..levels {
        key = NodeKey::new(
            "Level",
            vec![PathSegment::Node(key.clone()), PathSegment::Node(key)],
            0,
        )
        .expect("level");
    }
    key
}

#[test]
#[trace("TC-004", "FR-004-AC-4")]
fn over_limit_key_is_refused_before_its_canonical_text_is_built() {
    // The canonical text would be far beyond any allocation the process could
    // satisfy; refusal must come from the byte bound, not from building it.
    let key = doubling_key(48);
    let err = NodeIdentity::from_key(key, Limits::owner_max()).unwrap_err();
    assert_eq!(err.code(), ErrorCode::ResourceIncomplete);
}

#[test]
#[trace("TC-004", "FR-004-AC-4")]
fn key_at_the_exact_limit_is_admitted_and_one_under_is_refused() {
    let key = NodeKey::new("Checkout", vec![PathSegment::Text("a\"b".to_owned())], 7).expect("key");
    let text = r#"{"ordinal":7,"package":"Checkout","path":["a\"b"]}"#;
    let mut limits = Limits::owner_max();
    let encoded = serde_json::to_string(text).expect("JSON string").len() - 2;
    limits.max_string_bytes = encoded;
    let admitted = NodeIdentity::from_key(key.clone(), limits).expect("exact bound");
    assert_eq!(admitted.as_str(), text);

    limits.max_string_bytes = encoded - 1;
    assert_eq!(
        NodeIdentity::from_key(key, limits).unwrap_err().code(),
        ErrorCode::ResourceIncomplete
    );
}

#[test]
#[trace("TC-004", "FR-004-AC-4")]
fn key_without_escapes_is_admitted_at_the_exact_output_bound() {
    let key =
        NodeKey::new("Checkout", vec![PathSegment::Text("model".to_owned())], 7).expect("key");
    let text = r#"{"ordinal":7,"package":"Checkout","path":["model"]}"#;
    let mut limits = Limits::owner_max();
    limits.max_output_bytes = text.len();
    let admitted = NodeIdentity::from_key(key.clone(), limits).expect("exact output bound");
    assert_eq!(admitted.as_str(), text);

    limits.max_output_bytes = text.len() - 1;
    assert_eq!(
        NodeIdentity::from_key(key, limits).unwrap_err().code(),
        ErrorCode::ResourceIncomplete
    );
}
