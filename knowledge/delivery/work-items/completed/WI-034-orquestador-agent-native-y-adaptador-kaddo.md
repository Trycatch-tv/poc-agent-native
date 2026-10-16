---
type: spike
id: WI-034
alias: KAN-POC5-WI-034
title: Orquestador Agent-Native y adaptador Kaddo
knowledge_level: K3
status: completed
implementation_status: completed
phase: completed
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
ready_at: '2026-10-16'
domains:
  - orchestrator
  - system-spec
  - deterministic-toolchain
code:
  - crates/orchestrator/**
  - Cargo.toml
---

# WI-034 (KAN-POC5-WI-034) — Orquestador Agent-Native y adaptador Kaddo

## Outcome

- **Actor:** Operador del sistema y agente de desarrollo.
- **Target behavior:**
  - Implementar el crate `crates/orchestrator` como un orquestador independiente de desarrollo.
  - El orquestador proporciona interfaces de línea de comando y APIs programáticas para:
    1. Interpretar una narrativa funcional (`narrative.md`) o una secuencia de Work Items (`wi-*.md`).
    2. Sintetizar y versionar la **System Specification** consolidada (`system-spec.json`) sin contradicciones.
    3. Coordinar la generación / modificación de Semantic IR para cada componente.
    4. Invocar a `irtool` para verificar y compilar a WebAssembly Components.
    5. Administrar el ciclo de corrección autónomo guiado por los diagnósticos estructurados del verificador.
    6. Ejecutar suites de pruebas y generar evidencia de trazabilidad integral.
  - Implementar un adaptador modular para Kaddo que consuma Work Items y artefactos de conocimiento de Kaddo sin atar el core del orquestador a Kaddo.
- **Observable completion:** `cargo run -p orchestrator` compila y ejecuta comandos `spec`, `build`, `test` y `diff` en modo standalone y con flag `--kaddo`.

## Acceptance Criteria

- [x] Crate `crates/orchestrator` añadido al workspace Cargo.
- [x] Comando de consolidación de especificación a partir de narrativa y de secuencia de Work Items.
- [x] Mecanismo de ciclo de corrección guiado por diagnósticos deterministas.
- [x] Adaptador opcional para Kaddo funcional sin acoplamiento obligatorio del compilador.
