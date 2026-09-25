# Algorithm Design with Haskell — notas de lectura

Richard Bird y Jeremy Gibbons, 2020. Fuente `haskell`; localizadores: páginas físicas del PDF, numeradas desde 1. En el cuerpo, página física = página impresa + 17.

Estado: lectura parcial del texto extraído. Las fórmulas y el código pueden perder símbolos en la extracción; los originales son la autoridad. Estas notas son paráfrasis, no reglas de puntuación ni garantías transferidas a Elixir.

## Alcance declarado por los autores — PDF 14–20

Los ejercicios y respuestas forman parte integral del contenido. El libro desarrolla división y conquista, algoritmos voraces, reducción de candidatos, programación dinámica y búsqueda exhaustiva. No aborda en general la eficiencia espacial ni los problemas completos de sistemas en producción. Las definiciones didácticas no representan necesariamente las implementaciones de bibliotecas.

## 1. Programación funcional — PDF 22–41

- Un alias de tipo `Nat = Int` no impone por sí mismo valores no negativos: conservar los contratos semánticos además de los tipos.
- Recursión estructural y recursión general pueden especificar el mismo problema y conducir a algoritmos distintos. No inferir eficiencia por estilo superficial.
- `foldr` no implica evaluar toda la lista ni procesarla materialmente desde la derecha: el consumidor y la evaluación perezosa pueden detener el trabajo. No trasladar esto automáticamente a evaluación estricta.
- La fusión evita estructuras y recorridos intermedios cuando se satisface su condición algebraica. Para listas infinitas existen condiciones adicionales de estrictitud; la variante sensible al contexto requiere la condición solamente en estados alcanzables. Referencias: PDF 29–30 y respuestas 1.16–1.18, PDF 39–40.
- Acumular una lista mediante concatenaciones crecientes puede repetir trabajo; acumular funciones permite cambiar la asociación. Mantener simultáneamente un valor y su agregado evita recalcularlo (`collapse`, `steep`). El tupling también puede oscurecer el código: el beneficio necesita una justificación concreta.
- La igualdad de pliegues con operación asociativa e identidad vale bajo las premisas indicadas. Observación propia: no asumir esas leyes para cualquier operación numérica de máquina.
- Las respuestas muestran límites relevantes: `unwrap` es parcial; una versión de permutaciones que requiere igualdad no admite funciones como elementos; las optimizaciones de `collapse` preservan demanda parcial gracias a la pereza.

## 2. Tiempo — PDF 42–59

- Diferenciar cota superior O, inferior Ω y ajustada Θ. Una cota O(n²) por sí sola no demuestra que una implementación sea cuadrática o peor que una lineal. Declarar tamaño de entrada, mejor/peor caso y coste de operaciones elementales.
- Los promedios requieren una distribución explícita. Los costes amortizados no son promedios probabilísticos.
- El modelo del libro cuenta reducciones; admite que no son una medida completamente fiel del tiempo transcurrido. Para muchos análisis supone evaluación estricta como cota del número de reducciones; esto no constituye una garantía de latencia real ni de memoria.
- El coste de composición depende del tamaño de salida intermedia y de cuánto demanda el consumidor. `head . concat`, `length . inits` y `length . tails` ilustran por qué contar la salida completa cambia el problema medido. Compartir sufijos no evita el coste de imprimir todos sus caracteres.
- Con m listas de longitud n, la concatenación desde la derecha y desde la izquierda tienen costes distintos; no eliminar una dimensión de entrada sin declararlo. Los costes de enumeración deben compararse también con el volumen de resultados solicitado.
- Una secuencia de inserciones y descartes puede ser lineal aunque una inserción individual sea lineal: cada elemento se descarta como máximo una vez. El argumento requiere esa secuencia de uso. Conectar con persistencia y potenciales de `pfds`, no extrapolar a reutilización arbitraria de versiones.
- El método del potencial exige conservar el potencial inicial y final; omitir el inicial solo es válido bajo la condición correspondiente. Duplicar capacidad permite coste amortizado constante, sin asegurar coste constante por operación. El libro señala también interrupciones por recolección de basura.
- Duplicar tamaños y observar tiempos sirve como orientación empírica; observación propia: no convierte una muestra finita de tiempos en demostración de una clase asintótica.

