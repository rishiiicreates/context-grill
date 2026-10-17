use cgrill_core::{
    analyze_prompt, generate_questions, scan_environment, verify_patch, Contract,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    #[serde(default)]
    pub jsonrpc: Option<String>,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

fn make_success_response(id: Option<Value>, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "result": result
    })
}

fn make_error_response(id: Option<Value>, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn parse_items(val: Option<&Value>) -> Vec<String> {
    match val {
        Some(Value::String(s)) => s
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_usize_arg(val: Option<&Value>) -> Option<usize> {
    match val {
        Some(Value::Number(n)) => n.as_u64().map(|v| v as usize),
        Some(Value::String(s)) => s.parse::<usize>().ok(),
        _ => None,
    }
}

pub fn get_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "cgrill_scan",
                "description": "Scan a prompt or task description for ambiguity, vague terms, missing boundaries, and generate Socratic questions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The prompt or task description to analyze"
                        }
                    },
                    "required": ["prompt"]
                }
            },
            {
                "name": "cgrill_env",
                "description": "Inspect local runtime environment including active listening ports, git status, and stale build caches.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "dir": {
                            "type": "string",
                            "description": "Directory to inspect (defaults to current working directory)"
                        }
                    }
                }
            },
            {
                "name": "cgrill_contract_init",
                "description": "Initialize a new Ground Truth Contract (.cgrill/contract.json).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "goal": {
                            "type": "string",
                            "description": "The primary goal or task statement"
                        },
                        "diff_appetite": {
                            "type": "string",
                            "description": "Diff appetite (e.g. 'surgical (<20 lines)', 'targeted (<100 lines)')"
                        },
                        "allowed": {
                            "type": "string",
                            "description": "Comma-separated allowed file paths or patterns"
                        },
                        "forbidden": {
                            "type": "string",
                            "description": "Comma-separated forbidden file paths or patterns"
                        },
                        "criteria": {
                            "type": "string",
                            "description": "Comma-separated acceptance criteria commands"
                        }
                    },
                    "required": ["goal"]
                }
            },
            {
                "name": "cgrill_contract_check",
                "description": "Check if modifying a specific file is permitted by the active contract.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to the file to check"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "cgrill_patch_guard",
                "description": "Verify a patch before touching disk, checking line drift and delimiter balancing.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": {
                            "type": "string",
                            "description": "Path to the target file"
                        },
                        "start": {
                            "type": "number",
                            "description": "Starting line number (1-indexed)"
                        },
                        "end": {
                            "type": "number",
                            "description": "Ending line number (1-indexed)"
                        },
                        "target": {
                            "type": "string",
                            "description": "Exact target content to replace"
                        },
                        "replacement": {
                            "type": "string",
                            "description": "Replacement content"
                        }
                    },
                    "required": ["file", "start", "end", "target", "replacement"]
                }
            }
        ]
    })
}

fn execute_cgrill_scan(args: &Map<String, Value>) -> Result<Value, String> {
    let prompt = args
        .get("prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter 'prompt'".to_string())?;

    let report = analyze_prompt(prompt);
    let questions = generate_questions(&report, prompt);

    Ok(json!({
        "ambiguity_score": report.score,
        "score": report.score,
        "summary": report.summary,
        "vague_terms_found": report.vague_terms_found,
        "requires_interview": report.requires_interview,
        "missing_negative_scope": report.missing_negative_scope,
        "missing_diff_appetite": report.missing_diff_appetite,
        "missing_verification": report.missing_verification,
        "missing_runtime_context": report.missing_runtime_context,
        "questions": questions,
        "report": report
    }))
}

fn execute_cgrill_env(args: &Map<String, Value>) -> Result<Value, String> {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let target_dir = match args
        .get("dir")
        .or_else(|| args.get("directory"))
        .and_then(|v| v.as_str())
    {
        Some(d) if !d.trim().is_empty() => PathBuf::from(d),
        _ => current_dir,
    };

    let health = scan_environment(&target_dir);

    Ok(json!({
        "scanned_dir": health.scanned_dir,
        "listening_ports": health.active_ports,
        "active_ports": health.active_ports,
        "git_status": health.git_status,
        "stale_caches": health.stale_caches,
        "env_file_present": health.env_file_present,
        "env_example_discrepancy": health.env_example_discrepancy,
        "warnings": health.warnings,
        "is_clean": health.is_clean
    }))
}

