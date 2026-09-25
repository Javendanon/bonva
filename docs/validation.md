# Validación ejecutada — 2026-09-15

Este documento registra una validación histórica. Sus informes de ejecución se
retiraron del repositorio; las pruebas actuales generan su propia evidencia.

## Suite propia

`python3 -m unittest discover -s tests -v`: **30 pruebas aprobadas**.
Incluye las dos pruebas previas de integridad y consulta de conocimiento.

Comprobaciones: límites y monotonía de normalización, Decimal, pesos inválidos,
NaN/infinito/booleanos, versiones y esquemas, métricas ausentes, gates duros,
regresiones, parser real, provenance, timeout, herramienta ausente, symlinks,
repetición de métricas, compilación y tests reales fallidos, suites vacías y
omitidas, alteración de fuente, salidas temporales y cambios del contrato de tests.

## Ejemplo controlado

La comparación de fixtures produjo baseline 6.683333, candidato
10, mejora 3.316667. Dos tests aprobados en cada proyecto. El baseline excede el
umbral de decisiones; el candidato pasa. No se afirma una mejora de runtime.

## Proyecto real: deejai/web

La ejecución histórica final en copia temporal produjo:

- 39 archivos Elixir de lib; 247 cláusulas analizadas.
- Compilación exitosa; 85 tests, 85 aprobados, ninguno omitido ni excluido.
- Máximo indicador de decisiones: 14; parámetros: 4; p95 de longitud: 29 líneas.
- Score del perfil: 6.533333. Gates fallidos: decision_max y target_score.
- `valid_bar?/1`, track.ex:161: indicador 14.
- `valid_region_bounds?/1`, track.ex:262: indicador 12.
- Cuatro dimensiones no medidas conservadas como null.

El gate es una política del evaluador: estos hechos no prueban errores funcionales
en esas funciones. El rechazo no implica que deejai falle sus pruebas.

El inventario SHA-256 del original coincidió antes y después de ejecutar, y volvió
a coincidir al revisar el informe. `git status --short` del monorepo permaneció
vacío. No se modificaron archivos del proyecto. Los tests ejecutaron su configuración
de PostgreSQL de pruebas existente; esta ejecución no mide escalabilidad concurrente.

Recalcular el resultado con `evaluate(raw_metrics, policy)` a partir del informe
produjo nuevamente 6.533333. El informe final pasó el validador de contratos.

## Problemas del adaptador resueltos

La copia debe conservar `.git` dentro de dependencias Git, aunque omita el Git
del proyecto. Mix usa esos metadatos para resolver heroicons. Los tests reales
también sobrescriben ejecutables simulados en tmp: ahora se registran como salidas
permitidas y no falsean el estado del compilador. Los cambios de entrada tienen
un gate independiente, probado con casos positivos y negativos.
