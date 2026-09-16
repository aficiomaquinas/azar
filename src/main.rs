use clap::{Parser, ValueEnum};
use randid::{
    base32_plain, base58, base64, bech32m_bare, bech32m_payload, effective_bits, hex, overhead,
    Error, BECH32_MAX_LEN,
};

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
    name = "randid",
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

    /// omit trailing newline (pipe-friendly: randid -n | wl-copy)
    #[arg(short = 'n', long)]
    no_newline: bool,

    /// verify bech32(m) strings read from stdin (bare or standard form);
    /// exit 0 if every line is valid, 1 otherwise
    #[arg(long)]
    verify: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verify {
        use std::io::Read;
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).expect("stdin");
        let ok = input.lines().all(|l| randid::verify_bech32m(l.trim()));
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
            eprintln!("randid: {e}");
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
                randid::bech32_classic_payload(&hrp, payload_symbols)?
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
