# bonva — núcleo Rust, adaptadores por lenguaje

**bonva** es un evaluador determinista de calidad de código. Su nombre homenajea
a Eduardo Bonvallet, el Gurú, con un guiño a Refactoring Guru.

El repositorio vive en [Javendanon/bonva](https://github.com/Javendanon/bonva).
El paquete Rust, el ejecutable y la skill conservan el identificador técnico
`agent-quality`; los comandos existentes siguen vigentes.

El núcleo valida evidencia, normaliza indicadores, calcula puntajes y aplica gates.
El primer adaptador evalúa proyectos Elixir/Mix. No hay un LLM en el cálculo ni en
la decisión de aprobación.

## Ejecutar

Requiere Rust/Cargo para construir. El binario usa los archivos de configuración
de este repositorio; no es todavía una distribución autónoma.
El adaptador Elixir requiere Elixir 1.18+, Mix, dependencias del proyecto y sus
servicios de pruebas. Los comandos de consulta y cálculo del núcleo no necesitan
Elixir ni Python.

```sh
cargo build --release --locked
./target/release/agent-quality adapters
./target/release/agent-quality evaluate /ruta/proyecto_mix --output reports/nuevo.json
./target/release/agent-quality evaluate fixtures/good --baseline fixtures/bad --output reports/comparacion-rust.json
```

Los informes en `reports/` son salidas locales y no se versionan.

### Skill y análisis de listas

La skill distribuible está en `skills/agent-quality`. Instalada en el directorio
de skills de Codex, se invoca como `$agent-quality`. Su wrapper construye y ejecuta
el núcleo Rust, sin duplicarlo ni usar Python para evaluar:

```sh
bash skills/agent-quality/scripts/run.sh analyze-lists /ruta/proyecto --output reports/listas-nuevo.json
bash skills/agent-quality/scripts/run.sh evaluate /ruta/proyecto --output reports/evaluacion-nueva.json
```

`analyze-lists` conecta fuentes verificadas con detección AST, 366 comprobaciones
de identidad, un modelo de coste validado independientemente y microbenchmarks.
Lee el original sin copiarlo ni cargar módulos del proyecto. La primera regla
solo cubre copia identidad mediante append de singleton o prepend + reverse.
No altera el score MVP; timings y reducciones se reportan aparte de los costes
calculados. [Alcance, fórmulas y estados](docs/list-construction.md).

Para listas, salida 0 significa análisis acotado completado sin hallazgos;
1, hallazgos; 2, incompleto/error. Ninguno certifica eficiencia de toda la aplicación.

Opciones: `--policy`, `--profile`, `--cache-dir`, `--baseline`, `--output`.
Los informes no se sobrescriben y deben quedar fuera de los proyectos evaluados.
Salida 0: aprobado y, si aplica, mejorado. Salida 1: rechazado o sin mejora.
Salida 2: error o evidencia incompleta.

`agent-quality score METRICS.json` recalcula evidencia suministrada sin ejecutar
herramientas. Su resultado se marca `unattested_replay`, no acredita evaluación
de un proyecto y carece de capacidades verificadas para aprobarlo.

## Qué cambió

- **Rust:** núcleo puro, validación JSON, procedencia, diagnóstico, comparación,
  procesos con timeout y gestión del espacio de trabajo.
- **Elixir:** scripts externos que parsean código y capturan eventos ExUnit.
- **Análisis directo:** el parser lee el original sin compilarlo ni expandir macros.
  El hash del texto realmente parseado debe coincidir con el inventario de entrada.
- **Compilación reutilizable:** el proyecto se sincroniza en `.quality-cache/`.
  Solo se copian archivos nuevos o modificados y se retiran los eliminados.
  Se conserva `_build` y se deja a Mix decidir qué recompilar.
- **Pruebas frescas:** ExUnit se ejecuta en cada evaluación; no se cachean resultados.
- **Original intacto:** compilación y tests se ejecutan fuera del proyecto, y el
  inventario original se verifica al terminar.

Un bloqueo exclusivo impide usar simultáneamente el mismo espacio. Cambios en
dependencias, mix.exs, mix.lock, config, runtime, configuración de ejecución o
entorno generan otro espacio de compilación. Tras una ejecución interrumpida
o que alteró entradas, se descarta el build incompleto del espacio administrado.
Los espacios anteriores permanecen en la caché; todavía no hay recolección automática.

La copia separa archivos, pero no aísla red, bases de datos ni efectos externos.
Se evalúan proyectos de confianza con servicios configurados para pruebas.

## Perfil inicial

`profiles/elixir-mvp.json` describe capacidades necesarias, fórmulas y gates.
`scoring/policy.json` conserva pesos, umbrales, semilla y tolerancias.
La ausencia de una capacidad requerida bloquea aprobación.

| Dimensión | Indicador | Peso |
| --- | --- | ---: |
| correctness | Pruebas aprobadas / registradas × 10 | 0.40 |
| structural_complexity | Máximo de decisiones por cláusula | 0.30 |
| maintainability | Máximo de parámetros por cláusula | 0.15 |
| readability | Percentil 95 de líneas por cláusula | 0.15 |
| test_strength | No medida | 0 |
| algorithmic_efficiency | No medida | 0 |
| runtime_efficiency | No medida | 0 |
| memory_efficiency | No medida | 0 |

Las dimensiones sin evidencia son `null`. Si una recibe peso positivo, no hay
puntaje total ni aprobación. Los umbrales expresan una política local: los libros
no definen estos pesos ni una escala universal de calidad.
Detectar `++` genera una recomendación consultiva, sin afirmar Big-O ni restar puntos.

Se usan decimales y redondeo a seis posiciones. La versión Rust acepta política
numérica de hasta seis decimales y rechaza desbordamientos. La equivalencia se
comprueba con los informes previos y 64 casos de referencia del motor Python.

## Estructura

| Ruta | Responsabilidad |
| --- | --- |
| `src/engine.rs` | Cálculo independiente del lenguaje |
| `src/adapters/` | Contrato Adapter e integración Elixir |
| `src/workspace.rs` | Sincronización, inventario, bloqueo e invalidación |
| `src/runner.rs` | Procesos separados, logs y timeout |
| `src/report.rs` | Evidencia, procedencia y diagnóstico |
| `profiles/` | Requisitos del perfil de evaluación |
| `rules/`, `knowledge/` | Reglas y fundamentos trazables |
| `analyzers/*.exs` | Herramientas Elixir con salida estructurada |
| `tests/*.rs` | Pruebas Rust y de integración |

Solo hay un adaptador instalado. Añadir un lenguaje requiere implementar el
contrato, declarar capacidades y proporcionar pruebas; un perfil JSON no inventa
soporte de herramientas. El motor ya se prueba con un perfil sin conceptos Elixir.

## Validación

```sh
cargo test --locked
cargo test --locked --test elixir_integration -- --ignored --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Las pruebas de integración se ejecutan explícitamente porque necesitan Mix y
permisos para sockets locales. Cubren ejecuciones sin caché y con caché, cambios de
fuente, pruebas fallidas y vacías, alteraciones de entrada y conservación del original.

Los ejemplos malos en `fixtures/bad` son intencionales: sirven para probar al
evaluador, no son código recomendado. Su resultado se conserva: 6.683333 frente
a 10 en la versión mejorada, con el mismo comportamiento ejercitado.

## Migración y límites

La implementación Python anterior se conserva como referencia de migración y
para sus pruebas históricas. El binario Rust no la invoca. La consulta de libros
`scripts/knowledge.py` sigue disponible como herramienta independiente; no es
una dependencia del evaluador.

El revisor devuelve instrucciones y no modifica proyectos. No se implementó
un optimizador automático. Tampoco se mide aún cobertura, mutación, crecimiento
algorítmico, memoria ni escalabilidad concurrente.

Ver [arquitectura Rust](docs/rust-architecture.md) y
[validación de la migración](docs/rust-validation.md).