### Pasajes que requieren cotejo antes de reutilizar fórmulas

Observaciones propias sobre el texto extraído, todavía no erratas confirmadas del original:

- PDF 46: después de resolver una recurrencia exponencial, un párrafo afirma cotas lineales incompatibles con lo anterior.
- Respuesta 2.1, PDF 57: aparece Θ(n) donde el contexto trata Θ(1).
- Respuesta 2.8, PDF 58: la solución transcrita parece incompatible con la recurrencia del ejercicio en PDF 56.
- Respuesta 2.13, PDF 59: el promedio para una longitud finita parece confundirse con su límite; no incorporar su expresión literalmente como regla.

## 3. Estructuras útiles — PDF 60–75

Las listas simétricas representan una lista mediante dos listas, invirtiendo conceptualmente la segunda. La función de abstracción explica qué valor representa el estado interno. Las operaciones deben preservar el invariante además de cumplir la ecuación de correspondencia. Las garantías anunciadas son amortizadas. El texto omite mensajes de error en ejemplos por simplicidad didáctica, sin convertir esa omisión en recomendación de producción.

- La prueba del potencial para listas simétricas trata una secuencia desde la estructura vacía. Su uso con persistencia ramificada requiere el análisis adicional de `pfds`.
- Las listas de acceso aleatorio usan dígitos y árboles perfectos con tamaños almacenados mediante constructores que mantienen el invariante. Mejorar acceso/actualización puede empeorar otras operaciones. Incrementos y decrementos aislados tienen una garantía amortizada que no sobrevive automáticamente a su mezcla (PDF 68).
- Distinguir construir prefijos, contar prefijos y materializar/imprimir todo su contenido. Una solución lineal que invierte la entrada no sirve como algoritmo en línea para un flujo infinito.
- Las operaciones sobre los arrays inmutables de este capítulo favorecen acceso y actualizaciones en lote; repetir actualizaciones individuales copia repetidamente. No generalizar ese coste a toda estructura funcional llamada array.
- Las ecuaciones de abstracción tienen dominios: índices fuera de rango pueden producir error aunque una especificación con filtros simplemente los ignore (PDF 69).
- Pendientes de cotejo: PDF 66 atribuye O(log k) al acceso, pero los ceros iniciales para longitud potencia de dos requieren examinar la afirmación; PDF 75 usa 10 en una definición parametrizada por n. Registrar como sospechas propias, no hechos establecidos.

## 4. Búsqueda binaria — PDF 80–109

- Monotonía estricta, cotas del intervalo y reducción efectiva son premisas de la búsqueda. Tipos enteros de precisión limitada pueden invalidar una implementación aparentemente correcta. Contar evaluaciones de una función no equivale a contar tiempo si sus valores requieren aritmética costosa.
- Buscar primero límites por duplicación puede reducir enormemente el intervalo. Observación propia: no tomar la afirmación del ejemplo sobre precisión limitada como garantía contra todo desbordamiento posible de una función arbitraria.
- Las recurrencias simplificadas ignoran suelos/techos para obtener cotas; los conteos exactos necesitan tratarlos y no asumir que todo algoritmo tiene coste monótono con el tamaño.
- La búsqueda bidimensional depende de ambas dimensiones. Saddleback es óptimo asintóticamente en una cuadrícula cuadrada; búsqueda binaria por filas/columnas puede ser preferible en rectángulos. La igualdad de los conjuntos de soluciones no implica el mismo orden de enumeración.
- Un árbol binario de búsqueda solo asegura acceso logarítmico con altura logarítmica. Los constructores que almacenan altura y las rotaciones mantienen invariantes locales. `balance` requiere subárboles equilibrados con diferencia de altura acotada; `gbalance` aborda una precondición más amplia.
- Las cotas inferiores de ordenación por comparaciones corresponden a ese modelo: no prohíben algoritmos que exploten la estructura de enteros o palabras.
- Conjuntos dinámicos: combinar árboles requiere también separación de sus claves, además de equilibrio. La función de abstracción representa conjuntos, no igualdad estructural de todas sus representaciones. `pieces`/`sew` se relacionan con zipper; el coste se justifica sumando diferencias de alturas.
- Pendientes de cotejo: PDF 88 descripción del último caso de la búsqueda; PDF 104 respuesta 4.2 parece desplazar el índice respecto a `smallest`; PDF 107 cota de unión por inserciones necesita conservar ambos parámetros (n pequeño frente a m grande); PDF 108 respuesta 4.17 anuncia linealidad pese a reconstruir con `mktree`. No convertir estas afirmaciones en reglas sin resolverlas.

