# Mapa de conceptos y relaciones

Este mapa es una síntesis propia para localizar conocimiento. Cada relación remite a capítulos/unidades de los libros; no es un conjunto de reglas puntuables. PDF significa página física; EPUB significa unidad del spine. Ver las notas de cada fuente para premisas y pendientes.

| Concepto / términos de búsqueda | Referencias | Relación y límite |
| --- | --- | --- |
| Coste, cost model, worst-case, amortized | Haskell cap. 2, PDF 42–59; PFDS cap. 3, PDF 31–50 | El coste depende de operaciones contadas, representación y evaluación. Amortizado no significa promedio aleatorio ni latencia máxima por llamada. |
| Persistencia, persistence, sharing | PFDS caps. 2–4; Haskell cap. 3, PDF 60–75 | Reutilizar versiones puede repetir trabajo; la compartición relevante debe existir en la implementación. |
| Pereza, lazy evaluation, suspension | PFDS cap. 3 y apéndice A, PDF 143–148; Haskell caps. 1–3 | Suspender, compartir y memoizar son mecanismos distintos. Las garantías no se trasladan literalmente a evaluación estricta. |
| Invariantes, representation invariant | PFDS caps. 5–8; Haskell cap. 3, PDF 60–75; PropEr cap. 3, EPUB 26–32 | La representación impone premisas; las propiedades pueden buscar violaciones. Pasar muestras no demuestra un invariante universal. |
| Equivalencia, fusion, tupling | Haskell cap. 1, PDF 22–41; Pearls capítulos 1–30 | Una transformación necesita leyes y dominio. Eliminar listas intermedias no sustituye verificar equivalencia, terminación y costes. |
| Codicia, greedy, thinning, dominance | Haskell caps. 7–12, PDF 162–325; Pearls caps. 7–8, PDF 55–69 | Descartar candidatos exige preservar posibilidades futuras; menor coste actual no basta. Los desempates forman parte de la regla. |
| Memoización, dynamic programming | Haskell caps. 13–14, PDF 330–381; Pearls caps. 17–18, PDF 141–160 | Compartir subproblemas reduce repetición y retiene memoria. Claves completas y dependencias correctas son necesarias. |
| Secuencias, append, concatenation | PFDS cap. 7, PDF 97–116; Haskell caps. 1, 12, 14 | Asociatividad no implica mismo coste. Tampoco toda concatenación por la izquierda es cuadrática: considerar tamaños de segmentos. |
| Búsqueda, frontier, shortest path | Haskell caps. 9, 15–16; Pearls cap. 25, PDF 222–234 | Estructura de cola y tamaño de frontera son costes diferentes. BFS, Dijkstra y A* tienen hipótesis distintas. |
| Optimalidad, heuristic, admissible | Haskell cap. 16, PDF 422–448 | Una heurística admisible puede ser inconsistente; cerrar definitivamente visitados puede perder el óptimo. |
| Modelo de prueba, model, property | PropEr caps. 3, 5, 8–11, EPUB 26–32, 41–49, 61–65, 68–91 | Un modelo simple e independiente puede detectar errores; también puede estar equivocado u omitir casos. |
| Distribución, generator, shrinking | PropEr caps. 4, 6–7, EPUB 33–38, 50–60 | Generar variedad, controlar tamaño y reducir contraejemplos son tareas distintas. Shrinking no garantiza mínimo global. |
| Estado, state machine, precondition | PropEr caps. 8–10, EPUB 61–65, 68–83 | Secuencias de operaciones ejercitan interacciones que ejemplos aislados omiten; precondiciones no deben ocultar fallos. |
| Concurrencia, parallel, interleaving | PropEr cap. 9, EPUB 68–74 | Propiedades concurrentes contrastan ejecuciones con modelos. Una semilla fija no fija por sí sola la planificación del runtime. |
| Espacio, memory, materialization | Haskell caps. 11, 13–15; PFDS caps. 3, 7 | Conservar versiones, resultados y fronteras afecta memoria; contar variables sintácticas no mide espacio retenido. |

## Relaciones que deben conservarse al ampliar el corpus

- **Invariantes → propiedades:** convertir premisas de una estructura en comprobaciones, preservando el dominio. No convertir pruebas aleatorias en demostraciones.
- **Equivalencia → optimización:** mantener comportamiento antes de comparar costes; los libros de algoritmos dan contraejemplos y condiciones de transformaciones, PropEr aporta técnicas para explorarlas.
- **Persistencia → amortización → carga de operaciones:** una prueba sobre una secuencia restringida no cubre todas las mezclas o ramificaciones de versiones.
- **Representación → tiempo y espacio:** sustituir listas, árboles, arrays o suspensiones cambia las operaciones primitivas. Nombrar el mismo algoritmo no conserva automáticamente sus cotas.
- **Concurrencia → modelo y reproducibilidad:** conservar entradas, operaciones y evidencia de ejecución; una prueba concurrente no obtiene reproducibilidad total únicamente fijando aleatoriedad.

## Vacíos para el futuro diseño

No hay en este corpus una ponderación objetiva y universal entre factores de calidad. Tampoco una equivalencia entre complejidad ciclomática y complejidad temporal, ni un método general para deducir cotas exactas de cualquier programa. Las propiedades operativas de OTP, presión sobre mailboxes, supervisión y cargas concretas necesitarán fuentes y contratos específicos. Estos son límites de cobertura, no penalizaciones de código.
