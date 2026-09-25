#!/usr/bin/env bash
set -euo pipefail
skill_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -n "${AGENT_QUALITY_ROOT:-}" ]]; then
  engine_root="$AGENT_QUALITY_ROOT"
elif [[ -f "$skill_dir/engine-root.txt" ]]; then
  IFS= read -r engine_root < "$skill_dir/engine-root.txt"
else
  engine_root="$(cd -- "$skill_dir/../.." && pwd)"
fi
if [[ ! -f "$engine_root/Cargo.toml" || ! -f "$engine_root/src/list_analysis.rs" ]]; then
  printf '%s\n' 'No se encontró el motor. Define AGENT_QUALITY_ROOT con la ruta del repositorio refactoring-agent.' >&2
  exit 2
fi
command -v cargo >/dev/null || { printf '%s\n' 'Se requiere Cargo/Rust en PATH.' >&2; exit 2; }
# Pin the build output and profile: inherited Cargo target settings cannot select a stale binary.
cargo build --release --locked --manifest-path "$engine_root/Cargo.toml" --target-dir "$engine_root/target" >&2
exec "$engine_root/target/release/agent-quality" "$@"