## 5. Ordenación — PDF 110–137

- Criterios distintos: tiempo, adaptación a entradas parcialmente ordenadas, estabilidad y espacio. No hay una superioridad universal independiente de tamaño, representación y contrato. Las comparaciones históricas del libro no son benchmarks actuales.
- Quicksort con primer pivote puede tener peor caso cuadrático en entradas ordenadas. Elegir la posición central no equivale a elegir la mediana de valores.
- Mergesort: repartir alternadamente reduce recorridos de partición, pero pierde estabilidad. Combinar pares desde abajo evita particiones repetidas. Detectar tramos ordenados permite adaptación; invertir tramos descendentes exige desigualdad estricta para preservar orden entre iguales.
- Heapsort: construir el heap puede ser lineal al sumar costes por altura; sumar el peor coste de la raíz para todos los nodos sobreestima. Las propiedades de implementaciones imperativas en sitio no se trasladan a listas persistentes.
- Comparar palabras cuesta según sus caracteres. Bucket/radix explotan estructura y rango; mantener orden de cada cubeta importa. Concatenar al final de una cubeta repetidamente puede ser cuadrático; insertar al frente y revertir al extraer evita ese trabajo.
- Observación propia: las cotas de bucket/radix necesitan incluir rango y coste de discriminadores, no tratarlos como constantes sin declararlo. La versión ilustrativa que convierte y recorre números para cada dígito merece su propio análisis.
- Ordenar sumas: reducir comparaciones entre elementos originales no reduce necesariamente trabajo total; existen comparaciones auxiliares, índices y estructuras. Las hipótesis algebraicas de las transformaciones son parte del resultado. El estado de problema abierto se conserva como afirmación histórica de la edición 2020.
- Ejercicios: casos base también garantizan terminación; decorar elementos con su clave evita recalcularla en cada comparación; fusión puede revelar que dos algoritmos con nombres distintos son la misma implementación.
- Pendiente de cotejo y contraejemplo: respuesta 5.19, PDF 136, rellena palabras con `a`; esto puede confundir palabras de longitudes diferentes (por ejemplo `aa` y `a`) y no justifica orden lexicográfico general. En la optimización de quicksort, preservar valores ordenados no basta para preservar estabilidad de registros con claves iguales.

## 6. Selección — PDF 138–157

Calcular mínimo y máximo juntos reduce recorridos; reducir comparaciones puede aumentar otros costes. El algoritmo divide y conquista con particiones repetidas puede ser Θ(n log n) en tiempo aunque ahorre comparaciones. La combinación desde abajo obtiene otro coste: registrar ambas medidas por separado, no puntuar por número de pasadas únicamente.

- Selección por rango requiere aclarar índice desde cero/uno, mediana inferior/superior y duplicados. Particionar en menor/igual/mayor evita recursión innecesaria entre iguales. El caso de un elemento evita recursión circular entre `pivot` y `select`.
- Mediana de medianas: el tamaño de grupos y la suma de fracciones de subproblemas importan. No basta detectar dos llamadas recursivas para concluir coste exponencial.
- Seleccionar en dos colecciones ya ordenadas admite ahorro logarítmico si acceso, tamaño y representación lo permiten. Construir arrays o instalar tamaños no es gratis; el libro excluye explícitamente ese preprocesamiento en un ejemplo.
- Menor natural ausente: el principio de cardinalidad acota candidatos por tamaño de entrada. La variante con conteos tolera duplicados; la variante de partición usa ausencia de duplicados como premisa.
- Una relación que exige que todos los elementos de una lista sean menores que los de otra no es transitiva cuando la lista intermedia está vacía (respuesta 6.13). Atención a verdades vacías en propiedades.
- Cotejar antes de formalizar: desigualdad exacta de grupos de cinco (PDF 144, 153, 155) y casos de grupos incompletos; contar comparación trivaluada o pruebas booleanas separadas cambia los conteos de respuesta 6.12.

