---
type: spike
id: WI-041
alias: KAN-POC5-WI-041
title: 'Benchmark del ciclo de vida completo: Agent-Native vs Tradicional'
knowledge_level: K3
status: completed
implementation_status: completed
phase: completed
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
closed_at: '2026-10-16'
domains:
  - experiment-evidence
  - orchestrator
code:
  - BENCHMARK_REPORT_005.md
  - runs/benchmark-poc005/**
---

# WI-041 (KAN-POC5-WI-041) — Benchmark del ciclo de vida completo: Agent-Native vs Tradicional

## Outcome

- **Actor:** Investigador principal y agente en benchmark de POC 005.
- **Target behavior:**
  - Medir y comparar el ciclo de vida completo de desarrollo: desde la especificación narrativa hasta componentes ejecutables probados.
  - Comparar métricas entre:
    1. Flujo Agent-Native (Narrativa -> System Spec -> IR -> Componentes WASM).
    2. Flujo Tradicional asistido por LLM (Narrativa -> Código fuente Rust/Go -> Toolchains estándar -> Componentes WASM).
  - Medir latencia de ciclo completo (162.4 ms vs 14,240 ms -> 87.6x ventaja), volumen de tokens (93.5 % ahorro), compacidad de diagnósticos (96.4 % ahorro) y estabilidad determinista de artefactos.
  - Consolidar los resultados en `BENCHMARK_REPORT_005.md`.
- **Observable completion:** `BENCHMARK_REPORT_005.md` publicado con datos cuantitativos de ciclo de vida completo y métricas pareadas.

## Acceptance Criteria

- [x] Métricas de ciclo de vida de desarrollo completadas y tabuladas.
- [x] Comparación de compacidad diagnóstica y tasa de reparación.
- [x] Reporte `BENCHMARK_REPORT_005.md` documentado y versionado.
