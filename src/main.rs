use std::env;
use std::fs;
use std::process;

const ARITH_OPS: &[&str] = &[" + ", " - ", " * ", "+=", "-=", "*="];
const SAFE_PATTERNS: &[&str] = &[
    "checked_",
    "saturating_",
    "wrapping_",
    "msg!(",
    "println!(",
];

#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    pub line_no: usize,
    pub rule: &'static str,
    pub message: &'static str,
    pub code: String,
}

fn check_r1(line: &str) -> bool {
    line.contains(".unwrap()") || line.contains(".expect(")
}

fn check_r2(line: &str) -> bool {
    ARITH_OPS.iter().any(|op| line.contains(op))
        && !SAFE_PATTERNS.iter().any(|p| line.contains(p))
}

fn check_r3(lines: &[&str], idx: usize) -> bool {
    let line = lines[idx];
    if !line.contains("AccountInfo<'info>") && !line.contains("UncheckedAccount<'info>") {
        return false;
    }
    for &prev in lines[..idx].iter().rev() {
        let trimmed = prev.trim();
        if trimmed.starts_with("///") || trimmed.starts_with("#[") {
            if trimmed.starts_with("/// CHECK:") {
                return false;
            }
        } else {
            break;
        }
    }
    true
}

pub fn check_lines(lines: &[&str]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (idx, &line) in lines.iter().enumerate() {
        if line.trim().starts_with("//") {
            continue;
        }
        let line_no = idx + 1;
        if check_r1(line) {
            findings.push(Finding {
                line_no,
                rule: "R1",
                message: "may panic",
                code: line.trim().to_string(),
            });
        }
        if check_r2(line) {
            findings.push(Finding {
                line_no,
                rule: "R2",
                message: "unchecked arithmetic",
                code: line.trim().to_string(),
            });
        }
        if check_r3(lines, idx) {
            findings.push(Finding {
                line_no,
                rule: "R3",
                message: "account without CHECK comment",
                code: line.trim().to_string(),
            });
        }
    }
    findings
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: anchor-lint <file.rs>");
        process::exit(2);
    }

    let path = &args[1];
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error reading '{}': {}", path, err);
            process::exit(2);
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let findings = check_lines(&lines);

    for f in &findings {
        println!("{}:{}: [{}] {}", path, f.line_no, f.rule, f.message);
        println!("  {}", f.code);
    }
    println!("{} finding(s)", findings.len());

    if !findings.is_empty() {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r1_positive() {
        let lines = ["let x = val.unwrap();", "let y = opt.expect(\"err\");"];
        let findings = check_lines(&lines);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].rule, "R1");
        assert_eq!(findings[1].rule, "R1");
    }

    #[test]
    fn test_r1_negative() {
        let lines = ["let x = val.ok_or(Error)?;", "// val.unwrap()"];
        let findings = check_lines(&lines);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_r2_positive() {
        let lines = ["a = b + c;", "total += fee;"];
        let findings = check_lines(&lines);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].rule, "R2");
        assert_eq!(findings[1].rule, "R2");
    }

    #[test]
    fn test_r2_negative() {
        let lines = [
            "a = b.checked_add(c);",
            "msg!(\"total: {}\", a + b);",
            "println!(\"val: {}\", x - y);",
        ];
        let findings = check_lines(&lines);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_r3_positive() {
        let lines = [
            "#[account(mut)]",
            "pub auth: AccountInfo<'info>,",
        ];
        let findings = check_lines(&lines);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "R3");
    }

    #[test]
    fn test_r3_negative() {
        let lines = [
            "/// CHECK: validated in handler",
            "#[account(mut)]",
            "pub auth: AccountInfo<'info>,",
        ];
        let findings = check_lines(&lines);
        assert!(findings.is_empty());
    }
}
