#!/usr/bin/env bash
set -euo pipefail
: "${ORESLANG_SOURCE_DIR:?Set ORESLANG_SOURCE_DIR to the pinned oreslang-source.java checkout}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
expected="$(tr -d '[:space:]' < "$root/SOURCE_REF")"
if [[ ! "$expected" =~ ^[0-9a-f]{40}$ ]]; then
  echo 'SOURCE_REF must be an exact compiler commit SHA' >&2
  exit 65
fi
java_bin="${ORES_JAVA:-java}"
source_dir="$(cd "$ORESLANG_SOURCE_DIR" && pwd -P)"
if [[ "$(git -C "$source_dir" rev-parse --show-toplevel)" != "$source_dir" ]]; then
  echo 'Compiler directory must be its own Git checkout' >&2
  exit 65
fi
if [[ "$(git -C "$source_dir" rev-parse HEAD)" != "$expected" ]]; then
  echo 'Compiler revision differs from SOURCE_REF' >&2
  exit 65
fi
if [[ -n "$(git -C "$source_dir" status --porcelain --untracked-files=all)" ]]; then
  echo 'Compiler checkout must be clean' >&2
  exit 65
fi
if [[ ! -f "$source_dir/target/classpath.txt" ]]; then
  echo 'Build core with: mvn -q -DskipTests package dependency:build-classpath -Dmdep.outputFile=target/classpath.txt' >&2
  exit 69
fi
exec "$java_bin" --enable-native-access=ALL-UNNAMED -Dpolyglot.engine.WarnInterpreterOnly=false \
  -cp "$source_dir/target/classes:$(cat "$source_dir/target/classpath.txt")" dev.oreslang.launcher.OresMain "$@"
