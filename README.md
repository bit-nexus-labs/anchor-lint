# anchor-lint

A tiny text-based linter for Anchor (Solana) programs. The code was written by AI coding agents, and this README explains it.

## 1. What it does

`anchor-lint` reads one `.rs` file and checks it line by line. It does not understand Rust, it only looks at the text. It prints every line that looks risky, with a rule name. It can be useful as a first quick pass when I read Anchor code during a security review.

```
$ cargo run -- samples/bad.rs
samples/bad.rs:9: [R2] unchecked arithmetic
  vault.total = vault.total + amount;
samples/bad.rs:11: [R1] may panic
  let fee = amount.checked_mul(2).unwrap();
samples/bad.rs:12: [R2] unchecked arithmetic
  vault.fees += fee;
samples/bad.rs:21: [R3] account without CHECK comment
  pub authority: AccountInfo<'info>,
4 finding(s)
```

Rules:
- **R1** `.unwrap()` or `.expect(` can panic.
- **R2** arithmetic (` + `, ` - `, ` * `, `+=`, `-=`, `*=`) on a line without `checked_`, `saturating_`, `wrapping_`, `msg!(` or `println!(`.
- **R3** `AccountInfo<'info>` or `UncheckedAccount<'info>` without a `/// CHECK:` comment above it.

Lines that start with `//` are skipped.

## 2. Build and run

Needs Rust 1.85 or newer (the project uses edition 2024). Tested with rustc 1.99.1 on Windows. No external crates.

```
cargo build
cargo test
cargo run -- samples/bad.rs
```

Exit code: 0 = no findings, 1 = findings, 2 = usage or file error.

## 3. How I described it to the agents (main prompts)

I used one prompt for every model. It fixes the goal, the hard limits (standard library only, one file, at most 200 lines including tests, no warnings), the output format, the exit codes and the exact text of the three rules. I wrote the rules as precise text patterns on purpose: a vague request like "find security bugs" would give different programs, and I could not compare the models. I also asked for small testable functions and unit tests, and for a list of limitations at the end. The full prompt is in [PROMPT.md](PROMPT.md).

The prompt was prepared together with Claude (an AI assistant); I chose the idea and the rules. No other prompts were needed: both models produced working code on the first try (Model B fixed one of its own tests during its run).

## 4. Models I tried and how they differed

| Model | Where I ran it | Lines | Builds | Tests | Notes |
|---|---|---|---|---|---|
| Gemini 3.8 Flash (High) | Antigravity (agent) | 175 | yes, no warnings | 6 pass | One function per rule |
| Luna 6 | Codex desktop app | 116 | yes, no warnings | 3 pass | All rules in one function |

Both programs gave the same results on all my sample files and edge cases (empty file, Windows line endings, several rules on one line, comments). The difference is in structure.

Gemini put each rule in its own small function (`check_r1`, `check_r2`, `check_r3`) and wrote a positive and a negative test for each rule. That is closer to what I asked, but the file is longer. Luna 6 wrote shorter code, but all three rules sit in one long function, so they cannot be tested separately. Luna 6 also made a mistake in its own R1 test, which contained a real `.expect(` in the "negative" example, and fixed it after the first `cargo test`. The Gemini version had no problems.

Which one I kept and why: **Gemini 3.8 Flash (High)**. Its rules are separate and tested one by one, so it is easier to read rule by rule and to extend. The Luna 6 version is kept for comparison in [comparison/model-B-main.rs](comparison/model-B-main.rs). It is not part of the build.

The code was written by the agents. After that, Claude reviewed it: clean build, tests, clippy, and a comparison with a simple separate implementation of the same rules. I did not change the code. (The one thing Claude suggested, `cargo fmt`, is optional and I did not apply it.)

## 5. How it works (in my own words)

The program starts in `main`. It checks that there is exactly one argument (the file path); otherwise it prints a usage message and exits with code 2. Then it reads the whole file into a string with `fs::read_to_string`. If that fails, for example because the file does not exist, it prints the error and exits with code 2.

Then `content.lines()` splits the text into lines, and the function `check_lines` goes through them with their index. Lines that start with `//` after trimming are skipped, so comments are not checked. For every other line, three small functions are called. `check_r1` looks for `.unwrap()` or `.expect(`. `check_r2` looks for one of the arithmetic signs and also checks that none of the "safe" words are on the line. `check_r3` first checks if the line has `AccountInfo<'info>` or `UncheckedAccount<'info>`. If yes, it walks backwards over the lines above, as long as they start with `///` or `#[`. If it finds a line starting with `/// CHECK:` it says the account is fine. If it reaches any other line without finding one, it reports a finding.

Each finding is stored in a `Finding` struct (line number, rule, message, source line). One line can create several findings. At the end `main` prints each finding in the format `path:line: [RULE] message`, then the code line with two spaces, then the total count. It exits with code 1 if there is at least one finding, otherwise 0.

## 6. How I tested it

`cargo test` runs six unit tests: one positive and one negative case for each rule. The tests do not check line numbers, exit codes or lines that trigger several rules. I checked those by running the program on the files in `samples/` and by looking at the exit code. For example `samples/good.rs` returns 0, `samples/bad.rs` returns 1, and a missing file returns 2.

| File | Result |
|---|---|
| `samples/bad.rs` | 4 findings: line 9 R2, line 11 R1, line 12 R2, line 21 R3 |
| `samples/good.rs` | 0 findings, exit code 0 |
| `samples/limits.rs` | 1 finding (line 7, R2): a false positive. Line 9 is a real problem and is **not** reported (false negative) |

All sample code is made up. There are no keys or secrets in this repository.

## 7. Limitations

The linter reads text, not code, so it makes mistakes in both directions.

- **False positives.** `let limit = BASE + 5;` is flagged, even if both values are small constants and cannot overflow. Text inside a string, like `"a + b"`, is also flagged.
- **False negatives.** The rules look for ` + ` with spaces, so `vault.total+amount` is missed. A line that contains `checked_`, `saturating_` or `wrapping_` is trusted, because the author chose the behaviour on purpose. But `wrapping_` can hide an overflow: for example `balance.wrapping_sub(amount)` with a too large `amount` does not fail, it gives a huge number close to `u64::MAX`. The linter stays silent here, although a human auditor should look at it. Arithmetic split over several lines, or hidden in a macro, is also missed.
- **No context.** It does not know types, or whether a value was already validated earlier in the function, or whether an account is really checked somewhere else. The `/// CHECK:` rule only checks that a comment exists, not that the comment is true.

So it is not a replacement for a real audit. A serious tool would parse the code into a syntax tree (for example with the `syn` crate) and analyse it, not search for text.

## 8. What I would do next

Parse the file with `syn` so that comments, strings and macros are understood, and add a rule for missing signer or owner checks.

---
*Code written by AI models, see section 4. Samples are made up code; the repository contains no keys or secrets. This README was drafted with help from Claude and checked by me.*
