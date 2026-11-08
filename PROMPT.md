# Prompt used for all models

The same text was given to every model, in a new chat or session each time.

```
Write a small Rust command-line program called `anchor-lint`.

Goal: a simple text-based linter for Anchor (Solana) program source files. It reads one `.rs` file and reports suspicious lines. It does NOT parse Rust; it only looks at the text, line by line.

Hard constraints:
- Rust standard library only. No external crates, no unsafe.
- One file, src/main.rs, at most 200 lines in total including unit tests.
- Must build with `cargo build` without warnings and run with `cargo run -- <file.rs>`.
- Idiomatic, readable code. Prefer the standard library over hand-written helpers.

Usage and output:
- `anchor-lint <file.rs>` prints one line per finding: `<path>:<line>: [<RULE>] <message>`, followed by the offending source line indented by two spaces. After all findings print a one-line summary: `N finding(s)`.
- Exit code 1 if there is at least one finding, 0 if there are none, 2 on usage or I/O errors (missing argument, file not found), with a clear error message on stderr.
- Line numbers start at 1.

Rules (apply to every line except lines whose trimmed text starts with `//`):
- R1 "may panic": the line contains `.unwrap()` or `.expect(`.
- R2 "unchecked arithmetic": the line contains one of ` + `, ` - `, ` * `, `+=`, `-=`, `*=` and does NOT contain `checked_`, `saturating_`, `wrapping_`, `msg!(` or `println!(`.
- R3 "account without CHECK comment": the line contains `AccountInfo<'info>` or `UncheckedAccount<'info>`, and none of the directly preceding lines that start (after trimming) with `///` or `#[` starts with `/// CHECK:`. Walk backwards over consecutive lines of those two kinds and stop at the first other line.
A line can trigger several rules.

Structure: put the rules in small testable functions (for example one function `check_lines(&[&str]) -> Vec<Finding>`), keep I/O in `main`. Include unit tests (`#[cfg(test)]`) that cover each rule with one positive and one negative case.

After the code, list in three bullets the main limitations of this text-matching approach.
```

## Run log

| # | Model (as shown in the app) | Where | Date | Lines | Builds without warnings | Tests | Notes |
|---|---|---|---|---|---|---|---|
| A | Gemini 3.8 Flash (High) | Antigravity | 8 Nov 2026 | 175 | yes | 6 pass | One function per rule. Kept. |
| B | Luna 6 | Codex desktop app | 8 Nov 2026 | 116 | yes | 3 pass | All rules in one function. One of its own tests needed a fix. Kept for comparison only. |

For model B the task was given inside a short instruction about the working folder; the prompt block above was the same.
