use crate::{ImplementationAgent, SystemSpec};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AgentExecutionMetrics {
    pub provider: String,
    pub model: String,
    pub tokens_reported: bool,
    pub prompt_tokens: Option<usize>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
    pub latency_ms: u128,
    pub attempts: usize,
    pub recovered_from_diagnostics: bool,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: f64,
    pub max_tokens: usize,
    pub timeout_secs: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: std::env::var("AGENT_NATIVE_LLM_PROVIDER")
                .unwrap_or_else(|_| "openai-http".to_string()),
            endpoint: std::env::var("AGENT_NATIVE_LLM_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string()),
            model: std::env::var("AGENT_NATIVE_LLM_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            api_key: std::env::var("AGENT_NATIVE_LLM_KEY").ok(),
            temperature: 0.0,
            max_tokens: 4096,
            timeout_secs: std::env::var("AGENT_NATIVE_LLM_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
        }
    }
}

impl LlmConfig {
    /// Resolves LLM configuration for CLI `--agent llm` execution.
    /// Uses explicit environment variables if present; if no HTTP API key is set,
    /// detects whether local authenticated `claude` CLI is available.
    pub fn for_cli_build() -> Self {
        let mut cfg = Self::default();
        if std::env::var("AGENT_NATIVE_LLM_PROVIDER").is_ok() {
            return cfg;
        }
        if cfg.api_key.as_ref().is_some_and(|k| !k.trim().is_empty()) {
            return cfg;
        }
        if Self::is_claude_cli_available() {
            cfg.provider = "claude-cli".to_string();
            if std::env::var("AGENT_NATIVE_LLM_MODEL").is_err() {
                cfg.model = "claude-sonnet-4-6".to_string();
            }
        }
        cfg
    }

    pub fn is_claude_cli_available() -> bool {
        Command::new("claude")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }
}

/// Dynamic Implementation Agent with Real AI (LLM) Adapter, Optional Fallback,
/// and Diagnostic Telemetry & Recovery.
pub struct LlmImplementationAgent {
    pub config: LlmConfig,
    pub fallback_agent: Option<Box<dyn ImplementationAgent + Send + Sync>>,
    pub record_telemetry: bool,
    pub last_metrics: std::sync::Mutex<Option<AgentExecutionMetrics>>,
}

impl LlmImplementationAgent {
    pub fn new(fallback_agent: Box<dyn ImplementationAgent + Send + Sync>) -> Self {
        Self {
            config: LlmConfig::default(),
            fallback_agent: Some(fallback_agent),
            record_telemetry: true,
            last_metrics: std::sync::Mutex::new(None),
        }
    }

    pub fn strict(config: LlmConfig) -> Self {
        Self {
            config,
            fallback_agent: None,
            record_telemetry: true,
            last_metrics: std::sync::Mutex::new(None),
        }
    }

    pub fn with_config(
        config: LlmConfig,
        fallback_agent: Box<dyn ImplementationAgent + Send + Sync>,
    ) -> Self {
        Self {
            config,
            fallback_agent: Some(fallback_agent),
            record_telemetry: true,
            last_metrics: std::sync::Mutex::new(None),
        }
    }

    /// Prepares a structured prompt that supplies the Semantic IR v0.5 grammar,
    /// current component specification, WIT contracts, and previous diagnostics (if any).
    pub fn build_prompt(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[irtool::Diagnostic],
    ) -> String {
        let comp = spec.components.iter().find(|c| c.id == component_id);
        let relevant_reqs: Vec<_> = spec
            .requirements
            .iter()
            .filter(|r| r.component == component_id && r.status == "active")
            .collect();
        let relevant_ops: Vec<_> = spec
            .operations
            .iter()
            .filter(|o| o.component == component_id)
            .collect();
        let other_ops: Vec<_> = spec
            .operations
            .iter()
            .filter(|o| o.component != component_id)
            .collect();

        let mut prompt = String::new();
        prompt.push_str("You are the Agent-Native Implementation Agent.\n");
        prompt.push_str("Generate a valid Semantic IR v0.5 JSON document conforming strictly to the Semantic IR v0.5 specification.\n\n");
        prompt.push_str(&format!("Target Component ID: {}\n", component_id));

        if let Some(c) = comp {
            prompt.push_str(&format!("World: {}\n", c.world));
            prompt.push_str(&format!("WIT File: {}\n", c.wit_file));
            prompt.push_str(&format!("Export Interfaces: {:?}\n", c.export_interfaces));
            prompt.push_str(&format!("Import Interfaces: {:?}\n", c.import_interfaces));
            prompt.push_str(&format!("Capabilities: {:?}\n", c.capabilities));
            prompt.push_str(&format!("Component Dependencies: {:?}\n", c.dependencies));

            if let Ok(wit_src) = std::fs::read_to_string(&c.wit_file) {
                prompt.push_str("\nWIT Contract Source:\n```wit\n");
                prompt.push_str(&wit_src);
                prompt.push_str("```\n");
            }
        }

        if !spec.entities.is_empty() {
            prompt.push_str("\nDomain Entities (Record Types):\n");
            for ent in &spec.entities {
                let fields_desc: Vec<String> = ent
                    .fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, f.field_type))
                    .collect();
                prompt.push_str(&format!(
                    "- {} {{ {} }}\n",
                    ent.name,
                    fields_desc.join(", ")
                ));
            }
        }

        prompt.push_str("\nActive Requirements for this component:\n");
        for r in &relevant_reqs {
            prompt.push_str(&format!("- [{}] (v{}): {}\n", r.id, r.version, r.statement));
        }

        prompt.push_str("\nExported Operations to implement for this component:\n");
        for o in &relevant_ops {
            let params_str: Vec<String> = o
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.param_type))
                .collect();
            prompt.push_str(&format!(
                "- {}({}) -> returns: {}, wit_interface: {}, rules: {:?}\n",
                o.name,
                params_str.join(", "),
                o.returns,
                o.wit_interface,
                o.rules_applied
            ));
        }

        if !other_ops.is_empty() {
            prompt.push_str("\nOperations in other components (available if imported via wit_contract.import_interfaces):\n");
            for o in &other_ops {
                let params_str: Vec<String> = o
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.param_type))
                    .collect();
                prompt.push_str(&format!(
                    "- component '{}': {}({}) -> returns: {}, wit_interface: {}\n",
                    o.component,
                    o.name,
                    params_str.join(", "),
                    o.returns,
                    o.wit_interface
                ));
            }
        }

        prompt.push_str(
            r#"
### Semantic IR v0.5 Grammar & Verifier Rules (STRICT):
1. Root JSON object fields:
   - "ir_version": "0.5"
   - "spec_version": 1
   - "wit_contract": {
       "package": "<namespace:pkg@version, e.g. from WIT>",
       "world": "<world-name>",
       "export_interface": "<short-interface-name, e.g. weather-service>",
       "import_interfaces": ["<short-imported-interface-name>", ...],
       "wit_file": "<exact WIT File path from above>"
     }
   - "capabilities": ["<cap1>", ...] (only if component has imports; every import's "capability" MUST be listed here)
   - "imports": [
       {
         "module": "<'host' for host-services, or package name like 'kaddo:airspace@0.1.0' for component imports>",
         "name": "<snake_case_import_func_name>",
         "capability": "<capability_string_present_in_capabilities>",
         "wit_interface": "<short-imported-interface-name, e.g. host-services or weather-service>",
         "params": [{ "name": "<param_name>", "type": "i32" }],
         "returns": "i32" // or "bool"
       }
     ]
   - "types": [
       {
         "name": "<RecordTypeName>",
         "fields": [{ "name": "<snake_case_field>", "type": "i32" }]
       }
     ] (include ONLY if an exported/helper function in this component returns a Record)
   - "functions": [
       {
         "name": "<snake_case_func_name>",
         "export": true, // false for internal helper functions
         "wit_interface": "<short-export-interface-name, required when export is true>",
         "params": [{ "name": "<snake_case_param>", "type": "i32" }],
         "returns": "<i32 | bool | RecordTypeName>",
         "requirements": ["<REQ-ID-1>", ...], // non-empty array of active requirement IDs for exported functions; [] for helpers
         "body": <Expression>
       }
     ]

2. Allowed Expression "op" values (NO OTHER OPERATORS EXIST):
   - Constant: { "op": "const", "type": "i32", "value": 42 } or { "op": "const", "type": "bool", "value": true }
   - Parameter: { "op": "param", "name": "<param_name>" }
   - Arithmetic (i32, i32 -> i32): "add", "sub", "mul", "div_s" with "left" and "right"
   - Comparison (i32, i32 -> bool): ONLY "eq", "lt_s", "gte_s" with "left" and "right"
     * WARNING: "gt_s", "lte_s", "neq" DO NOT EXIST!
     * For (A > B), write: { "op": "lt_s", "left": B, "right": A }
     * For (A <= B), write: { "op": "gte_s", "left": B, "right": A }
     * For (A != B), write: { "op": "not", "value": { "op": "eq", "left": A, "right": B } }
   - Boolean logic (bool -> bool):
     * "and", "or" with "left" and "right" (both must evaluate to bool)
     * "not" with "value" (must evaluate to bool)
   - Conditional: { "op": "if", "condition": <bool_expr>, "then": <expr>, "else": <expr> }
   - Helper call: { "op": "call", "function": "<helper_func_name>", "args": [<expr>, ...] }
   - Import call: { "op": "call_import", "import": "<import_func_name>", "args": [<expr>, ...] }
     * To invoke side-effect imports like emit_audit(event_id, payload) before returning a record or value, wrap it in an "if" condition:
       { "op": "if", "condition": { "op": "gte_s", "left": { "op": "call_import", "import": "emit_audit", "args": [...] }, "right": { "op": "const", "type": "i32", "value": 0 } }, "then": <result_expr>, "else": <result_expr> }
   - Record construction: { "op": "record.construct", "type": "<RecordTypeName>", "fields": { "<field_1>": <expr>, ... } }
   - Record field read: { "op": "record.get", "target": <record_expr>, "field": "<field_1>" }
"#,
        );

        if !diagnostics.is_empty() {
            prompt.push_str("\n### CRITICAL: Previous attempt produced verification/compilation diagnostics. Fix ALL of these errors:\n");
            for d in diagnostics {
                prompt.push_str(&format!("- [{}] {}: {}\n", d.code, d.path, d.message));
            }
        }

        prompt.push_str("\nOutput ONLY the raw, valid JSON object for Semantic IR v0.5 without markdown code fences or commentary.\n");
        prompt
    }

    fn extract_clean_json(content_str: &str) -> &str {
        let trimmed = content_str.trim();
        if let Some(start_idx) = trimmed.find("```json") {
            let remainder = &trimmed[start_idx + 7..];
            if let Some(end_idx) = remainder.find("```") {
                return remainder[..end_idx].trim();
            }
            return remainder.trim();
        }
        if let Some(start_idx) = trimmed.find("```") {
            let remainder = &trimmed[start_idx + 3..];
            if let Some(end_idx) = remainder.find("```") {
                return remainder[..end_idx].trim();
            }
            return remainder.trim();
        }
        if let (Some(first_brace), Some(last_brace)) = (trimmed.find('{'), trimmed.rfind('}')) {
            if first_brace < last_brace {
                return &trimmed[first_brace..=last_brace];
            }
        }
        trimmed
    }

    /// Performs real inference via local authenticated Claude Code CLI (`claude -p --output-format json`).
    /// Credentials are managed by the CLI keyring; the prompt is passed via stdin.
    fn call_claude_cli(
        &self,
        prompt: &str,
    ) -> Result<(serde_json::Value, AgentExecutionMetrics), String> {
        let start = Instant::now();
        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg("--output-format")
            .arg("json")
            .arg("--tools")
            .arg("")
            .arg("--no-session-persistence")
            .arg("--system-prompt")
            .arg("You are the Agent-Native Implementation Agent. Output ONLY a valid Semantic IR v0.5 JSON object with no markdown fences and no extra text.");

        if !self.config.model.is_empty() && self.config.model != "gpt-4o-mini" {
            cmd.arg("--model").arg(&self.config.model);
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn 'claude' CLI for LLM inference: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| format!("Failed writing prompt to claude stdin: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed waiting for claude CLI output: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "claude CLI inference failed (exit code {:?}): stderr={}, stdout={}",
                output.status.code(),
                stderr,
                stdout
            ));
        }

        let resp_text = String::from_utf8_lossy(&output.stdout);
        let envelope: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| format!("Invalid JSON envelope from claude CLI: {e}"))?;

        if envelope
            .get("is_error")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return Err(format!("claude CLI returned error response: {resp_text}"));
        }

        let result_str = envelope
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                "claude CLI JSON response did not contain string 'result' field".to_string()
            })?;

        let clean_json_str = Self::extract_clean_json(result_str);
        let ir: serde_json::Value = serde_json::from_str(clean_json_str).map_err(|e| {
            format!("Failed parsing Semantic IR JSON from claude CLI result: {e} (raw result: {result_str})")
        })?;

        // Extract exact token telemetry reported by provider (never estimate if missing)
        let usage_obj = envelope.get("usage");
        let (tokens_reported, prompt_tokens, completion_tokens, total_tokens) =
            if let Some(u) = usage_obj {
                let in_tok = u.get("input_tokens").and_then(|v| v.as_u64());
                let out_tok = u.get("output_tokens").and_then(|v| v.as_u64());
                let cache_create = u
                    .get("cache_creation_input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cache_read = u
                    .get("cache_read_input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                match (in_tok, out_tok) {
                    (Some(i), Some(o)) => {
                        let p = (i + cache_create + cache_read) as usize;
                        let c = o as usize;
                        (true, Some(p), Some(c), Some(p + c))
                    }
                    _ => (false, None, None, None),
                }
            } else {
                (false, None, None, None)
            };

        let detected_model = envelope
            .get("modelUsage")
            .and_then(|m| m.as_object())
            .and_then(|obj| obj.keys().next().cloned())
            .unwrap_or_else(|| self.config.model.clone());

        let metrics = AgentExecutionMetrics {
            provider: "claude-cli".to_string(),
            model: detected_model,
            tokens_reported,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            latency_ms: start.elapsed().as_millis(),
            attempts: 1,
            recovered_from_diagnostics: false,
        };

        Ok((ir, metrics))
    }

    /// Performs real HTTP inference against an OpenAI-compatible /chat/completions endpoint
    /// or dispatches to `claude-cli` when configured.
    /// Credentials for HTTP are passed exclusively via stdin (`curl --config -`), never written to disk or CLI args.
    pub fn call_real_llm(
        &self,
        prompt: &str,
    ) -> Result<(serde_json::Value, AgentExecutionMetrics), String> {
        if self.config.provider == "claude-cli" {
            return self.call_claude_cli(prompt);
        }

        let api_key = self.config.api_key.as_deref().ok_or_else(|| {
            "No LLM API key configured (set AGENT_NATIVE_LLM_KEY or pass in LlmConfig)".to_string()
        })?;
        if api_key.trim().is_empty() {
            return Err("AGENT_NATIVE_LLM_KEY is empty".to_string());
        }

        let request_body = serde_json::json!({
            "model": self.config.model,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
            "messages": [
                {
                    "role": "system",
                    "content": "You are the Agent-Native implementation agent. You generate valid Semantic IR v0.5 JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "response_format": { "type": "json_object" }
        });

        let payload_str = serde_json::to_string(&request_body).map_err(|e| e.to_string())?;

        // Dispatch via curl passing Authorization header strictly through stdin (--config -)
        // so credentials are NEVER written to temporary files on disk or exposed in CLI arguments.
        let start = Instant::now();
        let timeout_str = self.config.timeout_secs.to_string();
        let mut child = Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg(&self.config.endpoint)
            .arg("--max-time")
            .arg(&timeout_str)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("--config")
            .arg("-")
            .arg("-d")
            .arg(&payload_str)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn curl for LLM inference: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            let cfg_line = format!("header = \"Authorization: Bearer {}\"\n", api_key);
            stdin
                .write_all(cfg_line.as_bytes())
                .map_err(|e| format!("Failed to write Authorization header to curl stdin: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed waiting for curl output: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "curl request failed (exit code {:?}): {}",
                output.status.code(),
                stderr
            ));
        }

        let resp_text = String::from_utf8_lossy(&output.stdout);
        let resp_json: serde_json::Value = serde_json::from_str(&resp_text).map_err(|e| {
            format!("Invalid JSON received from LLM endpoint: {e} (response was: {resp_text})")
        })?;

        if let Some(err_obj) = resp_json.get("error") {
            return Err(format!("LLM API returned error: {err_obj}"));
        }

        // Strict token reporting: only record tokens when explicitly returned by the provider
        let p_tok = resp_json
            .pointer("/usage/prompt_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let c_tok = resp_json
            .pointer("/usage/completion_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let (tokens_reported, prompt_tokens, completion_tokens, total_tokens) = match (p_tok, c_tok)
        {
            (Some(p), Some(c)) => (true, Some(p), Some(c), Some(p + c)),
            _ => (false, None, None, None),
        };

        let content_str = resp_json
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "LLM response did not contain choices[0].message.content".to_string())?;

        let clean_json_str = Self::extract_clean_json(content_str);
        let ir: serde_json::Value = serde_json::from_str(clean_json_str)
            .map_err(|e| format!("Failed parsing Semantic IR JSON from LLM: {e}"))?;

        let metrics = AgentExecutionMetrics {
            provider: "openai-http".to_string(),
            model: self.config.model.clone(),
            tokens_reported,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            latency_ms: start.elapsed().as_millis(),
            attempts: 1,
            recovered_from_diagnostics: false,
        };

        Ok((ir, metrics))
    }
}

impl ImplementationAgent for LlmImplementationAgent {
    fn name(&self) -> &str {
        "LlmImplementationAgent"
    }

    fn last_execution_metrics(&self) -> Option<AgentExecutionMetrics> {
        self.last_metrics.lock().ok().and_then(|g| g.clone())
    }

    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[irtool::Diagnostic],
    ) -> Result<serde_json::Value, String> {
        let start = Instant::now();
        let prompt = self.build_prompt(component_id, spec, diagnostics);

        let can_attempt_real_llm = self.config.provider == "claude-cli"
            || self
                .config
                .api_key
                .as_ref()
                .is_some_and(|k| !k.trim().is_empty());

        if can_attempt_real_llm {
            match self.call_real_llm(&prompt) {
                Ok((ir, mut metrics)) => {
                    metrics.attempts = if diagnostics.is_empty() { 1 } else { 2 };
                    metrics.recovered_from_diagnostics = !diagnostics.is_empty();
                    if let Ok(mut lock) = self.last_metrics.lock() {
                        *lock = Some(metrics);
                    }
                    return Ok(ir);
                }
                Err(e) => {
                    if self.fallback_agent.is_none() {
                        return Err(format!(
                            "[LlmImplementationAgent] Real LLM inference failed in strict mode: {e}"
                        ));
                    }
                    eprintln!(
                        "[LlmImplementationAgent] Real LLM inference error: {e}. Falling back to synthesizer."
                    );
                }
            }
        } else if self.fallback_agent.is_none() {
            if self
                .config
                .api_key
                .as_ref()
                .is_some_and(|k| k.trim().is_empty())
            {
                return Err(
                    "[LlmImplementationAgent] AGENT_NATIVE_LLM_KEY is empty, but strict LLM mode is active"
                        .to_string(),
                );
            }
            return Err(
                "[LlmImplementationAgent] No API key configured (AGENT_NATIVE_LLM_KEY is required for --agent llm)"
                    .to_string(),
            );
        }

        // Delegate to deterministic fallback agent ONLY if explicitly configured (never in strict mode)
        let fallback = self.fallback_agent.as_ref().ok_or_else(|| {
            "[LlmImplementationAgent] LLM inference failed and no fallback agent is configured"
                .to_string()
        })?;
        let ir = fallback.propose_ir(component_id, spec, diagnostics)?;

        let elapsed = start.elapsed().as_millis();
        let metrics = AgentExecutionMetrics {
            provider: "deterministic-fallback".to_string(),
            model: fallback.name().to_string(),
            tokens_reported: false,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            latency_ms: elapsed,
            attempts: if diagnostics.is_empty() { 1 } else { 2 },
            recovered_from_diagnostics: !diagnostics.is_empty(),
        };

        if let Ok(mut lock) = self.last_metrics.lock() {
            *lock = Some(metrics);
        }

        Ok(ir)
    }
}

