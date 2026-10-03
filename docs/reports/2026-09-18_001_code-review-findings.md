# Code review findings — azar: reproduction, fixes, post-fix status

- Date: 2026-09-18
- Repo: `~/Documents/DevelopmentV2/azar` (public: github.com/aficiomaquinas/azar)
- Head reviewed: `1e6a1b9` (master) — review task t_d29c05ff, report `REVIEW-azar-2026-09-17.md`
- Fix commit: `f494022` `fix(cli): close review findings F1-F6 — verify panics, plain path, degenerate lengths`
- Fix task: t_da9b01f9
- Method: every defect from the 2026-09-17 review was reproduced on `1e6a1b9` before fixing, and every repro was re-executed after the fix to confirm negation.

## Verdict

All six findings from the 2026-09-17 review are fixed at origin, plus review item R1 (non-UTF-8 stdin panic). Quality gates are green post-fix and the generator ⇔ verifier contract is now closed: every output azar can generate is accepted by its own `--verify`.

One finding (F5) embeds an operator decision (D2) that was NOT decided at fix time; the safe side was implemented and is **pending operator ratification** — see §D2.

## Quality gates

| Gate | Before (1e6a1b9) | After (f494022) |
|---|---|---|
| `cargo fmt --check` | PASS | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS | PASS (0 warnings) |
| `cargo test` | 22/22 | **23/23** (9 lib + 4 bin + 6 props + 4 vectors) |

New regression tests: `verifier_is_multibyte_safe` (F1), `verifier_rejects_empty_and_degenerate`, `minimum_bare_length_roundtrips` (F5), and the bare proptest range extended down to the new floor (7..=84).

## Findings: repro (before) → behavior (after)

### F1 — HIGH: `--verify` panicked on multibyte input
- Where: `src/lib.rs:206,229` — `split_at(lower.chars().count() - 6)` used a CHAR count as a BYTE index.
- Repro (before): `printf 'ñññññññ\n' | azar --verify` → panic `byte index 1 is not a char boundary`, exit 101.
- Fix: `verify_bech32m` is now ASCII-gated (`!s.is_ascii()` → `false`) before any byte-indexing split; both splits use `len()` (bytes) on validated ASCII, where char position == byte position.
- After: same repro → `0 of 1 token(s) verified` on stderr, **exit 1**. Negated.

### F2 — HIGH: underflow panic in `target_len` when `-l` < prefix (literal formats)
- Where: `src/main.rs:265` — `l - prefix.chars().count()` without `checked_sub`.
- Repro (before): `azar -f hex -P ab -l 1` → panic `attempt to subtract with overflow`, exit 101.
- Fix: `target_len` returns `Result<Option<usize>, Error>`; underflow → `LengthTooSmall { requested, minimum: prefix_len + 1 }` (same error family the bech32 path already used).
- After: `azar -f hex -P ab -l 1` → `azar: length 1 is below the minimum 3`, exit 1. Same for base58/base64/base58check. Negated.

### F3 — MEDIUM: `--verify` succeeded on empty input
- Where: `src/main.rs:154` — `input.lines().all(...)` vacuously true for zero lines.
- Repro (before): `printf '' | azar --verify` → exit 0.
- Fix: empty stdin is a fail-fast error (`Error::EmptyInput`); non-UTF-8 stdin no longer panics (review item R1 — `read_to_string` error → clean exit 1); verify always prints an `N of M token(s) verified` summary line (stderr).
- After: same repro → `azar: empty input: expected at least one token`, exit 1. Negated.

### F4 — MEDIUM: `-f bech32 -l <8` silently produced checksum-less output
- Where: `src/main.rs:188` — the `plain` branch was selected before the `NoBareForClassic` check (`main.rs:240-242`).
- Repro (before): `azar -f bech32 -l 5` → `n346s` (plain base32, no checksum, exit 0).
- Fix: the checksum-less `plain` path is REMOVED entirely (see F5/D2). `NoBareForClassic` now wins for classic bech32 at ANY `-l`, as the documented invariant always said.
- After: same repro → `azar: length 5 is below the minimum 7`, exit 1. Negated.

