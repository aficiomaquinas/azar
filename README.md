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
randid                                 # BARE bech32m: <payload><ck6>, 256-bit (58 chars)
randid -l 12                           # exactly 12 chars, bare + checksum
randid -l 6 -n | wl-copy               # casual 6-char token, clipboard
randid -P r32                          # namespace: r321<payload><ck6> (standard form)
randid -s                              # strict: standard form + >= 128 bits
randid -f bech32 -P legacy             # classic BIP-173 checksum (needs -P)
randid -f base58 -l 15                 # base58 / base58check / base64 / hex
randid -f hex -l 7                     # odd lengths exact
echo "<token>" | randid --verify       # validates bare AND standard forms
```

Effective entropy with `-l` is 5 bits per payload char (the 6 checksum
chars carry none); `--strict` enforces >= 128 effective bits and the
standard (prefixed) form. All errors go to stderr with exit code 1.

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
