use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuestionCategory {
    NegativeScope,
    DiffAppetite,
    RuntimeQuirks,
    VerificationProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityReport {
    pub score: u32,
    pub vague_terms_found: Vec<String>,
    pub missing_negative_scope: bool,
    pub missing_diff_appetite: bool,
    pub missing_verification: bool,
    pub missing_runtime_context: bool,
    pub summary: String,
    pub requires_interview: bool,
}

const VAGUE_VERBS: &[&str] = &[
    "clean up",
    "cleanup",
    "fix this",
    "fix it",
    "refactor",
    "optimize",
    "improve",
    "make it better",
    "polish",
    "standardize",
    "modernize",
    "tidy",
    "handle",
    "sort out",
    "do something about",
];

const VAGUE_ADJECTIVES: &[&str] = &[
    "better",
    "faster",
    "cleaner",
    "proper",
    "robust",
    "elegant",
    "standard",
    "nice",
    "slick",
];

const NEGATIVE_SCOPE_KEYWORDS: &[&str] = &[
    "don't touch",
    "do not touch",
    "leave",
    "keep",
    "preserve",
    "without changing",
    "avoid touching",
    "only modify",
    "only touch",
    "restricted to",
];

const DIFF_APPETITE_KEYWORDS: &[&str] = &[
    "hotfix",
    "one-liner",
    "surgical",
    "patch",
    "small fix",
    "quick fix",
    "rewrite",
    "full refactor",
    "minimal",
    "tiny diff",
    "diff under",
    "diff",
    "lines",
    "keep diff",
];

const VERIFICATION_KEYWORDS: &[&str] = &[
    "test",
    "verify",
    "check with",
    "cargo test",
    "npm test",
    "pytest",
    "assert",
    "curl",
    "criterion",
    "evidence",
];

const RUNTIME_STATE_KEYWORDS: &[&str] = &[
    "port",
    "server",
    "process",
    "daemon",
    "env",
    "cache",
    "database",
    "docker",
    "redis",
    "socket",
];

pub fn analyze_prompt(prompt: &str) -> AmbiguityReport {
    let lower = prompt.to_lowercase();
    let mut score = 0u32;
    let mut vague_terms_found = Vec::new();

    for &verb in VAGUE_VERBS {
        if lower.contains(verb) {
            vague_terms_found.push(verb.to_string());
            score += 15;
        }
    }

    for &adj in VAGUE_ADJECTIVES {
        let pattern = format!(r"\b{}\b", adj);
        if let Ok(re) = Regex::new(&pattern) {
            if re.is_match(&lower) {
                vague_terms_found.push(adj.to_string());
                score += 10;
            }
        }
    }

    let has_negative_scope = NEGATIVE_SCOPE_KEYWORDS.iter().any(|&k| lower.contains(k));
    let missing_negative_scope = !has_negative_scope;
    if missing_negative_scope {
        score += 25;
    }

    let has_diff_appetite = DIFF_APPETITE_KEYWORDS.iter().any(|&k| lower.contains(k));
    let missing_diff_appetite = !has_diff_appetite;
    if missing_diff_appetite {
        score += 20;
    }

    let has_verification = VERIFICATION_KEYWORDS.iter().any(|&k| lower.contains(k));
    let missing_verification = !has_verification;
    if missing_verification {
        score += 25;
    }

    let has_runtime_context = RUNTIME_STATE_KEYWORDS.iter().any(|&k| lower.contains(k));
    let missing_runtime_context = !has_runtime_context;
    if missing_runtime_context {
        score += 15;
    }

    // Cap score at 100
    if score > 100 {
        score = 100;
    }

    // Brief prompt penalty: under 50 chars is heavily penalized
    if prompt.trim().len() < 50 && score < 70 {
        score = (score + 25).min(100);
    }

    let requires_interview = score > 20;

    let summary = if score <= 20 {
        "Low ambiguity. The task scope, boundaries, and verification criteria appear well-defined.".to_string()
    } else if score <= 50 {
        format!(
            "Moderate ambiguity (score: {}). Missing key boundary or verification constraints.",
            score
        )
    } else {
        format!(
            "High ambiguity (score: {}). Prompt lacks boundary constraints, diff size expectations, and verification proof.",
            score
        )
    };

    AmbiguityReport {
        score,
        vague_terms_found,
        missing_negative_scope,
        missing_diff_appetite,
        missing_verification,
        missing_runtime_context,
        summary,
        requires_interview,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vague_prompt_scores_high() {
        let report = analyze_prompt("clean up the auth flow");
        assert!(report.score >= 50);
        assert!(report.requires_interview);
        assert!(report.missing_negative_scope);
        assert!(report.missing_diff_appetite);
        assert!(report.missing_verification);
    }

    #[test]
    fn test_precise_prompt_scores_low() {
        let report = analyze_prompt(
            "In src/auth.rs, replace the 5000ms timeout with 30000ms. Do not touch user models or DB migrations. Keep diff under 5 lines and verify with cargo test test_auth_timeout.",
        );
        assert!(report.score <= 35);
        assert!(!report.missing_negative_scope);
        assert!(!report.missing_diff_appetite);
        assert!(!report.missing_verification);
    }
}
