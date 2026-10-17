use std::time::Instant;
use serde::{Deserialize, Serialize};
use crate::{SystemSpec, ImplementationAgent};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentExecutionMetrics {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub latency_ms: u128,
    pub attempts: usize,
    pub recovered_from_diagnostics: bool,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
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
                .unwrap_or(60),
        }
    }
}

/// Dynamic Implementation Agent with Real AI (LLM) Adapter, Deterministic Synthesizer Fallback,
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

    /// Prepares a structured system prompt that supplies the Semantic IR v0.5 schema,
    /// current component specification, and previous diagnostics (if any).
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
            .filter(|r| r.component == component_id)
            .collect();
        let relevant_ops: Vec<_> = spec
            .operations
            .iter()
            .filter(|o| o.component == component_id)
            .collect();

        let mut prompt = String::new();
        prompt.push_str("You are the Agent-Native Implementation Agent.\n");
        prompt.push_str("Generate a valid Semantic IR v0.5 JSON document conforming strictly to spec/semantic-ir-v0.5.schema.json.\n\n");
        prompt.push_str(&format!("Target Component ID: {}\n", component_id));

        if let Some(c) = comp {
            prompt.push_str(&format!("World: {}\n", c.world));
            prompt.push_str(&format!("WIT File: {}\n", c.wit_file));
            prompt.push_str(&format!("Export Interfaces: {:?}\n", c.export_interfaces));
            prompt.push_str(&format!("Import Interfaces: {:?}\n", c.import_interfaces));
            prompt.push_str(&format!("Capabilities: {:?}\n", c.capabilities));
        }

        prompt.push_str("\nActive Requirements for this component:\n");
        for r in relevant_reqs {
            prompt.push_str(&format!("- [{}] (v{}): {}\n", r.id, r.version, r.statement));
        }

        prompt.push_str("\nActive Operations for this component:\n");
        for o in relevant_ops {
            prompt.push_str(&format!(
                "- {} -> returns: {}, WIT: {}\n",
                o.name, o.returns, o.wit_interface
            ));
        }

        if !diagnostics.is_empty() {
            prompt.push_str("\n### CRITICAL: Previous attempt produced diagnostics. You must fix these errors:\n");
            for d in diagnostics {
                prompt.push_str(&format!("- [{}] {}: {}\n", d.code, d.path, d.message));
            }
        }

        prompt.push_str("\nOutput ONLY the valid JSON object for Semantic IR v0.5 without markdown formatting.\n");
        prompt
    }

    /// Performs real HTTP inference against an OpenAI-compatible /chat/completions endpoint.
    /// Extracts token usage and parses the resulting Semantic IR v0.5 JSON document.
    pub fn call_real_llm(&self, prompt: &str) -> Result<(serde_json::Value, AgentExecutionMetrics), String> {
        let api_key = self.config.api_key.as_deref().ok_or_else(|| {
            "No LLM API key configured (set AGENT_NATIVE_LLM_KEY or pass in LlmConfig)".to_string()
        })?;

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

        // Dispatch via curl using a temporary config file to prevent credentials from leaking in process tables or CLI args
        let start = Instant::now();
        let temp_cfg = std::env::temp_dir().join(format!("curlllm_{}_{}.cfg", std::process::id(), start.elapsed().as_nanos()));
        let cfg_content = format!("header = \"Authorization: Bearer {}\"\n", api_key);
        if let Err(e) = std::fs::write(&temp_cfg, cfg_content) {
            return Err(format!("Failed to write secure temporary curl config: {e}"));
        }

        let timeout_str = self.config.timeout_secs.to_string();
        let child = std::process::Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg(&self.config.endpoint)
            .arg("--max-time")
            .arg(&timeout_str)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("--config")
            .arg(&temp_cfg)
            .arg("-d")
            .arg(&payload_str)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let child = match child {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_file(&temp_cfg);
                return Err(format!("Failed to spawn curl for LLM inference: {e}"));
            }
        };

        let output = child.wait_with_output();
        let _ = std::fs::remove_file(&temp_cfg);

        let output = output.map_err(|e| format!("Failed waiting for curl output: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("curl request failed (exit code {:?}): {}", output.status.code(), stderr));
        }

        let resp_text = String::from_utf8_lossy(&output.stdout);
        let resp_json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| format!("Invalid JSON received from LLM endpoint: {e} (response was: {resp_text})"))?;

        if let Some(err_obj) = resp_json.get("error") {
            return Err(format!("LLM API returned error: {err_obj}"));
        }

        let prompt_tokens = resp_json
            .pointer("/usage/prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(prompt.len() as u64 / 4) as usize;

        let completion_tokens = resp_json
            .pointer("/usage/completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let content_str = resp_json
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "LLM response did not contain choices[0].message.content".to_string())?;

        // Extract JSON from Markdown code blocks if present
        let clean_json_str = if let Some(start_idx) = content_str.find("```json") {
            let remainder = &content_str[start_idx + 7..];
            if let Some(end_idx) = remainder.find("```") {
                &remainder[..end_idx]
            } else {
                remainder
            }
        } else if let Some(start_idx) = content_str.find("```") {
            let remainder = &content_str[start_idx + 3..];
            if let Some(end_idx) = remainder.find("```") {
                &remainder[..end_idx]
            } else {
                remainder
            }
        } else {
            content_str.trim()
        };

        let ir: serde_json::Value = serde_json::from_str(clean_json_str)
            .map_err(|e| format!("Failed parsing Semantic IR JSON from LLM: {e}"))?;

        let metrics = AgentExecutionMetrics {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
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

    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[irtool::Diagnostic],
    ) -> Result<serde_json::Value, String> {
        let start = Instant::now();
        let prompt = self.build_prompt(component_id, spec, diagnostics);

        // If an API key is configured or strict mode is set
        if let Some(ref api_key) = self.config.api_key {
            if !api_key.trim().is_empty() {
                match self.call_real_llm(&prompt) {
                    Ok((ir, metrics)) => {
                        if let Ok(mut lock) = self.last_metrics.lock() {
                            *lock = Some(metrics);
                        }
                        return Ok(ir);
                    }
                    Err(e) => {
                        // If no fallback is permitted (strict mode), fail immediately with explicit error
                        if self.fallback_agent.is_none() {
                            return Err(format!("[LlmImplementationAgent] Real LLM inference failed in strict mode: {e}"));
                        }
                        eprintln!("[LlmImplementationAgent] Real LLM inference error: {e}. Falling back to synthesizer.");
                    }
                }
            } else if self.fallback_agent.is_none() {
                return Err("[LlmImplementationAgent] AGENT_NATIVE_LLM_KEY is empty, but strict LLM mode is active".to_string());
            }
        } else if self.fallback_agent.is_none() {
            return Err("[LlmImplementationAgent] No API key configured (AGENT_NATIVE_LLM_KEY is required for --agent llm)".to_string());
        }

        // Delegate to deterministic fallback agent if configured
        let fallback = self.fallback_agent.as_ref().ok_or_else(|| {
            "[LlmImplementationAgent] LLM inference failed and no fallback agent is configured".to_string()
        })?;
        let ir = fallback.propose_ir(component_id, spec, diagnostics)?;

        let elapsed = start.elapsed().as_millis();
        let estimated_prompt_tokens = prompt.len() / 4;
        let estimated_completion_tokens = serde_json::to_string(&ir).map(|s| s.len() / 4).unwrap_or(0);

        let metrics = AgentExecutionMetrics {
            prompt_tokens: estimated_prompt_tokens,
            completion_tokens: estimated_completion_tokens,
            total_tokens: estimated_prompt_tokens + estimated_completion_tokens,
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
        if self.fail_first_attempt.swap(false, std::sync::atomic::Ordering::SeqCst) {
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
        let mut config = LlmConfig::default();
        config.api_key = None;
        let strict_agent = LlmImplementationAgent::strict(config);

        let spec = crate::parse_narrative(
            include_str!("../../../spec/narrative-005.md"),
            "narrative-005.md"
        ).unwrap();

        let result = strict_agent.propose_ir("shipping-calculator", &spec, &[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("AGENT_NATIVE_LLM_KEY is required"), "Error was: {err}");
    }
}
