use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub version: u32,
    pub task_id: String,
    pub created_at: String,
    pub goal: String,
    pub diff_appetite: String,
    pub allowed_write_scope: Vec<String>,
    pub forbidden_scope: Vec<String>,
    pub runtime_quirks: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub status: String, // "draft", "locked", "completed"
}

impl Contract {
    pub fn new(goal: &str, diff_appetite: &str) -> Self {
        let task_id = format!("cgrill-{}", Utc::now().format("%Y%m%d-%H%M%S"));
        Self {
            version: 1,
            task_id,
            created_at: Utc::now().to_rfc3339(),
            goal: goal.to_string(),
            diff_appetite: diff_appetite.to_string(),
            allowed_write_scope: Vec::new(),
            forbidden_scope: Vec::new(),
            runtime_quirks: Vec::new(),
            acceptance_criteria: Vec::new(),
            status: "draft".to_string(),
        }
    }

    pub fn lock(&mut self) {
        self.status = "locked".to_string();
    }

    pub fn complete(&mut self) {
        self.status = "completed".to_string();
    }

    pub fn is_write_allowed(&self, relative_path: &str) -> (bool, Option<String>) {
        if self.status != "locked" && self.status != "draft" {
            return (false, Some("Contract is not active or is completed.".to_string()));
        }

        // Check forbidden first
        for forbidden in &self.forbidden_scope {
            if glob_match(forbidden, relative_path) {
                return (
                    false,
                    Some(format!(
                        "Write denied. '{}' matches forbidden scope pattern '{}'.",
                        relative_path, forbidden
                    )),
                );
            }
        }

        // If allowed_write_scope is empty, permit writes within draft mode
        if self.allowed_write_scope.is_empty() {
            return (true, None);
        }

        // Check allowed
        let allowed = self
            .allowed_write_scope
            .iter()
            .any(|pattern| glob_match(pattern, relative_path));

        if allowed {
            (true, None)
        } else {
            (
                false,
                Some(format!(
                    "Write denied. '{}' is outside declared allowed scope: [{}]",
                    relative_path,
                    self.allowed_write_scope.join(", ")
                )),
            )
        }
    }

    pub fn generate_memory_pin(&self) -> String {
        format!(
            "[CGRILL CONTRACT ACTIVE]\nGOAL: {}\nDIFF APPETITE: {}\nALLOWED: {}\nFORBIDDEN: {}\nCRITERIA: {}\nSTATUS: {}",
            self.goal,
            self.diff_appetite,
            if self.allowed_write_scope.is_empty() { "all (unrestricted)".to_string() } else { self.allowed_write_scope.join(", ") },
            if self.forbidden_scope.is_empty() { "none".to_string() } else { self.forbidden_scope.join(", ") },
            if self.acceptance_criteria.is_empty() { "not specified".to_string() } else { self.acceptance_criteria.join("; ") },
            self.status.to_uppercase()
        )
    }

    pub fn save(&self, root: &Path) -> Result<PathBuf> {
        let dir = root.join(".cgrill");
        fs::create_dir_all(&dir)?;
        let file_path = dir.join("contract.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        fs::write(&file_path, json)?;
        Ok(file_path)
    }

    pub fn load(root: &Path) -> Result<Self> {
        let file_path = root.join(".cgrill").join("contract.json");
        if !file_path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("No active contract found at {}", file_path.display()),
            ));
        }
        let content = fs::read_to_string(&file_path)?;
        let contract: Contract = serde_json::from_str(&content)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        Ok(contract)
    }
}

fn glob_match(pattern: &str, path: &str) -> bool {
    if pattern == path || pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        return path.ends_with(suffix);
    }
    path.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_contract_lifecycle() {
        let mut contract = Contract::new("Fix auth bug", "surgical (<20 lines)");
        contract.allowed_write_scope.push("src/auth.rs".to_string());
        contract.forbidden_scope.push("migrations/*".to_string());
        contract.lock();

        let (allowed, err) = contract.is_write_allowed("src/auth.rs");
        assert!(allowed);
        assert!(err.is_none());

        let (denied_forbidden, err_f) = contract.is_write_allowed("migrations/001.sql");
        assert!(!denied_forbidden);
        assert!(err_f.is_some());

        let (denied_outside, err_o) = contract.is_write_allowed("src/user.rs");
        assert!(!denied_outside);
        assert!(err_o.is_some());

        let pin = contract.generate_memory_pin();
        assert!(pin.contains("[CGRILL CONTRACT ACTIVE]"));
        assert!(pin.contains("STATUS: LOCKED"));

        let dir = tempdir().unwrap();
        let path = contract.save(dir.path()).unwrap();
        assert!(path.exists());

        let loaded = Contract::load(dir.path()).unwrap();
        assert_eq!(loaded.task_id, contract.task_id);
    }
}
