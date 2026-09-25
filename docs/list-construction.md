# Construcción de listas: regla ejecutable V1

`agent-quality analyze-lists PROJECT --output NEW_FILE` conecta conocimiento,
reconocimiento AST, comprobaciones de comportamiento, modelo de coste y diagnóstico.
La decisión se calcula en Rust; Elixir aporta AST y ejecución de expresiones reconocidas.
No interviene un LLM en métricas ni decisiones. El informe es independiente del score MVP.

## Alcance y seguridad

Se inspeccionan los archivos `.ex` y `.exs` bajo `lib/`, en orden estable. No se copia
el código para este modo ni se carga `mix.exs`, módulos, macros o tests del proyecto.
Se verifican hashes antes y después. Se rechazan enlaces simbólicos en el inventario.
Los logs están en una carpeta exclusiva de ejecución, fuera del proyecto.

Solo se reconocen cuerpos completos de funciones unarias con una única cláusula:

```elixir
Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end)
Enum.reverse(Enum.reduce(xs, [], fn x, acc -> [x | acc] end))
```

El archivo debe contener un único módulo literal, sin imports, aliases, atributos,
macros ni construcciones de módulo que impidan verificar los bindings. Las cláusulas
guardadas de la función evaluada y definiciones del operador `++` se rechazan conservadoramente. No se
expanden macros. Solo tras reconocer el AST completo se evalúa una función anónima
extraída, con bindings estándar, en otro proceso del sistema operativo.

Esto no verifica compilación del módulo original. Proyectos reales con `@doc`,
`@spec`, pipelines o callbacks distintos pueden quedar fuera de alcance. Lo no
reconocido no se declara eficiente. El análisis general `evaluate` sigue compilando
y probando en un workspace separado; sus pruebas pueden tener efectos externos.

## Conocimiento convertido en reglas

La regla versionada vive en `rules/list-construction.json`. Se verifican las huellas
de los libros y del texto extraído, y se conservan sus localizadores:

- *Purely Functional Data Structures*, páginas físicas 99–100, §7.1.2: coste del
  recorrido/copia de listas persistentes. La página 100 también motiva no generalizar
  a toda concatenación: segmentos con crecimiento geométrico requieren otro análisis.
- *Property-Based Testing with PropEr, Erlang, and Elixir*, unidad 28: generalización
  de ejemplos y oráculos independientes. El contrato de identidad permite comparar
  directamente con la entrada, sin usar la alternativa como único oráculo.
- La semántica específica se apoya en la documentación de
  [Kernel.++/2](https://elixir.hexdocs.pm/1.18.4/Kernel.html#++/2) y
  [Enum.reduce/3](https://elixir.hexdocs.pm/1.18.4/Enum.html#reduce/3).

Los textos no generan scripts automáticamente ni cambian reglas durante la evaluación.
La traducción de conocimiento a premisas y fórmulas es una decisión de ingeniería
versionada y testeada; el resultado de aplicarlas se calcula sin interpretación libre.

## Comportamiento y modelo

Dominio: listas propias. Se enumeran todas las listas con alfabeto `[-1,0,1]` y
longitudes 0–5 (364), más dos casos heterogéneos/repetidos: 366 comprobaciones.
Cada expresión y alternativa deben devolver exactamente la entrada. Rust comprueba
las entradas y resultados del informe, no solo un booleano enviado por Elixir.
Estas comprobaciones finitas no son una prueba universal.

Para longitud n:

| Modelo | Visitas a entrada | Visitas adicionales | Constructores lógicos |
|---|---:|---:|---:|
| append singleton | n | n(n−1)/2 | n(n+1)/2 |
| prepend + reverse | n | n | 2n |

Elixir ejecuta operaciones instrumentadas de listas; Rust verifica sus contadores
contra estas fórmulas independientes. Son unidades del modelo, no asignaciones
reales de BEAM, bytes ni duración. A n=1024: 524800 frente a 2048 constructores.
El crecimiento cuadrático/lineal se deriva del modelo, no se infiere de timings.
La recomendación requiere el patrón append, premisas verificadas, todas las
comprobaciones y menor trabajo alternativo en todos los tamaños evaluados.

## Observaciones y estados

Tamaños 128, 256, 512 y 1024; dos calentamientos y nueve muestras por variante.
Cada muestra usa un proceso BEAM nuevo, con GC previo y orden alternado entre
variantes. Se conservan tiempo en nanosegundos, reducciones y comprobación de
identidad; Rust calcula mediana y percentil 95 por rango más cercano. Son
microbenchmarks de expresiones extraídas, no del código compilado del proyecto.
Ambas variantes se construyen mediante `Code.eval_quoted`, evitando comparar una
función interpretada con una alternativa compilada. Aun así, no representan el
rendimiento de una compilación de producción.
No intervienen en decisiones o puntuaciones. La salida completa no es bit a bit
determinista por las observaciones, rutas de logs y metadatos de ejecución; los
hallazgos y costes sí lo son para las mismas fuentes y versión de regla.

Se evalúan hasta ocho sitios, con timeout de 90 segundos. Configuración cerrada
V1: cambiar dominio, tamaños o contrato requiere actualizar el validador Rust y
sus pruebas, no solo editar JSON.

- `completed_with_findings` (exit 1): sitios reconocidos con hallazgos.
- `completed` (exit 0): al menos un sitio reconocido, sin hallazgos.
- `incomplete` (exit 2): ningún sitio reconocido, errores de parseo, append no
  soportado, sitios omitidos o fallo de ejecución. Puede conservar hallazgos parciales.
- Errores de integridad/evidencia: error estructurado en stderr, exit 2, sin aprobación.

`completed` se refiere al alcance reconocido, nunca a toda la aplicación.
El informe incluye evidencia cruda, procedencia y límites explícitos.

## Ejecución como skill y pruebas

La distribución de la skill está en `skills/agent-quality`. Su wrapper ejecuta el
motor Rust; la instalación descubrible incluye un `engine-root.txt` con la ruta
local del repositorio. También admite `AGENT_QUALITY_ROOT`. No duplica el motor ni
la base de conocimiento. Invocación: `$agent-quality evalúa este proyecto ...`.

```sh
bash skills/agent-quality/scripts/run.sh analyze-lists /ruta/proyecto
cargo test --locked
elixir tests/list_construction_test.exs
cargo test --locked --test list_integration -- --ignored --test-threads=1
```