## 7. Algoritmos voraces sobre listas — PDF 162–193

- Separar generación de candidatos, criterio de coste y selección. `minWith` determinista escoge el primero o último entre empates según su definición: cambiar enumeración puede cambiar el resultado aunque no cambie el coste.
- Fallar una condición suficiente general no demuestra que un algoritmo sea incorrecto: la condición sensible al contexto solo exige estados alcanzables. Los ejemplos de ordenación muestran esta diferencia.
- Sustituir el criterio de coste puede facilitar una prueba, pero exige demostrar que minimizar el nuevo criterio satisface el original. En monedas, máximo lexicográfico no minimiza automáticamente cantidad ni peso; las denominaciones son parte del contrato y del contraejemplo.
- `MinWith` no determinista es notación de especificación/refinamiento, no una extensión ejecutable de Haskell ni recomendación de introducir azar. Una implementación puede escoger determinísticamente uno de los resultados admisibles. Mantener esta distinción para el requisito del usuario.
- Refinamiento permite conservar un óptimo sin conservar el mismo desempate. No usarlo para justificar cambiar una salida si el contrato exige exactamente el resultado previo.
- Decimales de TeX: cotas de dígitos, intervalos, redondeo y precisión limitada forman parte de la derivación. La especificación con infinitos candidatos no es un programa terminante. Un algoritmo voraz evita grandes enteros mediante invariantes específicos.
- Ejercicios/respuestas: igualdad de resultados concretos no comprueba que dos métodos alcancen el mismo coste óptimo cuando hay empates (7.13); una observación propia es comparar factibilidad y coste según contrato. El orden de evaluación perezoso puede hacer que insertion sort no ejecute los pasos sugeridos por su nombre.
- Pendientes: respuesta 7.6 da un ejemplo para una lista concreta, no establece necesidad o ausencia de necesidad para todas las listas; fórmulas y límites del ejemplo TeX requieren cotejo visual antes de reutilización.

## 8. Algoritmos voraces sobre árboles — PDF 194–221

- Altura mínima, coste de altura con hojas ponderadas y coste Huffman son objetivos diferentes. Dos árboles de igual coste actual pueden admitir extensiones de costes diferentes; conservar solo ese escalar puede perder el óptimo. El coste lexicográfico de la espina añade la información necesaria en el ejemplo.
- Separar generar espinas y reconstruir árboles evita reconstrucciones repetidas; guardar coste junto al árbol evita recalcularlo. El coste lineal final se prueba para la secuencia de combinaciones, no para cada combinación aislada.
- La teoría voraz general necesita terminación, corrección en estados finales y preservación de algún óptimo. Para pasar de existencia de un óptimo común a refinamiento, el paso no debe introducir candidatos ajenos al conjunto anterior.
- Huffman minimiza longitud ponderada dentro del contrato de códigos prefijos. Este capítulo solo construye el árbol: no implementa todo el formato, muestreo, codificación y decodificación. Imponer el orden original de las hojas cambia el problema al de árboles alfabéticos.
- El algoritmo de dos colas es lineal con pesos inicialmente ordenados y costes amortizados adecuados. Incluir la ordenación si forma parte de la tarea. Una cola de prioridad permite otra implementación con costes distintos.
- Heap izquierdo: rango es longitud del camino más corto a vacío, no altura global. La propiedad izquierda permite operar por las espinas derechas; los constructores deben preservar rango y prioridad.
- Pendientes propios: PDF 215 dice O(log r) para combinar árboles de rango r aunque describe recorridos de longitud r; distinguir consultar mínimo de extraerlo. La versión stack/queue de Huffman en PDF 212–213 necesita revisar entrada singleton: condición final solo mira la cola, inicialmente vacía. No certificar casos borde a partir del pseudocódigo.

## 9. Algoritmos voraces sobre grafos — PDF 222–253

