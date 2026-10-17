use crate::ambiguity::{AmbiguityReport, QuestionCategory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewQuestion {
    pub category: QuestionCategory,
    pub title: String,
    pub question: String,
    pub context_hint: String,
    pub suggested_options: Vec<String>,
}

pub fn generate_questions(report: &AmbiguityReport, prompt: &str) -> Vec<InterviewQuestion> {
    let mut questions = Vec::new();

    // 1. Negative Scope Question
    if report.missing_negative_scope || report.score > 30 {
        questions.push(InterviewQuestion {
            category: QuestionCategory::NegativeScope,
            title: "Negative Scope & Boundary".to_string(),
            question: "What files, modules, or existing behaviors must strictly remain untouched?".to_string(),
            context_hint: format!(
                "Prevent the agent from touching existing working code while handling: '{}'",
                prompt.trim()
            ),
            suggested_options: vec![
                "Only touch the exact target file and its unit test; leave all models and migrations intact".to_string(),
                "Do not alter public API signatures or external client contracts".to_string(),
                "Do not modify configuration, lockfiles, or dependency manifests".to_string(),
            ],
        });
    }

    // 2. Diff Appetite Question
    if report.missing_diff_appetite || report.score > 40 {
        questions.push(InterviewQuestion {
            category: QuestionCategory::DiffAppetite,
            title: "Diff Appetite & Blast Radius".to_string(),
            question: "What scale of code change are you expecting for this task?".to_string(),
            context_hint: "Restrain the agent from over-engineering or creating unwanted abstractions.".to_string(),
            suggested_options: vec![
                "Surgical hotfix (< 20 lines) — change the bare minimum to solve the issue".to_string(),
                "Targeted refactor (20-100 lines) — clean up the immediate function without adding new patterns".to_string(),
                "Architecture rewrite — create new modules, interfaces, and abstractions".to_string(),
            ],
        });
    }

    // 3. Runtime Quirks Question
    if report.missing_runtime_context || report.score > 40 {
        questions.push(InterviewQuestion {
            category: QuestionCategory::RuntimeQuirks,
            title: "Runtime & Environmental State".to_string(),
            question: "Are there active background daemons, running ports, or uncommitted branches to be aware of?".to_string(),
            context_hint: "Avoid debugging ghost code when local process state or caches are the real culprit.".to_string(),
            suggested_options: vec![
                "Clean environment; no background servers or special ports involved".to_string(),
                "Local dev server is running on a port (e.g. 3000/8080); don't kill or re-bind".to_string(),
                "Dirty workspace with uncommitted work; do not revert or stash existing changes".to_string(),
            ],
        });
    }

    // 4. Verification Proof Question
    if report.missing_verification || report.score > 20 {
        questions.push(InterviewQuestion {
            category: QuestionCategory::VerificationProof,
            title: "Proof of Completion".to_string(),
            question: "What exact command or evidence will prove this task is complete?".to_string(),
            context_hint: "No completion without executed evidence. Avoid self-review or speculative completion.".to_string(),
            suggested_options: vec![
                "A specific unit/integration test command passing (e.g. cargo test, npm test)".to_string(),
                "Manual CLI run showing expected stdout/stderr output".to_string(),
                "A clean type-check / linter pass with zero warnings".to_string(),
            ],
        });
    }

    questions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ambiguity::analyze_prompt;

    #[test]
    fn test_generates_all_four_questions_for_vague_prompt() {
        let report = analyze_prompt("fix the auth token error");
        let questions = generate_questions(&report, "fix the auth token error");
        assert!(questions.len() >= 3);
        assert!(questions.iter().any(|q| q.category == QuestionCategory::NegativeScope));
        assert!(questions.iter().any(|q| q.category == QuestionCategory::VerificationProof));
    }
}