/// Mock agent for testing diagnostic recovery loops and prompt building
pub struct MockDiagnosticAgent {
    pub fail_first_attempt: std::sync::atomic::AtomicBool,
    pub fallback: Box<dyn ImplementationAgent + Send + Sync>,
}

impl MockDiagnosticAgent {
    pub fn new(fallback: Box<dyn ImplementationAgent + Send + Sync>) -> Self {
        Self {
            fail_first_attempt: std::sync::atomic::AtomicBool::new(true),
            fallback,
        }
    }
}

impl ImplementationAgent for MockDiagnosticAgent {
    fn name(&self) -> &str {
        "MockDiagnosticAgent"
    }

    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[irtool::Diagnostic],
    ) -> Result<serde_json::Value, String> {
        if self
            .fail_first_attempt
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            // Intentionally propose broken IR missing required field
            let mut val = self.fallback.propose_ir(component_id, spec, diagnostics)?;
            if let Some(obj) = val.as_object_mut() {
                obj.remove("wit_contract");
            }
            Ok(val)
        } else {
            // Self-repair: return valid IR
            self.fallback.propose_ir(component_id, spec, diagnostics)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_llm_mode_fails_explicitly_without_api_key() {
        let config = LlmConfig {
            provider: "openai-http".to_string(),
            api_key: None,
            ..Default::default()
        };
        let strict_agent = LlmImplementationAgent::strict(config);

        let spec = crate::parse_narrative(
            include_str!("../../../spec/narrative-005.md"),
            "narrative-005.md",
        )
        .unwrap();

        let result = strict_agent.propose_ir("shipping-calculator", &spec, &[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("AGENT_NATIVE_LLM_KEY is required"),
            "Error was: {err}"
        );
    }

    #[test]
    fn test_unreported_tokens_are_none_never_estimated() {
        let config = LlmConfig {
            provider: "openai-http".to_string(),
            api_key: None,
            ..Default::default()
        };
        let agent = LlmImplementationAgent::with_config(config, Box::new(crate::SynthesizerAgent));

        let spec = crate::parse_narrative(
            include_str!("../../../spec/narrative-005.md"),
            "narrative-005.md",
        )
        .unwrap();

        let _ir = agent.propose_ir("shipping-calculator", &spec, &[]).unwrap();
        let metrics = agent.last_execution_metrics().expect("metrics recorded");
        assert!(
            !metrics.tokens_reported,
            "Fallback must not claim tokens were reported"
        );
        assert!(
            metrics.prompt_tokens.is_none(),
            "Unreported prompt_tokens must be None, not estimated"
        );
        assert!(
            metrics.completion_tokens.is_none(),
            "Unreported completion_tokens must be None"
        );
        assert!(
            metrics.total_tokens.is_none(),
            "Unreported total_tokens must be None"
        );
    }
}
