# Validación del núcleo Rust — 2026-09-15

Este documento registra una validación histórica. Los informes mencionados fueron
retirados del repositorio y la prueba que los leía se eliminó. La suite actual
conserva los 64 casos de referencia y las integraciones reproducibles sobre fixtures.

## Verificaciones completadas

- `cargo test --locked`: 13 pruebas de núcleo aprobadas; 3 integraciones excluidas por defecto.
- `cargo test --locked --test elixir_integration -- --ignored --test-threads=1`: las 3 integraciones aprobadas.
- `cargo clippy --locked --all-targets -- -D warnings`: sin advertencias.
- `cargo fmt --check`: aprobado.
- `cargo build --release --locked`: binario optimizado construido.

Una de las pruebas compara 64 casos fijos calculados por el motor Python previo.
Otra prueba, retirada junto con los informes, comparaba los resultados de las dos fixtures y de deejai, incluyendo diagnóstico
y comparación baseline/candidato. Las pruebas normales no ejecutan Python ni Elixir.

También se prueban pesos inválidos, datos ausentes, capacidades faltantes,
normalización monótona, bloqueo por gates, perfil ajeno a Elixir, timeout y
herramienta ausente. La caché cubre sincronización de cambios y eliminaciones,
bloqueo exclusivo, invalidación de runtime y mix.lock, y recuperación tras un run
que no dejó marcador de finalización.

Las integraciones prueban equivalencia entre ejecución inicial y reutilizada,
compilación de cambios de fuente, tests fallidos y vacíos, compilación fallida,
modificación de entradas por tests y conservación de los originales.

## deejai: ejecución real

Se compararon tres ejecuciones: el primer uso del espacio administrado, su
reutilización y la referencia Python anterior. Los informes eran salidas locales
y ya no se distribuyen.

En los tres coinciden `raw_metrics`, `quality_vector`, `score`, `gates`,
`diagnosis` y `source`. Compilación exitosa; 85 pruebas aprobadas de 85.
Puntaje 6.533333; rechazo por decision_max y target_score del perfil inicial.

| Medición | Primera ejecución | Con caché |
| --- | ---: | ---: |
| Recolección completa del adaptador | 26.954 s | 4.819 s |
| Preparar/sincronizar espacio | 0.993 s | 0.551 s |
| Comando de compilación | 21.612 s | 0.630 s |
| Comando de pruebas | 2.153 s | 2.288 s |
| Archivos copiados | 2656 | 0 |

La segunda ejecución retiró 5 salidas generadas del espacio administrado; no
eliminó archivos originales. El log de compilación caliente no contiene nuevas
compilaciones. Ambas pruebas registraron 85 eventos aprobados.

Los tiempos son observaciones de una ejecución inicial y una reutilizada en este
equipo, no un benchmark estadístico ni una comparación aislada Rust/Python.
El cronómetro mide la recolección del adaptador, no el arranque completo del CLI
ni la lectura del catálogo y el cálculo posterior. El principal ahorro observado
corresponde a evitar la compilación completa de dependencias.

El inventario SHA-256 del original volvió a coincidir con el informe al terminar.
`git status --short` del monorepo permaneció vacío. Las pruebas usaron los servicios
de test existentes; el evaluador no modificó el código de deejai.

## Límites conservados

La aprobación es del perfil de indicadores, no una certificación universal.
Cuatro dimensiones siguen sin medirse y quedan null. La caché solo reutiliza
artefactos; las pruebas se ejecutan de nuevo. El núcleo no necesita Python ni
Elixir para calcular, pero el adaptador Elixir necesita sus herramientas.