- Grafos dirigidos/no dirigidos, conectividad, lazos, multiaristas y pesos tienen contratos distintos. Árbol de expansión mínimo no implica caminos mínimos entre vértices; cubrir un subconjunto de vértices es otro problema.
- Kruskal y Prim crecen de modo diferente: una arista descartada por formar ciclo puede descartarse definitivamente en Kruskal; una arista todavía desconectada del árbol de Prim puede servir después. La validez futura de una operación importa.
- Union–find: nombres de conjuntos no son vértices arbitrarios; unir por tamaños limita cadenas. Costes cambian al representar tablas con arrays inmutables o listas de acceso aleatorio. Separar grafos densos y dispersos; no heredar cotas de implementaciones mutables ni afirmaciones históricas sobre mejores algoritmos.
- Dijkstra presupone pesos no negativos y alcance desde la raíz para el algoritmo presentado. El invariante distingue distancias definitivas de tentativas. El ejemplo con pesos negativos falla incluso sin ciclo negativo.
- Representar infinito con `maxInt` requiere operaciones especiales; observación propia: manejar el centinela no resuelve por sí solo desbordamiento de sumas de distancias finitas.
- Búsqueda bidireccional no puede detenerse sin justificación al primer vértice común: el ejercicio da un contraejemplo. El problema del corredor usa ciclo, raíz y pesos específicos; la propiedad de una sola arista fuera del árbol se prueba para su variante no dirigida.
- Ejercicios muestran que igualdad de estructuras puede ser un sustituto válido de identidad solo bajo invariantes particulares. Un algoritmo para grafo conectado puede fallar en desconectado; necesita otro criterio de terminación para producir un bosque.
- Pendientes propios: revisar coherencia del número de pasos, lista inicial de vértices frescos y raíz en Prim/Dijkstra (PDF 235–240); revisión del código propuesto para bosques desconectados en respuesta 9.7 y su llamada recursiva cuando se agotan aristas. Cayley en respuesta 9.3 corresponde al grafo completo, no a todo grafo de n vértices.

## 10. Reducción de candidatos — PDF 258–283

- `ThinBy` especifica una subsecuencia que domina a los candidatos descartados mediante una relación reflexiva y transitiva. No exige orden total ni eliminación máxima. La identidad es una implementación válida pero no reduce trabajo.
- Dominancia parcial puede conservar alternativas incomparables. Buscar la reducción más corta puede costar más que una reducción parcial lineal; orden de los candidatos determina eficacia del método concreto.
- Las leyes del operador no determinista no son necesariamente leyes de cada implementación determinista: una implementación de `thinBy` que elimina solo un elemento no es idempotente (respuesta 10.6).
- Introducir reducción exige compatibilidad con coste; anticipar filtros exige que lo descartado no pueda volverse válido mediante extensión. Para preservar extensiones se necesita que toda continuación del candidato descartado tenga una continuación al menos tan buena entre los conservados.
- En caminos por capas, conservar mejor camino para cada origen permite futuras conexiones; conservar solo el más barato global falla. Profundidad, anchura, número de aristas y estructura de acceso afectan a las cotas. El DAG admite aquí pesos negativos sin el problema de Dijkstra.
- En cambio de monedas, igualdad de residuo y menor cantidad permiten dominancia. Menor residuo y menor cantidad no bastan: respuesta 10.16 proporciona un contraejemplo. Agrupar residuos mejora la eliminación.
- Mochila 0/1, entera y fraccionaria son problemas distintos. La dominancia valor/peso sirve bajo sus premisas; con pesos reales pueden sobrevivir exponencialmente muchos candidatos.
- Observación propia: O(nw) en capacidad entera w es seudopolinómico si w está codificado en binario; no llamarlo lineal en tamaño de entrada sin definir tamaño. También hay que contar el coste de combinar listas: `mergeBy` de respuesta 10.15 usa un pliegue de merges y no garantiza coste lineal en toda colección de listas. Revisar las cotas que dependen de él antes de automatizar.

## 11. Segmentos y subsecuencias — PDF 284–305