fn execute_cgrill_contract_init(args: &Map<String, Value>) -> Result<Value, String> {
    let goal = args
        .get("goal")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter 'goal'".to_string())?;

    let diff_appetite = args
        .get("diff_appetite")
        .or_else(|| args.get("diff"))
        .and_then(|v| v.as_str())
        .unwrap_or("surgical (<20 lines)");

    let mut contract = Contract::new(goal, diff_appetite);

    let allowed = parse_items(args.get("allowed"));
    if !allowed.is_empty() {
        contract.allowed_write_scope = allowed;
    }

    let forbidden = parse_items(args.get("forbidden"));
    if !forbidden.is_empty() {
        contract.forbidden_scope = forbidden;
    }

    let criteria = parse_items(args.get("criteria"));
    if !criteria.is_empty() {
        contract.acceptance_criteria = criteria;
    }

    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let target_dir = match args.get("dir").and_then(|v| v.as_str()) {
        Some(d) if !d.trim().is_empty() => PathBuf::from(d),
        _ => current_dir,
    };

    match contract.save(&target_dir) {
        Ok(path) => {
            let pin = contract.generate_memory_pin();
            Ok(json!({
                "success": true,
                "saved_path": path.display().to_string(),
                "task_id": contract.task_id,
                "status": contract.status,
                "memory_pin": pin,
                "contract": contract
            }))
        }
        Err(e) => Ok(json!({
            "success": false,
            "error": format!("Failed to save contract: {}", e)
        })),
    }
}

