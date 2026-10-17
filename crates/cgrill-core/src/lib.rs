pub mod ambiguity;
pub mod contract;
pub mod env_scanner;
pub mod interview;
pub mod patch_guard;

pub use ambiguity::{analyze_prompt, AmbiguityReport, QuestionCategory};
pub use contract::Contract;
pub use env_scanner::{scan_environment, EnvHealth, GitSummary, PortInfo, StaleCacheWarning};
pub use interview::{generate_questions, InterviewQuestion};
pub use patch_guard::{verify_patch, PatchVerificationRequest, PatchVerificationResult};