### F5 — MEDIUM (D2): `-l 7` generated tokens its own `--verify` rejected
- Repro (before): `azar -l 7 | azar --verify` → exit 1 (generator emitted plain base32; verifier demanded a 6-char checksum).
- Fix — **D2 resolved safe-side, PENDING OPERATOR RATIFICATION**: option (b) from the review — every azar output is checksummed and verifiable. Bare minimum length drops 8 → 7 (1 payload char + 6 checksum), which is exactly the verifier's structural floor. Generator floor == verifier floor: `azar -l N | azar --verify` holds for every N the generator accepts. Requests below 7 are a clean `LengthTooSmall` error (minimum 7); with `-P`, the minimum grows by `overhead(hrp)` as before.
- After: `azar -l 7` → `wc6clv0`; piped to `--verify` → `1 of 1 token(s) verified`, exit 0. `azar -l 6` → `azar: length 6 is below the minimum 7`, exit 1. Negated.
- **Operator note (D2)**: options were (a) verifier tolerates plain short tokens, (b) drop `plain`, checksummed floor 7 (IMPLEMENTED), (c) document as-is. Rationale for (b): restores the core invariant "every azar output verifies"; (a) would weaken the verifier's meaning and (c) leaves the contract split in place. To ratify a different option, change `src/main.rs` (`run()`, bare branch) and `src/lib.rs` doc-comment together — the floors are: bare `7 = 1 + 6`, prefixed `overhead(hrp) + 1`.
- **RATIFIED 2026-10-03 — operator decision, option (d), beyond (a)-(c)**: permissive default with decomposed strict flags. The CLI now emits a CHECKSUMMED token whenever the checksum fits (bare >= 7, prefixed >= `overhead(hrp) + 1` — settling the 9-vs-10 prefixed floor question in favor of `+1`, because permissive forbids falling back to plain when a checksum IS possible) and falls back to checksum-less output only below that floor (`-l 1..6` bare; `hrp + 1 + 1` floor prefixed). Strictness was decomposed out of the removed `-s/--strict`: `--min-bits N` (per-format entropy accounting: exact5 bits/char bech32(m), 4 hex, conservative base58/base64), `--standard` (form, bech32(m) only), `--checksum` (verifiable output; also rejects `-f base58check -l`, which voids the check). All three fail POSIX-style: stderr, non-zero exit, empty stdout. `-l 0` stays `ZeroLength`; the 90-char BIP cap stays; `-f bech32` without `-P` errors at ANY `-l` (F4's stated intent — the old length check used to mask it). The generator NEVER emits a zero-payload token; `--verify` stays BIP-350-faithful and accepts the official zero-payload vector `?1v759aa` (it validates well-formedness, not entropy — `--min-bits` owns entropy). Shipped in0.2.0 (breaking).

### F6 — LOW: degenerate zero-length requests succeeded
- Repro (before): `azar -l 0` → exit 0, empty stdout; `-b 0` → checksum-only output (6 chars, ZERO entropy), exit 0.
- Fix: one guard in `run()` — `-l 0` / `-b 0` / `-B 0` → `Error::ZeroLength` (`zero-length request: -l/-b/-B must be >= 1`).
- After: all three → clean error, exit 1 (any format). Negated.

## Behavior changes / compatibility notes

- BREAKING: `azar -l <7` (bare) no longer prints a checksum-less token — it errors. Scripts using e.g. `-l 6` must move to `-l 7` (now verifiable) or another format. The README example changed accordingly.
- BREAKING: `-f bech32` without `-P` errors (`NoBareForClassic`) at every `-l`, not only `-l >= 8`.
- `--verify` prints a `N of M token(s) verified` summary to stderr (stdout stays clean for piping).
- `-b 0` / `-B 0` / `-l 0` are hard errors.
- Unchanged (re-verified after the fix): exact lengths for all formats incl. odd hex, uppercase verify roundtrip, strict entropy accounting (`-s -l 32` still rejects at 120 eff bits), default 58-char bare output, `--flip`, `-R MIN MAX`.

## Review items intentionally not acted on

- R2 (strict entropy counts 5 bits/char for non-bech formats): conservative direction, comment-only per the review — no action in this change.
- CI minor note (`cargo-audit` install swallowed by `|| true`): separate concern, not batched into this fix.
