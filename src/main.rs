use azar::{
    base32_plain, base58, base64, bech32m_bare, bech32m_payload, effective_bits, hex, overhead,
    Error, BECH32_MAX_LEN,
};
use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
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
                  Use -P for a namespace (standard bech32m form, decodable everywhere) and\n\
                  --strict for a cryptographic-grade token (>= 128 bits, standard form)."
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
    /// (bech32m) or prefix (others). Output is EXACTLY this long.
    #[arg(short = 'l', long)]
    length: Option<usize>,

    /// entropy in bits (default 256; bech32(m) uses 5 bits per payload char
    /// when -l is given, so -b is then only a strict-mode floor)
    #[arg(short = 'b', long, conflicts_with = "bytes")]
    bits: Option<usize>,

    /// entropy in bytes (overrides -b)
    #[arg(short = 'B', long)]
    bytes: Option<usize>,

    /// strict: require >= 128 bits of effective entropy AND the standard
    /// (prefixed) bech32m form — implies `-P r` when no -P is given
    #[arg(short = 's', long)]
    strict: bool,

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
    /// directly in if/&&/||). See README for piping examples.
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
        println!("{}", flip_str(azar::flip()));
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
        std::io::stdin().read_to_string(&mut input).expect("stdin");
        let ok = input.lines().all(|l| azar::verify_bech32m(l.trim()));
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

    // ---- strict mode forces the standard (prefixed) form ----
    let hrp = if is_bech {
        match (&cli.prefix, cli.strict) {
            (Some(p), _) => p.clone(),
            (None, true) => "r".to_string(),
            (None, false) => String::new(), // bare
        }
    } else {
        cli.prefix.clone().unwrap_or_default()
    };
    let prefixed = !hrp.is_empty();

    // plain (no checksum) only for short prefix-less bech32m requests
    let plain = is_bech && !prefixed && cli.length.is_some_and(|l| l < 8);

    // ---- entropy budget ----
    let bits_arg = match (cli.bits, cli.bytes) {
        (Some(b), _) => b,
        (None, Some(bytes)) => bytes * 8,
        (None, None) => 256,
    };

    if cli.strict {
        let eff = effective_bits(cli.length, &hrp, bits_arg, plain);
        if eff < 128 {
            return Err(Error::StrictEntropy {
                effective_bits: eff,
            });
        }
    }

    // ---- per-format generation ----
    let token = match cli.format {
        Format::Bech32m | Format::Bech32 => {
            let payload_symbols = if plain {
                cli.length.expect("plain implies -l")
            } else if let Some(l) = cli.length {
                if !prefixed {
                    // bare + checksummed: overhead is just the 6-char checksum
                    l.checked_sub(6)
                        .filter(|&t| t >= 2)
                        .ok_or(Error::LengthTooSmall {
                            requested: l,
                            minimum: 8,
                        })?
                } else {
                    let ovh = overhead(&hrp);
                    l.checked_sub(ovh)
                        .filter(|&t| t >= 2)
                        .ok_or(Error::LengthTooSmall {
                            requested: l,
                            minimum: ovh + 2,
                        })?
                }
            } else {
                bits_arg.div_ceil(5)
            };
            let total = payload_symbols + if prefixed { overhead(&hrp) } else { 6 };
            if total > BECH32_MAX_LEN {
                return Err(Error::TooLong { total });
            }
            if plain {
                base32_plain(payload_symbols)?
            } else if !prefixed {
                // only bech32m has a defined bare form
                if cli.format == Format::Bech32 {
                    return Err(Error::NoBareForClassic);
                }
                bech32m_bare(payload_symbols)?
            } else if cli.format == Format::Bech32 {
                azar::bech32_classic_payload(&hrp, payload_symbols)?
            } else {
                bech32m_payload(&hrp, payload_symbols)?
            }
        }
        Format::Base58 => base58(target_len(cli), false)?,
        Format::Base58check => base58(target_len(cli), true)?,
        Format::Base64 => base64(target_len(cli))?,
        Format::Hex => hex(target_len(cli))?,
    };

    Ok(match (&cli.prefix, is_bech) {
        // bech32(m) already embeds the prefix as HRP; literal formats get it prepended
        (Some(p), false) => format!("{p}{token}"),
        _ => token,
    })
}

fn target_len(cli: &Cli) -> Option<usize> {
    cli.length
        .map(|l| l - cli.prefix.as_ref().map_or(0, |p| p.chars().count()))
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
