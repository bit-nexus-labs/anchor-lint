use std::{env, fs, process};

struct Finding {
    line: usize,
    rule: &'static str,
    message: &'static str,
    source: String,
}

fn check_lines(lines: &[&str]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        let mut add = |rule, message| findings.push(Finding {
            line: index + 1,
            rule,
            message,
            source: (*line).to_owned(),
        });
        if line.contains(".unwrap()") || line.contains(".expect(") {
            add("R1", "may panic");
        }
        let arithmetic = [" + ", " - ", " * ", "+=", "-=", "*="];
        let guarded = ["checked_", "saturating_", "wrapping_", "msg!(", "println!("];
        if arithmetic.iter().any(|token| line.contains(token))
            && !guarded.iter().any(|token| line.contains(token))
        {
            add("R2", "unchecked arithmetic");
        }
        if line.contains("AccountInfo<'info>") || line.contains("UncheckedAccount<'info>") {
            let mut previous = index;
            let mut has_check = false;
            while previous > 0 {
                let doc = lines[previous - 1].trim();
                if doc.starts_with("///") || doc.starts_with("#[") {
                    has_check |= doc.starts_with("/// CHECK:");
                    previous -= 1;
                } else {
                    break;
                }
            }
            if !has_check {
                add("R3", "account without CHECK comment");
            }
        }
    }
    findings
}

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "anchor-lint".to_owned());
    let Some(path) = args.next() else {
        eprintln!("Usage: {program} <file.rs>");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("Usage: {program} <file.rs>");
        process::exit(2);
    }
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("{path}: {error}");
            process::exit(2);
        }
    };
    let lines: Vec<&str> = contents.lines().collect();
    let findings = check_lines(&lines);
    for finding in &findings {
        println!("{}:{}: [{}] {}", path, finding.line, finding.rule, finding.message);
        println!("  {}", finding.source);
    }
    println!("{} finding(s)", findings.len());
    if !findings.is_empty() {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::check_lines;

    #[test]
    fn r1_finds_panic_calls_but_ignores_comment_lines() {
        assert_eq!(check_lines(&["let x = value.unwrap();"]).len(), 1);
        assert_eq!(check_lines(&["value.expect(\"x\");"]).len(), 1);
        assert!(check_lines(&["// value.unwrap()", "/// value.expect(\"x\")"]).is_empty());
    }

    #[test]
    fn r2_finds_unchecked_arithmetic_but_not_guarded_or_comment_lines() {
        assert_eq!(check_lines(&["let sum = a + b;"]).len(), 1);
        assert!(check_lines(&[
            "let sum = a.checked_add(b);",
            "msg!(\"a + b\");",
            "// a + b",
        ])
        .is_empty());
    }

    #[test]
    fn r3_requires_a_direct_preceding_check_comment() {
        assert_eq!(check_lines(&["pub account: AccountInfo<'info>,"]).len(), 1);
        assert!(check_lines(&[
            "/// CHECK: verified by the caller",
            "#[account(mut)]",
            "pub account: AccountInfo<'info>,",
        ])
        .is_empty());
    }
}

