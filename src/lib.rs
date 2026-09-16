//! randid — random identifier generation.
//!
//! Bech32m output follows BIP-350 (checksum constant 0x2bc830a3) via the
//! `bech32` crate (rust-bitcoin). No checksum or charset code lives here:
//! encoding, the 8-to-5 bit packing and HRP validation all come from the
//! crate's primitives. Entropy comes from the OS CSPRNG (`getrandom`).
//!
//! Length sampling is bit-level: exactly 5 entropy bits per output
//! character (`bytes_to_fes().take(n)` — the crate's zero-padding only
//! applies AFTER the n-th symbol), so requested lengths are EXACT.
//!
//! BIP-173 requires a human-readable part of 1-83 characters for *decoder*
//! compatibility, but nothing in the checksum itself needs one. randid's
//! default output is therefore BARE: `<payload><checksum6>` with NO prefix
//! (checksum computed over the data alone, exactly BIP-350's
//! `bech32m_create_checksum(hrp="", ...)`). A namespace HRP is opt-in via
//! `-P`; `--strict` forces the standard (prefixed) form. `--verify` accepts
//! both bare and standard forms.

use bech32::primitives::iter::{ByteIterExt, Checksummed, Fe32IterExt};
use bech32::{Bech32, Bech32m, Fe32, Hrp};
use std::fmt;

/// BIP-173: a bech32(m) string is at most 90 characters.
pub const BECH32_MAX_LEN: usize = 90;

