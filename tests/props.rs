//! Property-based tests: every generated identifier satisfies the
//! structural contract, across the full flag surface.

use azar::{
    base58, base64, bech32m_bare, bech32m_payload, hex, overhead, verify_bech32m, BECH32_MAX_LEN,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn bech32m_bare_totals_are_exact(l in 7usize..=84) {
        // bech32m_bare takes PAYLOAD symbols; total = payload + 6 checksum.
        // regression F5: the floor is 7 (1 payload + 6 checksum) — exactly
        // the verifier's structural minimum, so generate ⇔ verify holds at
        // every generatable length. Below 7 nothing is generated at all
        // (the old checksum-less `plain` path is gone).
        let s = bech32m_bare(l - 6)?;
        prop_assert_eq!(s.chars().count(), l);
        prop_assert!(!s.contains('1'));
        prop_assert!(verify_bech32m(&s));
    }

    #[test]
    fn bech32m_prefixed_totals_are_exact(prefix in "[a-z]{1,5}", l in 16usize..=80) {
        let ovh = overhead(&prefix);
        let payload = l - ovh;
        let s = bech32m_payload(&prefix, payload)?;
        let expected_start = format!("{prefix}1");
        prop_assert_eq!(s.chars().count(), l);
        prop_assert!(s.starts_with(&expected_start));
        prop_assert!(verify_bech32m(&s));
        prop_assert!(l <= BECH32_MAX_LEN);
    }

    #[test]
    fn base58_targets_are_exact(t in 1usize..=100) {
        let s = base58(Some(t), false)?;
        prop_assert_eq!(s.chars().count(), t);
    }

    #[test]
    fn hex_targets_are_exact_odd_included(t in 1usize..=64) {
        let s = hex(Some(t))?;
        prop_assert_eq!(s.chars().count(), t);
        prop_assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn base64_targets_are_exact(t in 1usize..=200) {
        let s = base64(Some(t))?;
        prop_assert_eq!(s.chars().count(), t);
    }

    #[test]
    fn every_prefixed_output_roundtrips(n in 8usize..=80) {
        let s = bech32m_payload("r32", n)?;
        prop_assert!(verify_bech32m(&s));
    }
}
