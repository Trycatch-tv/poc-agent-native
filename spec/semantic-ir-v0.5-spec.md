# Agent-Native Semantic IR v0.5 Specification & Architecture Manual

**Version:** 0.5.0  
**Status:** Canonical Reference Standard  
**Maturity:** Production Ready for Deterministic Component Model  

---

## 1. Overview & Architectural Principles

**Semantic IR** is a strictly typed, declarative, JSON-encoded Intermediate Representation designed specifically for AI code generation agents and deterministic compilers. Rather than requiring models to generate low-level imperative code (such as Rust or C), Semantic IR abstracts domain logic into verifiable expression trees while preserving strict operational semantics.

Key principles:
1. **Total Determinism:** The same Semantic IR document compiled by `irtool` produces bit-for-bit identical WebAssembly Component Model binaries (`0x0d` magic).
2. **Contract-First Componentization:** Every component declares its WebAssembly Interface Types (WIT) contracts (`package`, `world`, `export_interface`, `import_interfaces`), compiling directly with Canonical ABI adaptors without glue code.
3. **No Hidden Defaults:** Undefined variables, invalid operator arity, type mismatches, and unknown interfaces produce structured, localized diagnostics with JSONPath expressions (`$.functions[0].body.left`).
4. **Isolated Host Boundary:** Components communicate with the runtime solely through explicit WIT imports and declared capabilities (`capabilities`), preventing unmediated memory access.

---

## 2. Supported Types

| Semantic IR Type | WebAssembly Representation | Description |
|---|---|---|
| `i32` | `i32` (32-bit signed integer) | Numeric calculations, cent amounts, quantities, status codes. |
| `bool` | `i32` (`0` = false, `1` = true) | Boolean truth values and conditional flags. |
| `Record(<Name>)` | Linear memory struct pointer (`i32`) | Flat structures with named typed fields (`status_code`, `total_cents`, etc.). |

---

## 3. Allowed AST Operators

Each expression node is a JSON object with an `"op"` field.

| Operator | Operands | Type Rule | Description |
|---|---|---|---|
| `const` | `"type"`, `"value"` | Matches `"type"` | Integer or boolean literal constant. |
| `param` | `"name"` | Matches parameter type | Reference to a function argument. |
| `not` | `"expr"` | `bool -> bool` | Logical negation. |
| `add` | `"left"`, `"right"` | `(i32, i32) -> i32` | Signed addition with overflow wrap. |
| `sub` | `"left"`, `"right"` | `(i32, i32) -> i32` | Signed subtraction. |
| `mul` | `"left"`, `"right"` | `(i32, i32) -> i32` | Signed multiplication. |
| `div_s`| `"left"`, `"right"` | `(i32, i32) -> i32` | Signed integer division. Traps on division by zero. |
| `eq` | `"left"`, `"right"` | `(T, T) -> bool` | Strict equality comparison. |
| `lt_s` | `"left"`, `"right"` | `(i32, i32) -> bool`| Signed less-than comparison. |
| `gte_s`| `"left"`, `"right"` | `(i32, i32) -> bool`| Signed greater-than-or-equal comparison. |
| `and` | `"left"`, `"right"` | `(bool, bool) -> bool`| Short-circuiting logical AND. |
| `or` | `"left"`, `"right"` | `(bool, bool) -> bool`| Short-circuiting logical OR. |
| `if` | `"condition"`, `"then"`, `"else"` | `(bool, T, T) -> T` | Branching expression. Both branches must have identical type. |
| `call` | `"function"`, `"args"` | Matches target signature | Invocation of internal function within the same module. |
| `call_import` | `"import"`, `"args"` | Matches imported WIT/host signature | Invocation of an external capability or linked component operation. |
| `record.construct` | `"type"`, `"fields"` | `Record(<type>)` | Allocates and initializes all fields of a declared record type. |
| `record.get` | `"expr"`, `"field"` | Matches field type | Reads a field from a record instance pointer. |

---

## 4. Diagnostic Code Reference

When `irtool verify` evaluates Semantic IR, it emits structured diagnostics if contracts are violated:

| Code | Severity | Description |
|---|---|---|
| `E_SCHEMA_INVALID` | Fatal | Structural JSON Schema violation or unknown root property. |
| `E_UNKNOWN_TYPE` | Fatal | Type referenced in function or record is not declared. |
| `E_UNDECLARED_PARAM` | Fatal | Parameter accessed in `param` op is not in function signature. |
| `E_TYPE_MISMATCH` | Fatal | Operand type does not match expected operator type. |
| `E_CALL_ARITY` | Fatal | Argument count in `call` or `call_import` does not match declaration. |
| `E_UNKNOWN_IMPORT` | Fatal | `call_import` references an import not declared in `"imports"`. |
| `E_UNKNOWN_EXPORT_INTERFACE` | Fatal | Function declares `wit_interface` not matching `wit_contract`. |
| `E_UNSUPPORTED_OP` | Fatal | Expression `"op"` is not in the allowed operator set. |
| `E_COMPONENT_ENCODE` | Fatal | Failure in WebAssembly Component Model metadata encoding or Canonical ABI synthesis. |
