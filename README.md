# Bonva

Bonva evalúa proyectos Elixir/Mix con evidencia obtenida del código fuente,
la compilación y las pruebas. Produce un informe JSON con indicadores de calidad,
puntajes, condiciones de aprobación y recomendaciones trazables.

El motor está escrito en Rust. Las puntuaciones se calculan con una política
explícita; un modelo de lenguaje no interviene en el cálculo ni en la aprobación.
El nombre homenajea a Eduardo Bonvallet, el Gurú.

## Qué hace

- Analiza el código Elixir sin ejecutar módulos ni expandir macros.
- Compila y ejecuta ExUnit en una copia del proyecto, verificando que el original
  conserve sus archivos. Reutiliza compilaciones; las pruebas se ejecutan siempre.
- Calcula indicadores y aplica condiciones de aprobación (*gates*), como ausencia
  de fallos, cantidad mínima de pruebas y disponibilidad de evidencia.
- Compara un candidato con una versión base y detecta regresiones, pérdida de
  mediciones y cambios en el contrato de pruebas.
- Analiza patrones acotados de construcción de listas con un modelo de coste,
  comprobaciones de identidad y mediciones separadas del puntaje general.

Bonva devuelve un diagnóstico e instrucciones. No modifica ni refactoriza
los proyectos evaluados automáticamente.

## Requisitos

- Rust/Cargo para construir el ejecutable.
- Elixir 1.18+ y Mix para evaluar proyectos o analizar listas.
- Dependencias y servicios de pruebas del proyecto configurados.

El ejecutable usa los recursos de este repositorio; aún no es una distribución
autónoma. Su nombre técnico y el de la skill son `agent-quality`.

## Uso

Desde la raíz del repositorio:

```sh
cargo build --release --locked
./target/release/agent-quality adapters
./target/release/agent-quality evaluate /ruta/proyecto_mix --output reports/evaluacion.json
```

Para comparar las fixtures incluidas:

```sh
./target/release/agent-quality evaluate fixtures/good --baseline fixtures/bad --output reports/comparacion.json
```

Opciones de evaluación: `--policy`, `--profile`, `--cache-dir`, `--baseline` y
`--output`. Los informes deben quedar fuera de los proyectos evaluados y usar
un archivo nuevo. Las salidas de `reports/` no se versionan.

La evaluación devuelve código 0 si aprueba y, cuando hay baseline, mejora;
1 si rechaza o no mejora; 2 si hay un error o evidencia incompleta.

### Construcción de listas

```sh
./target/release/agent-quality analyze-lists /ruta/proyecto --output reports/listas.json
```

Reconoce copias de listas mediante append de un elemento o prepend seguido de
reverse. Lee el original sin compilarlo y no altera el puntaje de evaluación.
Devuelve 0 si completa el análisis sin hallazgos, 1 si encuentra problemas dentro
del alcance de la regla y 2 si el análisis queda incompleto o falla.
Ver [alcance del análisis de listas](docs/list-construction.md).

### Integración con agentes

La skill está en `skills/agent-quality`. Su wrapper construye y ejecuta el motor:

```sh
bash skills/agent-quality/scripts/run.sh evaluate /ruta/proyecto --output reports/otra-evaluacion.json
```

El comando `agent-quality score METRICS.json` permite recalcular métricas
suministradas. Se identifica como `unattested_replay`: no acredita una evaluación
real del proyecto ni dispone de las capacidades verificadas para aprobarlo.

## Qué mide y cuáles son sus límites

El perfil de evaluación está en `profiles/elixir-mvp.json`; los pesos y umbrales,
en `scoring/policy.json`.

| Dimensión | Indicador | Peso |
| --- | --- | ---: |
| Corrección | Pruebas aprobadas / registradas × 10 | 0.40 |
| Complejidad estructural | Máximo de decisiones por cláusula | 0.30 |
| Mantenibilidad | Máximo de parámetros por cláusula | 0.15 |
| Legibilidad | Percentil 95 de líneas por cláusula | 0.15 |

Son indicadores definidos por una política local, no una escala universal de
calidad. Las dimensiones sin evidencia quedan en `null`; si tienen peso positivo,
no hay puntaje total ni aprobación. El evaluador todavía no mide cobertura,
mutación, eficiencia general de algoritmos, memoria ni escalabilidad concurrente.
Detectar `++` por sí solo genera una recomendación, no una afirmación de Big-O.

La copia de trabajo separa archivos, pero no aísla red, bases de datos ni otros
efectos externos. Bonva evalúa proyectos de confianza con servicios de pruebas
configurados. Actualmente el único adaptador disponible es Elixir/Mix.

## Conocimiento y desarrollo

`knowledge/` contiene fuentes, notas y conceptos que fundamentan las reglas.
La herramienta independiente `scripts/knowledge.py` permite importar, verificar
y consultar libros. Es el único componente que requiere Python; el evaluador
Rust no lo ejecuta. Ver [guía de conocimiento](knowledge/README.md).

```sh
cargo test --locked
cargo test --locked -- --ignored --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
python3 -m unittest discover -s tests -v
```

Las integraciones requieren Elixir/Mix y permisos para sockets locales. Las
fixtures incluyen ejemplos deliberadamente deficientes y versiones mejoradas
para comprobar el evaluador. Ver [arquitectura Rust](docs/rust-architecture.md).
