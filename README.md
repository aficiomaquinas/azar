# randid

Cryptographically random, human-safe identifier generator.
Single static binary; the shell functions `rand32` / `rand58` are thin
wrappers around it.

## Design

- **bech32m** output follows **BIP-350** (checksum constant `0x2bc830a3`)
  via the [`bech32` crate](https://crates.io/crates/bech32) (rust-bitcoin) —
  no hand-rolled crypto. The BCH checksum guarantees detection of any error
  affecting up to 4 characters (< 1e-9 false-negative beyond that).
- Entropy comes from the OS CSPRNG (`getrandom` crate — same source as
  `/dev/urandom` via the standard library path).
- Length sampling is **bit-level**: exactly 5 entropy bits per output
  character, so `--length N` is **exactly N characters, always**.
- Charset excludes `1`, `b`, `i`, `o` (visual ambiguity) and is all-lower:
  safe for double-click, URLs, and voice.

## Install

```bash
cargo install --path .
```

## Usage

```bash
randid                                 # bech32m bare, 256-bit (58 chars)
randid -l 12                           # exactly 12 chars (checksummed)
randid -P r32                          # HRP namespace: r321<52><6>
randid -P id- -l 15 -f base58          # literal prefix, base58
randid -f hex -l 7                     # odd lengths exact
randid -s                              # enforce >= 128 bits effective
randid -n | wl-copy                    # pipe-friendly (no newline)
echo "r321..." | randid --verify       # exit 0/1 checksum validation
```

Formats: `bech32m` (default) · `bech32` (legacy BIP-173) · `base58` ·
`base58check` (NOT truncation-safe with `-l`) · `base64` · `hex`.

Entropía efectiva con `-l`: 5 bits por carácter de payload (los 6 de
checksum no aportan entropía). Con `-s` se exige >= 128 bits efectivos.

## Shell wrappers

`~/.config/shell/functions.sh` (repo `shell-config`) wraps this binary:

```sh
rand32()  { exec randid "$@"; }
rand58()  { exec randid -f base58 "$@"; }   # transitional; maps flags
```

## Development

```bash
cargo test        # unit + BIP-350 vectors (incl. exhaustive single-char
                  # mutation detection) + proptest length contracts
cargo clippy -- -D warnings
cargo fmt --check
cargo audit
```

## License

MIT