- Segmento es contiguo; subsecuencia puede saltar posiciones. Permitir vacío cambia el resultado de máximo segmento cuando todos los valores son negativos. Distinguir secuencia estrictamente creciente de no decreciente.
- LIS conserva longitud y posibilidad de extensión (primer elemento al construir desde la derecha). El vacío debe permanecer disponible. Un árbol equilibrado permite búsqueda y actualización sin copiar un array entero.
- LCS recursiva directa repite subproblemas. Reducirla a LIS exige codificar posiciones repetidas en orden inverso para no reutilizar un símbolo indebidamente; considerar tamaño de la codificación, que puede ser cuadrático. La variante por reducción conserva posición más a la derecha y longitud.
- El ejemplo LCS deja un componente indefinido en candidatos inválidos y lo descarta antes de demandarlo. Ese patrón depende de pereza; no trasladarlo literalmente a Elixir.
- Scan lemma evita recomputar un fold para cada sufijo/prefijo. La reducción sola no asegura mejora: mapear todos los prefijos aún cuesta O(bn). Representarlos por diferencias, guardar sumas/longitudes, usar deque y acumuladores funcionales permite el argumento amortizado lineal del ejemplo.
- La materialización del resultado tiene un coste adicional; acotarla por longitud real y no por una cota b arbitrariamente grande. Las garantías de deques son amortizadas.
- Pendientes propios: casos b=0, n=0 y b>n del segmento acotado; respuesta 11.13 usa desigualdad estricta que necesita resolver empates; verificar antes de tomarlo como ley para cualquier elección de `msp`.

## 12. Particiones — PDF 306–325

Anticipar filtros durante construcción desde la izquierda/derecha exige cierre por sufijos/prefijos respectivamente. Las particiones usan segmentos no vacíos. En planificación de transferencias, mantener mínimo número de segmentos no basta para extender cualquier óptimo: un criterio secundario sobre el primer segmento permite el argumento voraz. El ejemplo es un modelo matemático de transacciones conocidas, no un contrato completo de un sistema bancario.


- Párrafos: palabras no vacías que caben en el ancho máximo; el ejemplo presupone tipografía monoespaciada. Excluir la última línea del coste cambia el problema. Minimizar número de líneas, desperdicio total, cuadrados o máximo desperdicio son objetivos distintos.
- Llenar cada línea sirve para algunos objetivos, pero tiene contraejemplos para otros. La reducción conserva igual ancho de última línea y coste no mayor; comparar solo costes antes de añadir una línea puede descartar el óptimo. Guardar costes y anchos evita recomputarlos.
- Los ejercicios incluyen desempates, cierres direccionales y transacciones individualmente imposibles. El contrato debe resolver esos casos antes de aplicar la transformación.

## 13. Programación dinámica y tabulación — PDF 330–351

- Subproblemas compartidos forman un grafo de dependencias. Memoización desde la demanda y tabulación desde dependencias pueden calcular conjuntos diferentes de estados. La clave debe representar todos los argumentos relevantes.
- Fibonacci ilustra tabla completa, conservación de dos resultados y duplicación rápida. Observación propia: contar operaciones aritméticas no equivale a contar operaciones sobre bits de enteros crecientes.
- Coeficientes binomiales y mochila muestran tablas con celdas innecesarias y reducción por filas. La mochila 0/1 no permite reutilizar un objeto; su cota depende de capacidad entera y representación.
- Distancia de edición depende de operaciones permitidas y sus costes. Copiar caracteres iguales de forma voraz exige las desigualdades del modelo. Guardar el coste junto al resultado evita recalcularlo al comparar candidatos.
- LCS y distancia de edición no son intercambiables bajo cualquier coste de sustitución. Simetría requiere costes compatibles de inserción y borrado. Observación propia: calcular repetidamente la longitud de listas candidatas puede invalidar una cota basada solo en número de celdas.
- El autobús modela pasajeros ordenados y coste de caminar a paradas: es una especificación concreta. Contar estados, transiciones y cálculo de costes por tramo antes de afirmar complejidad.

## 14. Parentización óptima — PDF 352–381