fn execute_cgrill_contract_check(args: &Map<String, Value>) -> Result<Value, String> {
    let path_str = args
        .get("path")
        .or_else(|| args.get("file"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter 'path'".to_string())?;

    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let target_dir = match args.get("dir").and_then(|v| v.as_str()) {
        Some(d) if !d.trim().is_empty() => PathBuf::from(d),
        _ => current_dir,
    };

    match Contract::load(&target_dir) {
        Ok(contract) => {
            let (permitted, reason) = contract.is_write_allowed(path_str);
            Ok(json!({
                "path": path_str,
                "permitted": permitted,
                "allowed": permitted,
                "reason": reason,
                "status": contract.status,
                "task_id": contract.task_id
            }))
        }
        Err(e) => Ok(json!({
            "path": path_str,
            "permitted": false,
            "allowed": false,
            "reason": Some(format!("No active contract found in {}: {}", target_dir.display(), e)),
            "error": Some(format!("No active contract found: {}", e))
        })),
    }
}

fn execute_cgrill_patch_guard(args: &Map<String, Value>) -> Result<Value, String> {
    let file_str = args
        .get("file")
        .or_else(|| args.get("file_path"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter 'file'".to_string())?;

    let start = parse_usize_arg(args.get("start")).ok_or_else(|| {
        "Missing or invalid required parameter 'start' (must be positive integer)".to_string()
    })?;

    let end = parse_usize_arg(args.get("end")).ok_or_else(|| {
        "Missing or invalid required parameter 'end' (must be positive integer)".to_string()
    })?;

    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter 'target'".to_string())?;

    let replacement = args
        .get("replacement")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter 'replacement'".to_string())?;

    let file_path = PathBuf::from(file_str);
    let res = verify_patch(&file_path, start, end, target, replacement);

    Ok(json!({
        "valid": res.valid,
        "drift_detected": res.drift_detected,
        "suggested_start_line": res.suggested_start_line,
        "suggested_end_line": res.suggested_end_line,
        "delimiter_balanced": res.delimiter_balanced,
        "delimiter_mismatch_detail": res.delimiter_mismatch_detail,
        "error_message": res.error_message
    }))
}

pub fn execute_tool(name: &str, args: &Map<String, Value>) -> Result<Value, String> {
    match name {
        "cgrill_scan" => execute_cgrill_scan(args),
        "cgrill_env" => execute_cgrill_env(args),
        "cgrill_contract_init" => execute_cgrill_contract_init(args),
        "cgrill_contract_check" => execute_cgrill_contract_check(args),
        "cgrill_patch_guard" => execute_cgrill_patch_guard(args),
        other => Err(format!("Unknown tool: {}", other)),
    }
}

pub fn handle_single_request(req: JsonRpcRequest) -> Option<Value> {
    match req.method.as_str() {
        "initialize" => {
            let res = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "cgrill",
                    "version": env!("CARGO_PKG_VERSION")
                }
            });
            Some(make_success_response(req.id, res))
        }
        "notifications/initialized" => None,
        "ping" => Some(make_success_response(req.id, json!({}))),
        "tools/list" => Some(make_success_response(req.id, get_tools_list())),
        "tools/call" => {
            let (tool_name, args) = match &req.params {
                Some(Value::Object(map)) => {
                    let name = map.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let arguments = if let Some(Value::Object(args_map)) = map.get("arguments") {
                        args_map.clone()
                    } else {
                        let mut clone = map.clone();
                        clone.remove("name");
                        clone
                    };
                    (name.to_string(), arguments)
                }
                _ => ("".to_string(), Map::new()),
            };

            if tool_name.is_empty() {
                return Some(make_error_response(
                    req.id,
                    -32602,
                    "Invalid params: missing tool 'name'",
                ));
            }

            match execute_tool(&tool_name, &args) {
                Ok(data) => {
                    let text = serde_json::to_string_pretty(&data)
                        .unwrap_or_else(|_| data.to_string());
                    let mut result_map = Map::new();
                    result_map.insert(
                        "content".to_string(),
                        json!([
                            {
                                "type": "text",
                                "text": text
                            }
                        ]),
                    );
                    let is_error = data.get("error").is_some()
                        && data.get("success") == Some(&Value::Bool(false));
                    result_map.insert("isError".to_string(), json!(is_error));

                    if let Value::Object(data_obj) = data {
                        for (k, v) in data_obj {
                            if k != "content" && k != "isError" {
                                result_map.insert(k, v);
                            }
                        }
                    }
                    Some(make_success_response(req.id, Value::Object(result_map)))
                }
                Err(err) => {
                    let res = json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Error: {}", err)
                            }
                        ],
                        "isError": true
                    });
                    Some(make_success_response(req.id, res))
                }
            }
        }
        "cgrill_scan"
        | "cgrill_env"
        | "cgrill_contract_init"
        | "cgrill_contract_check"
        | "cgrill_patch_guard" => {
            let args = match &req.params {
                Some(Value::Object(map)) => map.clone(),
                _ => Map::new(),
            };
            match execute_tool(&req.method, &args) {
                Ok(data) => Some(make_success_response(req.id, data)),
                Err(err) => Some(make_error_response(req.id, -32602, &err)),
            }
        }
        _ => {
            if req.method.starts_with("notifications/") || req.id.is_none() {
                None
            } else {
                Some(make_error_response(
                    req.id,
                    -32601,
                    &format!("Method not found: {}", req.method),
                ))
            }
        }
    }
}

pub fn handle_jsonrpc_message(val: Value) -> Option<Value> {
    match val {
        Value::Array(items) => {
            let mut responses = Vec::new();
            for item in items {
                if let Ok(req) = serde_json::from_value::<JsonRpcRequest>(item) {
                    if let Some(resp) = handle_single_request(req) {
                        responses.push(resp);
                    }
                } else {
                    responses.push(make_error_response(None, -32600, "Invalid Request"));
                }
            }
            if responses.is_empty() {
                None
            } else {
                Some(Value::Array(responses))
            }
        }
        Value::Object(_) => match serde_json::from_value::<JsonRpcRequest>(val) {
            Ok(req) => handle_single_request(req),
            Err(_) => Some(make_error_response(None, -32600, "Invalid Request")),
        },
        _ => Some(make_error_response(None, -32600, "Invalid Request")),
    }
}

