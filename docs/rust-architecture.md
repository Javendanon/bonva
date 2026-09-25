# bonva — arquitectura Rust v0.3

## Fronteras

El motor de puntuación recibe métricas JSON, política, perfil y capacidades.
No lee código, ejecuta Mix ni depende de Python o Elixir. Sus expresiones permiten
ratios, normalización por coste, comparaciones, suma de contadores y condiciones.
Las dimensiones protegidas y requisitos se declaran en el perfil.

El contrato `Adapter` declara identidad, capacidades y una operación de medición.
Elixir implementa el primer adaptador. Los scripts externos intercambian JSON
con un protocolo versionado: AST con hash de entrada y contadores al finalizar ExUnit.
Los errores de herramientas o de decodificación nunca se convierten en éxito.

La capa de informe enlaza hallazgos con las reglas y conceptos existentes. Las
fuentes citadas se verifican por hash. No hay generación de cifras por un modelo.
El contrato de informe conserva las ocho dimensiones del MVP.

## Espacio de ejecución

1. Canonicalizar rutas y rechazar superposición entre caché y original.
2. Inventariar archivos originales por contenido y permisos; rechazar enlaces y archivos especiales.
3. Separar los directorios de salida declarados (por defecto tmp).
4. Tomar un bloqueo del sistema operativo por proyecto.
5. Elegir un espacio según runtime, adaptador, dependencias, configuración y entorno.
6. Sincronizar diferencias de archivos, conservar compilados completos y verificar la copia.
7. Parsear directamente el original y contrastar el hash del texto parseado.
8. Ejecutar compilación incremental y toda la suite de pruebas en el espacio separado.
9. Verificar cambios de entradas y que el original siga intacto.
10. Marcar el espacio como reutilizable solo tras una ejecución completa válida para caché.

Una suite con fallos conserva compilación reutilizable si finalizó y no alteró
entradas; sus resultados se vuelven a medir. Un timeout, error de compilación o
cambio de entradas impide conservar el marcador de finalización. La próxima
ejecución reconstruye _build. No se comparten compilados entre proyectos distintos.

La invalidación de dependencias/configuración es conservadora y puede recompilar
más de lo necesario. Para cambios ordinarios de fuente, Mix gestiona su propio
análisis incremental. Los hashes del original se vuelven a calcular: aún hay
lecturas completas aunque se eviten copias y recompilaciones completas.

El adaptador actual está probado en macOS/Unix. El runner usa grupos de procesos
para terminar descendientes al agotarse el timeout. Los bloqueos se liberan al
cerrarse el proceso, sin archivos de bloqueo obsoletos que requieran borrado manual.
Los logs separan stdout y stderr. La caché es local, propia del evaluador; no es
un mecanismo de ejecución segura de código hostil ni un aislamiento de servicios.

## Determinismo y precisión

Los tiempos de preparación, herramientas y recolección se registran en campos
de ejecución. Son sensibles al entorno y nunca entran al puntaje. Comparar dos
ejecuciones significa comparar métricas, gates, vectores, hallazgos y hashes;
no exigir que coincidan tiempos, UUID, logs o rutas del espacio de trabajo.

El núcleo usa rust_decimal, nunca aritmética binaria para calcular el score.
La política numérica admite seis decimales como máximo; se rechazan entradas
fuera de rango y operaciones que desborden. La salida usa seis decimales con
empates alejándose de cero, compatible con la salida half-up del motor previo.
No se afirma equivalencia para todas las entradas del Decimal arbitrario de Python:
el contrato Rust limita explícitamente su dominio numérico. Se comprueban casos
de borde y los resultados reales previos dentro del dominio soportado.

## Compatibilidad y evolución

Se conservan el perfil, fórmulas y gates iniciales. El conjunto de 64 casos de
referencia se generó antes de sustituir la ruta de ejecución, con semilla 481.
Las pruebas normales Rust leen esa evidencia fija y no ejecutan Python.
Las integraciones sobre fixtures verifican igualdad de diagnóstico entre
ejecuciones con caché y sin ella. No dependen de informes locales guardados.

Los scripts Python se mantienen como referencia histórica y herramienta de
consulta del corpus; no forman parte del camino de ejecución Rust. Los archivos
de la migración parcial a un núcleo Elixir fueron retirados.

Pendientes: distribución del binario con recursos, adaptadores adicionales,
clasificación de métricas avanzadas, límites de retención de logs/caché y medición
de costes bajo contratos de carga acordados. No hay restauraciones ni refactors
automáticos de proyectos originales.

Referencias de implementación: [grupos de procesos Rust](https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#method.process_group),
[bloqueos de archivo](https://docs.rs/fs2/latest/fs2/trait.FileExt.html),
[precisión decimal](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html).
