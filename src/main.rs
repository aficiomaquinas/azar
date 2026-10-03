use azar::{base58, base64, bech32m_bare, bech32m_payload, hex, overhead, Error, BECH32_MAX_LEN};
use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    /// bech32m per BIP-350 (default): charset without 1/b/i/o, 6-char BCH checksum
    Bech32m,
    /// bech32 classic checksum (BIP-173), only for legacy interop
    Bech32,
    /// base58 (Bitcoin alphabet)
    Base58,
    /// base58check (double-SHA256 checksum; NOT truncation-safe with -l)
    Base58check,
    /// standard padded base64
    Base64,
    /// lowercase hex
    Hex,
}

#[derive(Parser)]
#[command(
    name = "azar",
    version,
    about = "Random identifier generator — bech32m / base58 / base64 / hex",
    long_about = "Cryptographically random, human-safe identifiers.\n\
                  Default output is BARE bech32m: <payload><checksum6> — no prefix, no separator,\n\
                  lowercase, never contains 1/b/i/o. Lengths are EXACT (bit-level sampling).\n\
                  Use -P for a namespace (standard bech32m form, decodable everywhere).\n\
                  The default is PERMISSIVE: any length is produced; lengths too short for the\n\
                  6-char checksum are emitted checksum-less (not verifiable) rather than\n\
                  erroring, so no length breaks a pipeline. Guarantees are opt-in via strict\n\
                  flags — -m/--min-bits N (entropy), --standard (form), --checksum (verifiable\n\
                  output); violations are standard errors: stderr, non-zero exit, no stdout."
)]
struct Cli {
    /// output format
    #[arg(short, long, value_enum, default_value_t = Format::Bech32m)]
    format: Format,

    /// namespace HRP (bech32(m): standard <hrp>1<payload><ck> form; other
    /// formats: literal prefix). Default: none — bare output.
    #[arg(short = 'P', long)]
    prefix: Option<String>,

    /// total printed length INCLUDING prefix, separator and checksum
    /// (bech32m) or prefix (others). Output is EXACTLY this long. Below
    /// the checksum floor (7 bare / overhead+1 prefixed) bech32 output is
    /// checksum-less unless --checksum forbids it
    #[arg(short = 'l', long)]
    length: Option<usize>,

    /// entropy in bits (default 256; with -l the effective entropy is
    /// derived from the output length instead and -b/-B are ignored)
    #[arg(short = 'b', long, conflicts_with = "bytes")]
    bits: Option<usize>,

    /// entropy in bytes (overrides -b)
    #[arg(short = 'B', long)]
    bytes: Option<usize>,

    /// strict guarantee: require at least N bits of effective entropy
    /// (exact for bech32(m), conservative for base58/base64, 4 bits/char
    /// for hex); on violation: stderr error, non-zero exit, empty stdout
    #[arg(short = 'm', long, value_name = "N")]
    min_bits: Option<usize>,

    /// strict guarantee: require the standard bech32(m) form
    /// <hrp>1<payload><ck> — implies `-P r` when no -P is given
    #[arg(long)]
    standard: bool,

    /// strict guarantee: require verifiable (checksummed) output — errors
    /// when the length cannot carry the 6-char checksum or the format has
    /// no checksum at all
    #[arg(long)]
    checksum: bool,

    /// omit trailing newline (pipe-friendly: azar -n | wl-copy)
    #[arg(short = 'n', long)]
    no_newline: bool,

    /// verify bech32(m) strings read from stdin (bare or standard form);
    /// exit 0 if every line is valid, 1 otherwise
    #[arg(long)]
    verify: bool,

    /// create symlinks for the canonical alias family (busybox-style)
    #[arg(long)]
    install_aliases: bool,

    /// remove the canonical alias symlinks
    #[arg(long)]
    uninstall_aliases: bool,

    /// target directory for alias symlinks (default: ~/.local/bin)
    #[arg(long, value_name = "DIR")]
    aliases_dir: Option<String>,

    /// print the story behind each canonical alias name and exit
    #[arg(long)]
    lore: bool,

    /// one fair coin flip: prints "true" or "false" (bash-native — usable
    /// directly in if/&&/||); exits 1 with an error if the CSPRNG fails,
    /// never fabricating a result. See README for piping examples.
    #[arg(long)]
    flip: bool,

    /// uniform random integer in [MIN, MAX] inclusive (rejection-sampled,
    /// no modulo bias): azar -R 1 6. Negative bounds OK.
    #[arg(
        short = 'R',
        long,
        num_args = 2,
        value_names = ["MIN", "MAX"],
        allow_negative_numbers = true
    )]
    randbetween: Option<Vec<i64>>,
}