pub fn run_mcp_loop<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    let mut buffer = String::new();

    while reader.read_line(&mut buffer)? > 0 {
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            buffer.clear();
            continue;
        }

        match serde_json::from_str::<Value>(trimmed) {
            Ok(json_val) => {
                if let Some(resp) = handle_jsonrpc_message(json_val) {
                    serde_json::to_writer(&mut writer, &resp)?;
                    writer.write_all(b"\n")?;
                    writer.flush()?;
                }
                buffer.clear();
            }
            Err(_) => {
                if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                    let err_resp =
                        make_error_response(None, -32700, "Parse error: invalid JSON");
                    serde_json::to_writer(&mut writer, &err_resp)?;
                    writer.write_all(b"\n")?;
                    writer.flush()?;
                    buffer.clear();
                } else if buffer.len() > 10 * 1024 * 1024 {
                    let err_resp =
                        make_error_response(None, -32700, "Parse error: message too large");
                    serde_json::to_writer(&mut writer, &err_resp)?;
                    writer.write_all(b"\n")?;
                    writer.flush()?;
                    buffer.clear();
                }
            }
        }
    }

    let trimmed = buffer.trim();
    if !trimmed.is_empty() {
        if let Ok(json_val) = serde_json::from_str::<Value>(trimmed) {
            if let Some(resp) = handle_jsonrpc_message(json_val) {
                serde_json::to_writer(&mut writer, &resp)?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
        } else {
            let err_resp = make_error_response(None, -32700, "Parse error");
            serde_json::to_writer(&mut writer, &err_resp)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }

    Ok(())
}

pub fn run_mcp_server() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_mcp_loop(stdin.lock(), stdout.lock())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use tempfile::tempdir;

    #[test]
    fn test_tools_list() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });
        let resp = handle_jsonrpc_message(req).expect("Expected response");
        assert_eq!(resp["id"], 1);
        let tools = resp["result"]["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 5);

        let names: Vec<&str> = tools
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"cgrill_scan"));
        assert!(names.contains(&"cgrill_env"));
        assert!(names.contains(&"cgrill_contract_init"));
        assert!(names.contains(&"cgrill_contract_check"));
        assert!(names.contains(&"cgrill_patch_guard"));
    }

    #[test]
    fn test_initialize() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": "init-1",
            "method": "initialize",
            "params": {}
        });
        let resp = handle_jsonrpc_message(req).expect("Expected response");
        assert_eq!(resp["id"], "init-1");
        assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(resp["result"]["serverInfo"]["name"], "cgrill");
    }

    #[test]
    fn test_cgrill_scan_tool_call() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "cgrill_scan",
                "arguments": {
                    "prompt": "fix the broken auth logic"
                }
            }
        });
        let resp = handle_jsonrpc_message(req).expect("Expected response");
        assert_eq!(resp["id"], 2);
        let res = &resp["result"];
        assert!(res["content"].is_array());
        assert_eq!(res["isError"], false);
        assert!(res["ambiguity_score"].as_u64().unwrap() > 20);
        assert!(!res["summary"].as_str().unwrap().is_empty());
        let questions = res["questions"].as_array().unwrap();
        assert!(!questions.is_empty());
    }

    #[test]
    fn test_cgrill_scan_direct_method() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "cgrill_scan",
            "params": {
                "prompt": "refactor database schema"
            }
        });
        let resp = handle_jsonrpc_message(req).expect("Expected response");
        assert_eq!(resp["id"], 3);
        let res = &resp["result"];
        assert!(res["ambiguity_score"].as_u64().unwrap() > 20);
        assert!(res["questions"].is_array());
    }

    #[test]
    fn test_cgrill_env_tool_call() {
        let dir = tempdir().unwrap();
        let req = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "cgrill_env",
                "arguments": {
                    "dir": dir.path().to_str().unwrap()
                }
            }
        });
        let resp = handle_jsonrpc_message(req).expect("Expected response");
        assert_eq!(resp["id"], 4);
        let res = &resp["result"];
        assert!(res["listening_ports"].is_array());
        assert!(res["git_status"].is_object());
        assert!(res["stale_caches"].is_array());
    }

    #[test]
    fn test_cgrill_contract_init_and_check() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();

        // 1. Init contract
        let init_req = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "cgrill_contract_init",
                "arguments": {
                    "goal": "Add MCP server mode",
                    "diff_appetite": "surgical (<20 lines)",
                    "allowed": "src/mcp.rs, src/main.rs",
                    "forbidden": "crates/cgrill-core/src/contract.rs",
                    "criteria": "cargo test",
                    "dir": dir_str
                }
            }
        });
        let init_resp = handle_jsonrpc_message(init_req).expect("Expected response");
        assert_eq!(init_resp["id"], 5);
        assert_eq!(init_resp["result"]["success"], true);
        assert!(dir.path().join(".cgrill/contract.json").exists());

        // 2. Check permitted write
        let check_req_allowed = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "cgrill_contract_check",
                "arguments": {
                    "path": "src/mcp.rs",
                    "dir": dir_str
                }
            }
        });
        let check_resp = handle_jsonrpc_message(check_req_allowed).expect("Expected response");
        assert_eq!(check_resp["result"]["permitted"], true);

        // 3. Check forbidden write
        let check_req_forbidden = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "cgrill_contract_check",
                "arguments": {
                    "path": "crates/cgrill-core/src/contract.rs",
                    "dir": dir_str
                }
            }
        });
        let check_resp_f = handle_jsonrpc_message(check_req_forbidden).expect("Expected response");
        assert_eq!(check_resp_f["result"]["permitted"], false);

        // 4. Check out of scope write
        let check_req_out = json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": {
                "name": "cgrill_contract_check",
                "arguments": {
                    "path": "random/file.rs",
                    "dir": dir_str
                }
            }
        });
        let check_resp_o = handle_jsonrpc_message(check_req_out).expect("Expected response");
        assert_eq!(check_resp_o["result"]["permitted"], false);
    }

    #[test]
    fn test_cgrill_patch_guard() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let initial_content = "fn main() {\n    let a = 1;\n    println!(\"{}\", a);\n}\n";
        fs::write(&file_path, initial_content).unwrap();

        // Valid patch
        let guard_req = json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "tools/call",
            "params": {
                "name": "cgrill_patch_guard",
                "arguments": {
                    "file": file_path.to_str().unwrap(),
                    "start": 2,
                    "end": 2,
                    "target": "    let a = 1;",
                    "replacement": "    let a = 42;"
                }
            }
        });
        let resp = handle_jsonrpc_message(guard_req).expect("Expected response");
        assert_eq!(resp["result"]["valid"], true);
        assert_eq!(resp["result"]["drift_detected"], false);
        assert_eq!(resp["result"]["delimiter_balanced"], true);

        // Unbalanced braces in replacement
        let unbalanced_req = json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/call",
            "params": {
                "name": "cgrill_patch_guard",
                "arguments": {
                    "file": file_path.to_str().unwrap(),
                    "start": 2,
                    "end": 2,
                    "target": "    let a = 1;",
                    "replacement": "    let a = { 42;"
                }
            }
        });
        let resp_unbalanced = handle_jsonrpc_message(unbalanced_req).expect("Expected response");
        assert_eq!(resp_unbalanced["result"]["valid"], false);
        assert_eq!(resp_unbalanced["result"]["delimiter_balanced"], false);
    }

    #[test]
    fn test_run_mcp_loop_stdio() {
        let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n";
        let mut output = Vec::new();
        run_mcp_loop(Cursor::new(input), &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let resp: Value = serde_json::from_str(output_str.trim()).unwrap();
        assert_eq!(resp["id"], 1);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_notification_produces_no_output() {
        let input = "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n";
        let mut output = Vec::new();
        run_mcp_loop(Cursor::new(input), &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_unknown_method_returns_error() {
        let input = "{\"jsonrpc\":\"2.0\",\"id\":99,\"method\":\"unknown_method\"}\n";
        let mut output = Vec::new();
        run_mcp_loop(Cursor::new(input), &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let resp: Value = serde_json::from_str(output_str.trim()).unwrap();
        assert_eq!(resp["id"], 99);
        assert_eq!(resp["error"]["code"], -32601);
    }
}