- Asociatividad preserva el valor al cambiar paréntesis; no permite permutar operandos. La subestructura óptima usa pesos independientes de parentización y costes monótonos.
- La tabla por intervalos tiene coste cúbico y espacio cuadrático bajo operaciones auxiliares constantes. Reducir a coste cuadrático exige restricciones demostradas sobre posiciones de raíces óptimas y desempates consistentes; no es una optimización general de toda programación dinámica.
- La desigualdad cuadrangular ofrece condiciones suficientes. No cumplirlas no prueba que ninguna optimización sea posible.
- Ejemplos de aritmética usan modelos simplificados de dígitos. Coste de construir/materializar la respuesta puede superar el coste de elegir su forma. La variante paralela del ejercicio cambia suma por máximo de dependencias, sin modelar un runtime real.
- Garsia–Wachs construye y reconstruye conservando profundidades; no toda secuencia de profundidades es válida. La mejora depende de invariantes de orden y representación de búsqueda. El libro remite a otras fuentes para su demostración completa: no atribuir una prueba leída aquí.
- Pendiente propio: cotejar visualmente los árboles y costes de respuesta 14.14 (PDF 381) antes de reutilizarlos.

## 15. Búsqueda exhaustiva — PDF 386–421

- El orden de generación afecta al tiempo hasta la primera solución. Encontrar una solución no cuesta necesariamente una fracción uniforme de enumerarlas todas. Compartición y representación de estados cambian el trabajo real.
- N-reinas con vectores de bits tiene límites de anchura y semántica de operaciones; no extrapolar Word16 a tamaños arbitrarios.
- Podar expresiones por monotonicidad presupone dígitos positivos y operadores concretos; cero, fracciones o exponentes invalidan argumentos. El valor actual solo no representa toda la información necesaria para extender.
- Marcar visitados por camino y marcarlos globalmente responde a problemas diferentes: todos los caminos simples frente a alcanzar estados. BFS minimiza número de movimientos cuando tienen igual coste, no costes arbitrarios.
- Una cola eficiente no elimina el tamaño de la frontera BFS. Las cotas de memoria dependen de ramificación, profundidad y representación.
- Planificar movimientos preparatorios puede requerir volver a movimientos ordinarios para conservar completitud. Los planes deben reevaluarse en el estado actual; una preparación puede invalidar otra. El plan más rápido de encontrar no tiene necesariamente menos movimientos.
- Las tablas de tiempos históricas del capítulo son ejemplos experimentales, no umbrales de calidad para Elixir.

## 16. Búsqueda heurística — PDF 422–448; índice PDF 449–454

- Priorizar g+h combina coste recorrido y estimación restante; usar solamente h pierde optimalidad incluso si h es admisible.
- Admisibilidad significa no sobreestimar el coste restante. Consistencia exige h(u) <= coste(u,v)+h(v), con cero en objetivos. Con heurísticas admisibles inconsistentes puede ser necesario reabrir estados; un conjunto de visitados definitivo no basta.
- La terminación de búsqueda en árbol depende de hipótesis sobre grafo y costes; el ejercicio con ciclo de coste cero demuestra un bloqueo. Las afirmaciones sobre grafos finitos no se extienden a infinitos.
- Colas de prioridad con clave de estado conservan el mejor candidato por clave; la validez de esa clave y de la comparación es parte del argumento.
- Navegación: movimientos ortogonales, diagonales y visibilidad arbitraria definen grafos y óptimos distintos. Suavizar un camino puede reducir distancia sin obtener el óptimo de visibilidad completa. Contar comprobaciones de obstáculos y construcción del grafo, no solo extracciones de cola.
- En el rompecabezas, las heurísticas de piezas fuera de lugar y Manhattan excluyen el hueco para conservar consistencia. La paridad permite rechazar estados imposibles bajo las reglas del tablero.
- El coste de calcular una heurística también cuenta. Las mediciones GHCi publicadas no predicen tiempos ni escalabilidad de otra implementación.

## Estado final de lectura

Leído todo el texto extraído, bloques 001–034, incluidos prefacio, capítulos, ejercicios, respuestas, referencias e índice. Esto no certifica cada fórmula, figura, demostración ni programa: las extracciones dañadas y observaciones pendientes requieren cotejo antes de convertirse en reglas ejecutables.
