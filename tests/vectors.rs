//! Official BIP-350 / BIP-173 test vectors + combined-decoder semantics.

use azar::verify_bech32m;

/// Valid Bech32m strings — BIP-350 "Test vectors for Bech32m".
const VALID_BECH32M: &[&str] = &[
    "A1LQFN3A",
    "a1lqfn3a",
    "abcdef1l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryx",
    "split1checkupstagehandshakeupstreamerranterredcaperredlc445v",
    "?1v759aa",
    // 83-char HRP vector from BIP-350:
    "an83characterlonghumanreadablepartthatcontainsthetheexcludedcharactersbioandnumber11sg7hg6",
    // 11... vector from BIP-350:
    "11llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllludsr8",
];

/// Valid Bech32 (classic) strings — these must verify via the combined
/// decoder but NOT be produced by our bech32m generator.
const VALID_BECH32: &[&str] = &[
    "A12UEL5L",
    "a12uel5l",
    "abcdef1qpzry9x8gf2tvdw0s3jn54khce6mua7lmqqqxw",
    "split1checkupstagehandshakeupstreamerranterredcaperred2y9e3w",
];

/// Invalid strings — must be rejected (BIP-173 + BIP-350 negative vectors).
const INVALID: &[&str] = &[
    "pzry9x0s0muk",                                  // no separator
    "1pzry9x0s0muk",                                 // empty HRP
    "x1b4n0q5v", // invalid data char ('b' excluded from charset)
    "li1dgmt3",  // too short checksum
    "de1lg7wt",  // invalid char in checksum (truncated vector)
    "A1G7SGD8",  // checksum calculated with uppercase form of HRP
    "10a06t8",   // empty HRP
    "1qzzfhee",  // empty HRP
    "abcdef1l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryy", // last char mutated
    "abcdef1l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryX", // mixed case
];

#[test]
fn valid_bech32m_vectors_accepted() {
    for v in VALID_BECH32M {
        assert!(verify_bech32m(v), "must accept bech32m vector: {v}");
    }
}

#[test]
fn valid_bech32_vectors_accepted_by_combined_decoder() {
    for v in VALID_BECH32 {
        assert!(verify_bech32m(v), "combined decoder must accept: {v}");
    }
}

#[test]
fn invalid_vectors_rejected() {
    for v in INVALID {
        assert!(!verify_bech32m(v), "must reject: {v}");
    }
}

#[test]
fn mutation_always_detected() {
    // flipping ANY single character of a valid bech32m string must fail
    let base = "abcdef1l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryx";
    let charset: Vec<char> = "qpzry9x8gf2tvdw0s3jn54khce6mua7l".chars().collect();
    for (i, orig) in base.chars().enumerate() {
        for &c in &charset {
            if c == orig {
                continue;
            }
            let mutated: String = base
                .chars()
                .enumerate()
                .map(|(j, ch)| if j == i { c } else { ch })
                .collect();
            assert!(
                !verify_bech32m(&mutated),
                "single-char mutation must be detected: {mutated}"
            );
        }
    }
}
