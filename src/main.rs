use clap::{Parser, ValueEnum};
use randid::{
    base32_plain, base58, base64, bech32m_payload, effective_bits, hex, overhead, Error,
    BECH32_MAX_LEN, DEFAULT_HRP,
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
                  bech32m output is bit-level length-exact and uses the BIP-350 checksum\n\
                  (guaranteed detection of up to 4 character errors). Default namespace\n\
                  HRP is \"r\": r1<payload><checksum6>."
)]
struct Cli {
    /// output format
    #[arg(short, long, value_enum, default_value_t = Format::Bech32m)]
    format: Format,

    /// namespace HRP for bech32(m) (default \"r\"); literal prefix otherwise
    #[arg(short = 'P', long)]
    prefix: Option<String>,

    /// total printed length INCLUDING prefix, separator and checksum
    /// (bech32m) or prefix (others). bech32m output is EXACTLY this long.
    #[arg(short = 'l', long)]
    length: Option<usize>,

    /// entropy in bits (default 256)
    #[arg(short = 'b', long, conflicts_with = "bytes")]
    bits: Option<usize>,

    /// entropy in bytes (overrides -b)
    #[arg(short = 'B', long)]
    bytes: Option<usize>,

    /// strict: require at least 128 bits of effective entropy
    #[arg(short = 's', long)]
    strict: bool,

    /// omit trailing newline (pipe-friendly: randid -n | wl-copy)
    #[arg(short = 'n', long)]
    no_newline: bool,

    /// verify bech32(m) strings read from stdin instead of generating
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
    let hrp = cli
        .prefix
        .clone()
        .unwrap_or_else(|| DEFAULT_HRP.to_string());
    // plain (no checksum) only for short, prefix-less bech32m requests
    let plain = is_bech && cli.prefix.is_none() && cli.length.is_some_and(|l| l < 8);

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
            let ovh = overhead(&hrp);
            let payload_symbols = match cli.length {
                Some(l) => {
                    if plain {
                        l
                    } else {
                        match l.checked_sub(ovh) {
                            Some(t) if t >= 2 => t,
                            _ => {
                                return Err(Error::LengthTooSmall {
                                    requested: l,
                                    minimum: ovh + 2,
                                })
                            }
                        }
                    }
                }
                None => bits_arg.div_ceil(5),
            };
            let total = ovh + payload_symbols;
            if total > BECH32_MAX_LEN {
                return Err(Error::TooLong { total });
            }
            if plain {
                base32_plain(payload_symbols)?
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

    Ok(match &cli.prefix {
        // bech32(m) already embeds the prefix as HRP; literal formats get it prepended
        Some(p) if !is_bech => format!("{p}{token}"),
        _ => token,
    })
}

fn target_len(cli: &Cli) -> Option<usize> {
    cli.length
        .map(|l| l - cli.prefix.as_ref().map_or(0, |p| p.chars().count()))
}