#[derive(Debug)]
pub enum Error {
    /// Requested length cannot fit the structural overhead.
    LengthTooSmall { requested: usize, minimum: usize },
    /// Effective entropy below the strict threshold (128 bits).
    StrictEntropy { effective_bits: usize },
    /// Request exceeds the BIP-173 90-character maximum.
    TooLong { total: usize },
    /// HRP outside BIP-173 validity (empty, non-ASCII, > 83 chars...).
    BadPrefix(String),
    /// bech32 classic has no bare form (BIP-173 requires an HRP).
    NoBareForClassic,
    /// OS entropy source failed.
    Entropy(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::LengthTooSmall { requested, minimum } => {
                write!(f, "length {requested} is below the minimum {minimum}")
            }
            Error::StrictEntropy { effective_bits } => {
                write!(f, "strict mode requires >= 128 bits (got {effective_bits})")
            }
            Error::TooLong { total } => {
                write!(
                    f,
                    "request exceeds the 90-character bech32 maximum ({total})"
                )
            }
            Error::BadPrefix(p) => write!(
                f,
                "prefix \"{p}\" is not a valid bech32 HRP (1-83 printable ASCII chars)"
            ),
            Error::NoBareForClassic => write!(
                f,
                "format bech32 (classic) has no bare form; use -P <prefix> or format bech32m"
            ),
            Error::Entropy(e) => write!(f, "entropy error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

/// Fill a buffer from the operating system CSPRNG.
fn entropy_bytes(n: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0u8; n];
    getrandom::fill(&mut buf).map_err(|e| Error::Entropy(e.to_string()))?;
    Ok(buf)
}

/// Structural overhead for a checksummed identifier: HRP + separator + 6.
#[must_use]
pub fn overhead(prefix: &str) -> usize {
    prefix.chars().count() + 1 + 6
}

fn parse_hrp(hrp: &str) -> Result<Hrp, Error> {
    Hrp::parse(hrp).map_err(|_| Error::BadPrefix(hrp.to_string()))
}

/// Encode exactly `payload_symbols` 5-bit symbols of fresh entropy under
/// the given checksum variant. `bytes_to_fes()` performs the crate's
/// byte->Fe32 packing (zero-padding only past the taken prefix), so the
/// output embeds exactly `5 * payload_symbols` random bits.
fn encode_with_entropy<Ck: bech32::Checksum>(
    hrp: &str,
    payload_symbols: usize,
) -> Result<String, Error> {
    let h = parse_hrp(hrp)?;
    let bytes = entropy_bytes((payload_symbols * 5).div_ceil(8))?;
    Ok(bytes
        .into_iter()
        .bytes_to_fes()
        .take(payload_symbols)
        .with_checksum::<Ck>(&h)
        .bytes()
        .map(|b| b as char)
        .collect())
}

/// Generate a bech32m identifier (BIP-350) with exactly `payload_symbols`
/// payload characters: `<hrp>1<payload><checksum6>`. Fully decodable by
/// any standard bech32m decoder.
pub fn bech32m_payload(hrp: &str, payload_symbols: usize) -> Result<String, Error> {
    encode_with_entropy::<Bech32m>(hrp, payload_symbols)
}

/// Generate a bech32 identifier (classic BIP-173 checksum) — legacy interop.
pub fn bech32_classic_payload(hrp: &str, payload_symbols: usize) -> Result<String, Error> {
    encode_with_entropy::<Bech32>(hrp, payload_symbols)
}

/// BARE bech32m identifier: `<payload><checksum6>` with NO prefix. The
/// crate's `Checksummed` engine computes the checksum over the data alone
/// (empty-HRP expansion, matching BIP-350's
/// `bech32m_create_checksum(hrp="", ...)`). Not decodable by standard
/// BIP-173 decoders (they require an HRP); verified by `verify_bech32m`.
pub fn bech32m_bare(payload_symbols: usize) -> Result<String, Error> {
    let bytes = entropy_bytes((payload_symbols * 5).div_ceil(8))?;
    Ok(
        Checksummed::<_, Bech32m>::new(bytes.into_iter().bytes_to_fes().take(payload_symbols))
            .map(Fe32::to_char)
            .collect(),
    )
}

/// Generate a plain base32 string (NO checksum) of exactly `n` characters,
/// for casual non-cryptographic identifiers.
pub fn base32_plain(n: usize) -> Result<String, Error> {
    let bytes = entropy_bytes((n * 5).div_ceil(8))?;
    Ok(bytes
        .into_iter()
        .bytes_to_fes()
        .take(n)
        .map(Fe32::to_char)
        .collect())
}

/// Verify a bech32(m) string. Two forms are accepted:
/// * standard `<hrp>1<payload><checksum6>`: HRP non-empty (BIP-173), valid
///   charset and checksum under either variant;
/// * bare `<payload><checksum6>` (no separator): checksum verified over the
///   data alone with the crate engine (randid's default form).
///
/// All-lower or all-upper presentation required; mixed case rejected.
#[must_use]
pub fn verify_bech32m(s: &str) -> bool {
    let lower = s.to_lowercase();
    let upper = s.to_uppercase();
    // BIP-173: mixed case is invalid; all-lower and all-upper are valid.
    if s != lower && s != upper {
        return false;
    }

    // Bare form: no separator anywhere.
    if !lower.contains('1') {
        if lower.chars().count() < 7 {
            return false; // checksum alone is 6; at least 1 entropy char
        }
        // feed only the payload body; the engine appends its own checksum
        let (body, _ck) = lower.split_at(lower.chars().count() - 6);
        let fes: Result<Vec<Fe32>, _> = body.chars().map(Fe32::from_char).collect();
        let Ok(fes) = fes else {
            return false;
        };
        let reencoded: String = Checksummed::<_, Bech32m>::new(fes.into_iter())
            .map(Fe32::to_char)
            .collect();
        return reencoded == lower;
    }

    let Some(seppos) = lower.rfind('1') else {
        return false;
    };
    let (hrp, data_part) = lower.split_at(seppos);
    let data_part = &data_part[1..];
    if hrp.is_empty() || data_part.chars().count() < 6 {
        return false;
    }
    let Ok(h) = Hrp::parse(hrp) else {
        return false;
    };
    // feed only the payload body; the crate appends its own 6-char checksum
    let (body, _ck) = data_part.split_at(data_part.chars().count() - 6);
    let fes: Result<Vec<Fe32>, _> = body.chars().map(Fe32::from_char).collect();
    let Ok(fes) = fes else {
        return false;
    };
    matches_checksum::<Bech32m>(&h, &fes, &lower) || matches_checksum::<Bech32>(&h, &fes, &lower)
}

fn matches_checksum<Ck: bech32::Checksum>(h: &Hrp, fes: &[Fe32], expected: &str) -> bool {
    fes.iter()
        .copied()
        .with_checksum::<Ck>(h)
        .bytes()
        .map(|b| b as char)
        .collect::<String>()
        == expected
}

/// Effective entropy accounting shared by the CLI (bech32m only).
#[must_use]
pub fn effective_bits(length: Option<usize>, prefix: &str, bits: usize, plain: bool) -> usize {
    match length {
        Some(l) if plain => 5 * l,
        Some(l) => 5 * l.saturating_sub(overhead(prefix)),
        None => bits,
    }
}

/// base58 / base58check token. Without a length target this is plain
/// 32-byte encoding; with a target, output is EXACTLY `target` chars
/// (retry loop absorbs the rare short encoding from leading zero bytes).
/// NOTE: truncating base58check output voids its checksum — same caveat
/// as the legacy shell implementation.
pub fn base58(target: Option<usize>, check: bool) -> Result<String, Error> {
    let encode_one = |n: usize| -> Result<String, Error> {
        let bytes = entropy_bytes(n)?;
        Ok(if check {
            bs58::encode(&bytes).with_check().into_string()
        } else {
            bs58::encode(&bytes).into_string()
        })
    };
    match target {
        None => encode_one(32),
        Some(t) => {
            let mut s = encode_one(t + 16)?;
            while s.chars().count() < t {
                s = encode_one(t + 16)?;
            }
            Ok(s.chars().take(t).collect())
        }
    }
}

/// base64 (standard, padded) token, exactly `target` chars when targeted.
pub fn base64(target: Option<usize>) -> Result<String, Error> {
    use base64::Engine as _;
    let encode_one = |n: usize| -> Result<String, Error> {
        Ok(base64::engine::general_purpose::STANDARD.encode(entropy_bytes(n)?))
    };
    match target {
        None => encode_one(32),
        Some(t) => {
            let s = encode_one((t * 3).div_ceil(4))?;
            Ok(s.chars().take(t).collect())
        }
    }
}

/// Lowercase hex token, exactly `target` chars when targeted (odd lengths
/// exact: the final nibble comes from the high half of a fresh byte).
pub fn hex(target: Option<usize>) -> Result<String, Error> {
    let encode_one = |n: usize| -> Result<String, Error> {
        Ok(entropy_bytes(n)?
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>())
    };
    match target {
        None => encode_one(32),
        Some(t) => {
            let s = encode_one(t.div_ceil(2))?;
            Ok(s.chars().take(t).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_default_shape() {
        let s = bech32m_bare(52).unwrap();
        assert_eq!(s.chars().count(), 58); // 52 payload + 6 checksum
        assert!(!s.contains('1'));
        assert!(verify_bech32m(&s));
    }

    #[test]
    fn prefixed_namespace() {
        let s = bech32m_payload("r32", 52).unwrap();
        assert_eq!(s.chars().count(), 62);
        assert!(s.starts_with("r321"));
        assert!(verify_bech32m(&s));
    }

    #[test]
    fn empty_hrp_is_rejected_not_panics() {
        assert!(matches!(bech32m_payload("", 52), Err(Error::BadPrefix(_))));
    }

    #[test]
    fn classic_variant_decodes_as_bech32() {
        let s = bech32_classic_payload("legacy", 20).unwrap();
        assert!(s.starts_with("legacy1"));
        assert!(verify_bech32m(&s)); // combined verifier accepts both variants
    }

    #[test]
    fn plain_base32_length() {
        for n in [2, 3, 5, 7, 8] {
            let s = base32_plain(n).unwrap();
            assert_eq!(s.chars().count(), n);
        }
    }

    #[test]
    fn base58_exact_and_hex_odd() {
        assert_eq!(base58(Some(12), false).unwrap().chars().count(), 12);
        assert_eq!(hex(Some(7)).unwrap().chars().count(), 7);
        assert_eq!(base64(Some(100)).unwrap().chars().count(), 100);
    }
}
