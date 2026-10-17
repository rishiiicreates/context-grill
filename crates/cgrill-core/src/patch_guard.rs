use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchVerificationRequest {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub target_content: String,
    pub replacement_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchVerificationResult {
    pub valid: bool,
    pub drift_detected: bool,
    pub suggested_start_line: Option<usize>,
    pub suggested_end_line: Option<usize>,
    pub delimiter_balanced: bool,
    pub delimiter_mismatch_detail: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct DelimiterCounts {
    curly: i32,    // { }
    paren: i32,    // ( )
    bracket: i32,  // [ ]
}

fn count_delimiters(text: &str) -> DelimiterCounts {
    let mut counts = DelimiterCounts::default();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut prev_char = ' ';

    for ch in text.chars() {
        if ch == '\'' && prev_char != '\\' && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if ch == '"' && prev_char != '\\' && !in_single_quote {
            in_double_quote = !in_double_quote;
        } else if !in_single_quote && !in_double_quote {
            match ch {
                '{' => counts.curly += 1,
                '}' => counts.curly -= 1,
                '(' => counts.paren += 1,
                ')' => counts.paren -= 1,
                '[' => counts.bracket += 1,
                ']' => counts.bracket -= 1,
                _ => {}
            }
        }
        prev_char = ch;
    }
    counts
}

pub fn verify_patch(
    file_path: &Path,
    start_line: usize,
    end_line: usize,
    target_content: &str,
    replacement_content: &str,
) -> PatchVerificationResult {
    let target_content = if target_content.contains("\\n") && !target_content.contains('\n') {
        target_content.replace("\\n", "\n")
    } else {
        target_content.to_string()
    };
    let target_content = target_content.as_str();

    let replacement_content = if replacement_content.contains("\\n") && !replacement_content.contains('\n') {
        replacement_content.replace("\\n", "\n")
    } else {
        replacement_content.to_string()
    };
    let replacement_content = replacement_content.as_str();

    if !file_path.exists() {
        return PatchVerificationResult {
            valid: false,
            drift_detected: false,
            suggested_start_line: None,
            suggested_end_line: None,
            delimiter_balanced: false,
            delimiter_mismatch_detail: None,
            error_message: Some(format!("Target file does not exist: {}", file_path.display())),
        };
    }

    let file_text = match fs::read_to_string(file_path) {
        Ok(t) => t,
        Err(e) => {
            return PatchVerificationResult {
                valid: false,
                drift_detected: false,
                suggested_start_line: None,
                suggested_end_line: None,
                delimiter_balanced: false,
                delimiter_mismatch_detail: None,
                error_message: Some(format!("Failed to read target file: {}", e)),
            };
        }
    };

    let lines: Vec<&str> = file_text.lines().collect();
    let total_lines = lines.len();

    // 1. Check line bounds
    if start_line == 0 || end_line == 0 || start_line > end_line || end_line > total_lines {
        // Line out of bounds, check for drift
        let (drift, s_start, s_end) = find_target_in_lines(&lines, target_content);
        return PatchVerificationResult {
            valid: false,
            drift_detected: drift,
            suggested_start_line: s_start,
            suggested_end_line: s_end,
            delimiter_balanced: false,
            delimiter_mismatch_detail: None,
            error_message: Some(format!(
                "Lines [{}-{}] out of range for file with {} lines.",
                start_line, end_line, total_lines
            )),
        };
    }

    // 2. Check exact content in range
    let slice_lines = &lines[start_line - 1..end_line];
    let actual_slice = slice_lines.join("\n");

    let exact_match = actual_slice.trim() == target_content.trim()
        || actual_slice.contains(target_content.trim());

    if !exact_match {
        let (drift, s_start, s_end) = find_target_in_lines(&lines, target_content);
        return PatchVerificationResult {
            valid: false,
            drift_detected: drift,
            suggested_start_line: s_start,
            suggested_end_line: s_end,
            delimiter_balanced: false,
            delimiter_mismatch_detail: None,
            error_message: Some(format!(
                "Target content was not found at lines [{}-{}]. Line drift detected.",
                start_line, end_line
            )),
        };
    }

    // 3. Delimiter Balance Check
    let target_delims = count_delimiters(target_content);
    let repl_delims = count_delimiters(replacement_content);

    let mut delimiter_balanced = true;
    let mut delimiter_mismatch_detail = None;

    if target_delims != repl_delims {
        delimiter_balanced = false;
        let mut reasons = Vec::new();
        if target_delims.curly != repl_delims.curly {
            reasons.push(format!(
                "Curly braces mismatched: target net delta {}, replacement net delta {}",
                target_delims.curly, repl_delims.curly
            ));
        }
        if target_delims.paren != repl_delims.paren {
            reasons.push(format!(
                "Parentheses mismatched: target net delta {}, replacement net delta {}",
                target_delims.paren, repl_delims.paren
            ));
        }
        if target_delims.bracket != repl_delims.bracket {
            reasons.push(format!(
                "Square brackets mismatched: target net delta {}, replacement net delta {}",
                target_delims.bracket, repl_delims.bracket
            ));
        }
        delimiter_mismatch_detail = Some(reasons.join("; "));
    }

    PatchVerificationResult {
        valid: exact_match && delimiter_balanced,
        drift_detected: false,
        suggested_start_line: Some(start_line),
        suggested_end_line: Some(end_line),
        delimiter_balanced,
        delimiter_mismatch_detail,
        error_message: if delimiter_balanced {
            None
        } else {
            Some("Patch introduces unbalanced brackets or clipped delimiters.".to_string())
        },
    }
}

fn find_target_in_lines(
    lines: &[&str],
    target: &str,
) -> (bool, Option<usize>, Option<usize>) {
    let target_trimmed = target.trim();
    if target_trimmed.is_empty() {
        return (false, None, None);
    }

    let target_lines: Vec<&str> = target_trimmed.lines().collect();
    let target_len = target_lines.len();

    if target_len == 1 {
        let single = target_lines[0].trim();
        for (idx, line) in lines.iter().enumerate() {
            if line.trim().contains(single) {
                return (true, Some(idx + 1), Some(idx + 1));
            }
        }
    } else {
        for idx in 0..=lines.len().saturating_sub(target_len) {
            let slice = &lines[idx..idx + target_len];
            let matches = slice
                .iter()
                .zip(target_lines.iter())
                .all(|(a, b)| a.trim() == b.trim());
            if matches {
                return (true, Some(idx + 1), Some(idx + target_len));
            }
        }
    }

    (false, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_patch_passes() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "fn foo() {{\n    let x = 1;\n    let y = 2;\n}}").unwrap();

        let res = verify_patch(
            file.path(),
            2,
            3,
            "let x = 1;\n    let y = 2;",
            "let x = 10;\n    let y = 20;",
        );
        assert!(res.valid);
        assert!(res.delimiter_balanced);
    }

    #[test]
    fn test_unbalanced_braces_fails() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "fn foo() {{\n    let x = 1;\n}}").unwrap();

        let res = verify_patch(
            file.path(),
            2,
            2,
            "let x = 1;",
            "let x = 1; }", // introduces extra closing brace!
        );
        assert!(!res.valid);
        assert!(!res.delimiter_balanced);
    }

    #[test]
    fn test_line_drift_suggests_new_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "// line 1\n// line 2\nfn target_function() {{\n}}").unwrap();

        // Asking for lines 10-11 when it's at lines 3-4
        let res = verify_patch(file.path(), 1, 2, "fn target_function() {\n}", "fn target_function() {\n    // new\n}");
        assert!(!res.valid);
        assert!(res.drift_detected);
        assert_eq!(res.suggested_start_line, Some(3));
        assert_eq!(res.suggested_end_line, Some(4));
    }
}