fn main() {
    let cli = Cli::parse();

    if cli.lore {
        print!("{}", lore_text());
        return;
    }

    if cli.flip {
        match azar::flip() {
            Ok(v) => println!("{}", flip_str(v)),
            Err(e) => {
                eprintln!("azar: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(bounds) = &cli.randbetween {
        if bounds.len() != 2 {
            eprintln!("azar: --randbetween needs exactly MIN and MAX");
            std::process::exit(1);
        }
        match azar::rand_between(bounds[0], bounds[1]) {
            Ok(v) => println!("{v}"),
            Err(e) => {
                eprintln!("azar: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    if cli.install_aliases || cli.uninstall_aliases {
        let dir = aliases_dir(&cli);
        let action = if cli.install_aliases {
            install_aliases(&dir)
        } else {
            uninstall_aliases(&dir)
        };
        match action {
            Ok(msg) => println!("{}", msg),
            Err(e) => {
                eprintln!("azar: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    if cli.verify {
        use std::io::Read;
        let mut input = String::new();
        if std::io::stdin().read_to_string(&mut input).is_err() {
            eprintln!("azar: stdin is not valid UTF-8");
            std::process::exit(1);
        }
        let lines: Vec<&str> = input.lines().map(str::trim).collect();
        if lines.is_empty() {
            eprintln!("azar: {}", Error::EmptyInput);
            std::process::exit(1);
        }
        let verified = lines.iter().filter(|l| azar::verify_bech32m(l)).count();
        let ok = verified == lines.len();
        eprintln!("{verified} of {} token(s) verified", lines.len());
        std::process::exit(if ok { 0 } else { 1 });
    }

    match run(&cli) {
        Ok(token) => {
            print!("{token}");
            if !cli.no_newline {
                println!();
            }
        }
        Err(e) => {
            eprintln!("azar: {e}");
            std::process::exit(1);
        }
    }
}

fn run(cli: &Cli) -> Result<String, Error> {
    let is_bech = matches!(cli.format, Format::Bech32m | Format::Bech32);

    // ---- the standard form forces the namespace HRP ----
    let hrp = if is_bech {
        match (&cli.prefix, cli.standard) {
            // an explicitly empty -P is the same as no -P at all, so
            // --standard still implies the `-r` namespace instead of
            // silently emitting bare output while claiming standard
            (Some(p), _) if !p.is_empty() => p.clone(),
            (_, true) => "r".to_string(),
            _ => String::new(), // bare
        }
    } else {
        cli.prefix.clone().unwrap_or_default()
    };
    let prefixed = !hrp.is_empty();

    // Degenerate zero-length requests are rejected outright: -l 0 would
    // print nothing, and -b 0 / -B 0 without -l would emit a checksum (or
    // token) with ZERO entropy characters.
    if cli.length == Some(0) || cli.bits == Some(0) || cli.bytes == Some(0) {
        return Err(Error::ZeroLength);
    }

    // BIP-173 caps every bech32(m) string at 90 characters — a hard
    // property of the standard, independent of the length branch below
    if is_bech && cli.length.is_some_and(|l| l > BECH32_MAX_LEN) {
        return Err(Error::TooLong {
            total: cli.length.unwrap_or_default(),
        });
    }

    // format impossibility wins before any length arithmetic: BIP-173
    // defines no bare form for classic bech32 (it always needs an HRP)
    if is_bech && !prefixed && cli.format == Format::Bech32 {
        return Err(Error::NoBareForClassic);
    }

    // ---- strict flags: each enforces exactly ONE property and fails
    // POSIX-style — message on stderr, non-zero exit, NOTHING on stdout,
    // so a failing guarantee empties the pipe instead of corrupting it ----
    if cli.standard && !is_bech {
        return Err(Error::StandardUnsupported(format!("{:?}", cli.format)));
    }
    if cli.checksum {
        match cli.format {
            Format::Bech32m | Format::Bech32 => {
                if let Some(l) = cli.length {
                    let ck_ovh = if prefixed { overhead(&hrp) } else { 6 };
                    if l <= ck_ovh {
                        return Err(Error::ChecksumUnavailable {
                            requested: l,
                            minimum: ck_ovh + 1, // + 1 payload char
                        });
                    }
                }
            }
            Format::Base58check => {
                if cli.length.is_some() {
                    // -l truncates base58check and voids its checksum
                    return Err(Error::ChecksumVoided);
                }
            }
            other => return Err(Error::ChecksumUnsupported(format!("{other:?}"))),
        }
    }
    if let Some(required) = cli.min_bits {
        let eff = effective_bits(cli, &hrp, prefixed, is_bech);
        if eff < required {
            return Err(Error::StrictEntropy {
                effective_bits: eff,
                required,
            });
        }
    }

    // ---- entropy budget (no -l requests) ----
    let bits_arg = match (cli.bits, cli.bytes) {
        (Some(b), _) => b,
        (None, Some(bytes)) => bytes * 8,
        (None, None) => 256,
    };

    // ---- per-format generation ----
    let token = match cli.format {
        Format::Bech32m | Format::Bech32 => {
            // Permissive default (operator-ratified 2026-10-03, review
            // F5/D2): whenever the 6-char checksum FITS (>= 1 payload
            // char — bare >= 7, prefixed >= overhead+1) the output is
            // checksummed and verifiable; below that floor it is emitted
            // checksum-less instead of erroring, so no requested length
            // ever breaks a pipeline. --checksum turns this fallback into
            // an error. The generator never emits a zero-payload token.
            let ck_ovh = if prefixed { overhead(&hrp) } else { 6 };
            if let Some(l) = cli.length {
                if l > ck_ovh {
                    // checksummed branch — payload >= 1, exactly what
                    // --verify accepts (l == ck_ovh would be a ZERO
                    // payload token: checksum of nothing, no entropy)
                    let payload = l - ck_ovh;
                    if !prefixed {
                        bech32m_bare(payload)?
                    } else if cli.format == Format::Bech32 {
                        azar::bech32_classic_payload(&hrp, payload)?
                    } else {
                        bech32m_payload(&hrp, payload)?
                    }
                } else {
                    // below the floor: checksum-less (permissive). The
                    // prefix + separator must still fit >= 1 payload char.
                    let plain_ovh = if prefixed { hrp.chars().count() + 1 } else { 0 };
                    let plain_payload = l.checked_sub(plain_ovh).filter(|&t| t >= 1).ok_or(
                        Error::LengthTooSmall {
                            requested: l,
                            minimum: plain_ovh + 1,
                        },
                    )?;
                    if !prefixed {
                        azar::base32_plain(plain_payload)?
                    } else {
                        azar::plain_prefixed(&hrp, plain_payload)?
                    }
                }
            } else {
                // no -l: bit-budget request — always checksummed (the
                // default 256-bit payload sits far above every floor)
                let payload = bits_arg.div_ceil(5);
                let total = payload + ck_ovh;
                if total > BECH32_MAX_LEN {
                    return Err(Error::TooLong { total });
                }
                if !prefixed {
                    bech32m_bare(payload)?
                } else if cli.format == Format::Bech32 {
                    azar::bech32_classic_payload(&hrp, payload)?
                } else {
                    bech32m_payload(&hrp, payload)?
                }
            }
        }
        Format::Base58 => base58(target_len(cli)?, false)?,
        Format::Base58check => base58(target_len(cli)?, true)?,
        Format::Base64 => base64(target_len(cli)?)?,
        Format::Hex => hex(target_len(cli)?)?,
    };

    Ok(match (&cli.prefix, is_bech) {
        // bech32(m) already embeds the prefix as HRP; literal formats get it prepended
        (Some(p), false) => format!("{p}{token}"),
        _ => token,
    })
}

/// Effective entropy of what the generator will actually emit, per
/// format: exact 5 bits/char for the bech32(m) payload, 4 for hex,
/// ~6 for base64 (conservative min), a conservative floor for base58
/// (log2(58) ~ 5.86). Without -l the requested budget (-b/-B/default
/// 256) is what --min-bits sees.
fn effective_bits(cli: &Cli, hrp: &str, prefixed: bool, is_bech: bool) -> usize {
    let Some(l) = cli.length else {
        return match (cli.bits, cli.bytes) {
            (Some(b), _) => b,
            (None, Some(bytes)) => bytes * 8,
            (None, None) => 256,
        };
    };
    if is_bech {
        let ck_ovh = if prefixed { overhead(hrp) } else { 6 };
        let plain_ovh = if prefixed { hrp.chars().count() + 1 } else { 0 };
        // checksummed whenever it fits (checksum chars carry no entropy);
        // every printed char is payload on the plain path below the floor
        if l > ck_ovh {
            5 * (l - ck_ovh)
        } else {
            5 * l.saturating_sub(plain_ovh)
        }
    } else {
        let prefix_len = cli.prefix.as_ref().map_or(0, |p| p.chars().count());
        let t = l.saturating_sub(prefix_len);
        match cli.format {
            Format::Hex => 4 * t,
            Format::Base64 => (6 * t).min(8 * (t * 3).div_ceil(4)),
            Format::Base58 | Format::Base58check => 5 * t,
            Format::Bech32m | Format::Bech32 => 0, // unreachable: is_bech
        }
    }
}

fn target_len(cli: &Cli) -> Result<Option<usize>, Error> {
    let Some(l) = cli.length else {
        return Ok(None);
    };
    let prefix_len = cli.prefix.as_ref().map_or(0, |p| p.chars().count());
    let target = l.checked_sub(prefix_len).ok_or(Error::LengthTooSmall {
        requested: l,
        minimum: prefix_len + 1,
    })?;
    Ok(Some(target))
}

/// Canonical alias family (busybox-style). Every alias runs the exact same
/// engine: a secret tool must never vary its output by the name it is
/// invoked with. Symlinks are created by --install-aliases.
/// (The primary name, azar, needs no symlink — it IS the binary.)
const CANONICAL_ALIASES: &[&str] = &["bola8", "dado", "ficha", "gettone", "precinto"];

fn lore_text() -> String {
    let mut out = String::from("canonical alias family — every name runs the same engine:\n\n");
    for name in CANONICAL_ALIASES {
        let story = match *name {
            "bola8" => "the magic 8-ball: ask, shake, receive an answer you did not choose.",
            "dado" => "the die — the oldest randomness device worth trusting.",
            "ficha" => "the token: what you hand over when identity matters.",
            "gettone" => "the Italian payphone token: a small coin that meant connection granted.",
            "precinto" => "the tamper-evident seal: if it verifies, nobody touched it in transit.",
            _ => unreachable!("alias list is closed"),
        };
        out.push_str(&format!("  {name:10} {story}\n"));
    }
    out.push_str("\ninstall them with: azar --install-aliases\n");
    out
}

/// Bash-native boolean spelling: the exact strings `test`/`if` consume.
fn flip_str(v: bool) -> &'static str {
    if v {
        "true"
    } else {
        "false"
    }
}

fn aliases_dir(cli: &Cli) -> std::path::PathBuf {
    if let Some(d) = &cli.aliases_dir {
        return std::path::PathBuf::from(d);
    }
    let home = home::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(".local").join("bin")
}

fn install_aliases(dir: &std::path::Path) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate own binary: {e}"))?;
    let mut linked = 0usize;
    let mut skipped = 0usize;
    for name in CANONICAL_ALIASES {
        let link = dir.join(name);
        if link.symlink_metadata().is_ok() {
            if std::fs::read_link(&link).is_ok_and(|t| t == exe) {
                skipped += 1; // already ours
                continue;
            }
            skipped += 1; // occupied by something else: never clobber
            continue;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&exe, &link)
            .map_err(|e| format!("cannot link {}: {e}", link.display()))?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&exe, &link)
            .map_err(|e| format!("cannot link {}: {e}", link.display()))?;
        linked += 1;
    }
    Ok(format!(
        "{} alias(es) linked into {}, {} already present",
        linked,
        dir.display(),
        skipped
    ))
}

fn uninstall_aliases(dir: &std::path::Path) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate own binary: {e}"))?;
    let mut removed = 0usize;
    let mut kept = 0usize;
    for name in CANONICAL_ALIASES {
        let link = dir.join(name);
        let Ok(target) = std::fs::read_link(&link) else {
            continue; // not a symlink (or absent): never touch
        };
        if target == exe {
            std::fs::remove_file(&link)
                .map_err(|e| format!("cannot remove {}: {e}", link.display()))?;
            removed += 1;
        } else {
            kept += 1;
        }
    }
    Ok(format!(
        "{} alias(es) removed, {} kept (not ours)",
        removed, kept
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_family_is_sane() {
        assert_eq!(CANONICAL_ALIASES.len(), 5);
        let mut sorted = CANONICAL_ALIASES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            CANONICAL_ALIASES.len(),
            "aliases must be unique"
        );
        // symlink-safe names: [a-z0-9] only
        assert!(CANONICAL_ALIASES.iter().all(|a| !a.is_empty()
            && a.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())));
    }

    #[test]
    fn lore_mentions_every_alias() {
        let lore = lore_text();
        for name in CANONICAL_ALIASES {
            assert!(lore.contains(name), "lore must mention {name}");
        }
    }

    #[test]
    fn lore_never_promises_behavior_differences() {
        assert!(lore_text().contains("same engine"));
    }

    #[test]
    fn flip_output_is_bash_native() {
        // the exact strings bash if/&&/|| consume
        assert_eq!(flip_str(true), "true");
        assert_eq!(flip_str(false), "false");
    }
}
