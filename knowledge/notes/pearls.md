# Pearls of Functional Algorithm Design — Richard Bird, 2010

Fuente: `pearls`, SHA-256 en `../sources/manifest.json`. Localizadores PDF físicos; página impresa = PDF − 14 en el cuerpo. Notas de lectura, no reglas ejecutables. Texto y símbolos extraídos pueden tener defectos; consultar original antes de formalizar una ecuación.

## Método (PDF 11–13)

El objeto de estudio es la derivación desde una especificación clara hasta una implementación eficiente. El resultado optimizado puede ser menos legible; la explicación de por qué funciona forma parte esencial del conocimiento. Una implementación breve no demuestra calidad global.

## 1. Menor natural libre (PDF 15–20)

- Acotar búsqueda por número de entradas; comparar checklist con divide y vencerás. Recurrencia de una sola rama de tamaño máximo mitad más partición lineal permite costo lineal.
- La solución por partición necesita naturales distintos; checklist admite duplicados. No trasladar premisas entre implementaciones.
- Cachear tamaño evita recomputación, pero exige mantener coherencia. El costo de arrays mutables, inmutables e indexación depende de la representación.
- Las mejoras porcentuales del experimento son históricas y locales.

## 2. Surpassers (PDF 21–25)

- Un máximo aislado pierde información necesaria para combinar partes. Generalizar a tabla ordenada de cuentas permite combinación lineal y recurrencia de dos mitades.
- Contar elementos mayores a la derecha distingue estrictamente desigualdad de igualdad. Tamaño residual se actualiza incrementalmente.
- Límite inferior por reducción a ordenación depende del modelo basado en comparaciones. La implementación en listas consume espacio diferente de una solución in-place.

## 3. Saddleback search (PDF 26–34)

- Monotonía estricta de ambas entradas permite descartar filas/columnas. Buscar límites reduce dominio; buscar por fila o columna según forma del rectángulo cambia recurrencias.
- Contar evaluaciones de una función costosa no equivale a contar todo el trabajo. La evidencia experimental usa funciones concretas y GHCi.
- Conservar hipótesis de extremos de búsqueda, casos vacíos y dimensiones. No copiar expresiones simplificadas con log(n/m) al caso m=n sin revisar término constante: límite expresado antes usa log(1+n/m).

## 4. Selección (PDF 35–40)

- Posición k comienza en cero. Con listas y evaluación perezosa, fusionar y tomar solo el prefijo necesario evita materializar todo.
- División binaria no garantiza tiempo logarítmico con listas: split/indexación cuestan recorridos. Arrays con acceso constante o árboles adecuados cambian el costo.
- Conjuntos disjuntos y multiconjuntos tienen contratos distintos. Equivalencia de representación requiere conservar intervalos y desplazamientos; verificar código de índices antes de reutilizar.

## 5. Sumas por pares (PDF 41–46)

- Monotonía sola no proporciona las leyes de grupo abeliano necesarias para sustituir comparaciones por diferencias.
- Reducir comparaciones de valores a O(n²) deja O(n² log n) trabajo total en el algoritmo presentado. Optimizar un contador aislado puede empeorar tiempo real.
- El estado de problemas abiertos corresponde a 2010; no presentarlo como situación actual sin verificar.

## 6. Búsqueda exhaustiva de expresiones (PDF 47–54)

- Poda correcta exige que toda solución buena sea admisible y que descartar un parcial no elimine una extensión buena. Fusionar generación y validación bajo esas leyes.
- Guardar valor junto a candidato evita recalcular; a veces hace falta un resumen más rico (factor, término, potencia decimal), no solo el total.
- No generalizar monotonía de + y × a negativos, ceros u operadores nuevos sin comprobar. Costos y cifras experimentales dependen del dominio.
- Pedir la primera solución evita trabajo restante bajo evaluación perezosa; no suponerlo en una traducción estricta.

## 7. Árbol de costo mínimo (PDF 55–63)

- Minimizar altura/costo local no basta: dos árboles de igual costo reaccionan distinto al insertar. Reforzar criterio con costos de la espina permite demostrar monotonía.
- Diferenciar igualdad de resultados y refinamiento de una especificación que acepta varios óptimos; desempate determina resultado concreto.
- Smart constructors mantienen costo cacheado. Prueba de total de llamadas, no ausencia de recursión anidada, justifica linealidad.

## 8. Greedy y upravels (PDF 64–69)

- La salida es bolsa de secuencias: orden entre secuencias irrelevante, multiplicidades relevantes.
- Menor longitud parcial no implica mejor extensión. Orden parcial reforzado y monotonía justifican selección voraz.
- Mínimo y minimal no son equivalentes. Debe existir resultado menor o igual a todos para aplicar el razonamiento usado.
- Búsqueda lineal sobre listas frente a estructuras con búsqueda logarítmica determina costo final.

## 9. Celebridades (PDF 70–77)

- Encontrar suponiendo existencia y verificar existencia son problemas distintos; aquí esa precondición cambia el costo asintótico.
- La solución candidata lineal no certifica por sí sola su validez ante entradas fuera de contrato. No transferir límites de clique general al problema más restringido.
- Una transformación puede ser refinamiento condicionado, no igualdad total. Retener dominio y relación exacta.

## 10. Eliminar duplicados (PDF 78–86)

- Conservar primeras apariciones y obtener subsecuencia lexicográfica mínima son especificaciones distintas. Eq frente a Ord cambia operaciones disponibles.
- Sustituir listas por sets no elimina automáticamente recorridos repetidos: el ejemplo aún filtra el resto de entrada y queda cuadrático.
- Un acumulador de elementos ya emitidos permite diferir borrados y contabilizar cada elemento globalmente. Mantener orden y multiplicidad requeridos.

## 11. Máximo de no-segmentos (PDF 87–92)

- Marcaciones reconocibles por autómata permiten mantener mejores valores por estado y fusionar búsqueda exponencial en una pasada.
- El número de estados es parámetro de costo. Un resumen constante solo es constante si ese parámetro está fijado.
- El problema necesita al menos tres elementos; −infinito de la especificación no es un entero ordinario. Inicialización y dominio deben explicitarse.

## 12. Ranking de sufijos (PDF 93–104)

- Comparar sufijos no cuesta lo mismo que comparar elementos; prefijos repetidos generan peores casos.
- Refinar ranks duplicando longitud de prefijo evita comparar texto completo repetidamente. Terminar al obtener ranks distintos.
- O(n log n) requiere las condiciones de particionado y acceso indicadas; el quicksort con primer pivote que aparece como ejemplo tiene peor caso cuadrático.
- Mantener grupos en refinamiento reduce trabajo; igualdad hasta permutación dentro de grupo es suficiente solo porque el consumidor no observa ese orden.
- Experimentos muestran que casos repetitivos cambian radicalmente el rendimiento; no extrapolar de texto corriente a todos los datos.

## 13. Burrows–Wheeler (PDF 105–115)

- Transformación debe ser invertible con columna y posición. Agrupar símbolos para compresión no basta si pierde recuperación del original.
- Estabilidad de ordenación es condición esencial de la reconstrucción. Reutilizar una permutación elimina ordenamientos redundantes.
- Seleccionar la fila sin reconstruir matriz completa reduce trabajo y almacenamiento intermedio; reemplazar indexación de listas por arrays es parte de la prueba de costo.
- Entrada no vacía y marcador de fin ausente del dominio son premisas. Verificar construcción exacta del marcador y algoritmo antes de traducción.

## 14. Último sufijo (PDF 116–125)

- Especificación corta puede requerir derivación compleja. Resumen suficiente incluye border, longitudes y sufijos, no solo resultado parcial.
- Aun suponiendo concatenación constante puede persistir recómputo cuadrático: tratar ambos problemas por separado.
- Periodicidad permite reducir recursión a menos de la mitad; referencias a sufijos existentes evitan construirlos repetidamente.

## 15. Prefijos comunes / Z (PDF 126–130)

- Reutilizar prefijos conocidos elimina comparaciones; contar avances de frontera limita total de coincidencias, y hay a lo sumo un fallo por paso.
- El análisis inicialmente asume constantes drop, snoc e indexación, pero debe refinar representación para satisfacerlo. Dos colas y array sustituyen esos accesos.
- Relación con Okasaki: garantías de cola son parte de la justificación, no propiedad de cualquier implementación llamada Queue.

## 16. Boyer–Moore (PDF 131–140)

- Scan lemma reutiliza plegados de prefijos, con costo de operación acumuladora explícito.
- Saltos necesitan demostrar que no omiten coincidencias; preparación de tabla también entra en costo total.
- El programa final usa mejora de Galil; no atribuir su garantía a todas las variantes BM. Patrón no vacío, igualdad de costo constante y estructuras apropiadas son premisas.

## 17. Morris–Pratt / KMP (PDF 141–149)

- Generalizar booleano de coincidencia a prefijo reconocido y resto por reconocer da estado suficiente.
- Abstracción inversa por la izquierda justifica cambio de representación sin exigir biyección. Eliminar datos nunca observados es optimización posterior.
- Enlaces de fallo compartidos evitan volver a calcular prefijos. Representación cíclica perezosa requiere adaptación explícita al entorno estricto.
- Separar Morris–Pratt presentado y refinamiento KMP adicional. Constante amortizada por símbolo no significa latencia constante individual.

## 18. Rush Hour (PDF 150–160)

- BFS encuentra camino mínimo bajo el costo por movimiento definido; DFS/planificación encuentran alguna solución y pueden devolver más movimientos.
- Cambiar acumuladores reduce costo de gestionar frontera, no su tamaño máximo. Ahorrar tiempo no demuestra ahorro espacial.
- Planes voraces solos pueden perder completitud; ramas de respaldo mantienen búsqueda de soluciones cuando fallan planes.
- Movimientos preparatorios cambian estado y pueden invalidar pasos planeados: recalcular expansión. Un movimiento del ejemplo es una celda, no desplazamiento arbitrario.
- Benchmarks sobre cuatro tableros no demuestran dominio universal de planificación; registrar tiempo y calidad de solución por separado.

## 19. Sudoku (PDF 161–169)

- Especificación por expansión de opciones y validación; poda debe preservar conjunto de soluciones. Filas, columnas y cajas se expresan como transformaciones de matriz con leyes sobre dimensiones válidas.
- Expandir una sola celda y elegir menor número de opciones reduce ramificación. Diferenciar completo, contradictorio y sin opciones; minimum sobre conjunto vacío no es válido.
- Dominio fijo de nueve elementos puede favorecer algoritmo asintóticamente peor con menor constante. La complejidad debe usar el tamaño que realmente varía.
- Varias soluciones válidas no tienen orden único; comparación de conjuntos puede corresponder al contrato, comparación de listas quizá no.

## 20. Countdown (PDF 170–181)

- Fuente admite duplicados y usar solo algunos números. Toda operación intermedia debe ser legal: positivos y divisiones exactas.
- Eliminar simetrías y formas redundantes preserva algún resultado de valor óptimo, no todas las expresiones posibles ni el mismo desempate.
- Guardar expresión y valor evita recálculo. Memoización requiere orden de dependencias correcto; lookup parcial es justificado solo por ese invariante.
- Memoizar todas las expresiones aumenta retención y garbage collection. Memoizar esqueletos cambia el compromiso; datos compilados de mayor tamaño cambian el ranking observado en una prueba pequeña interpretada.
- El costo de detectar candidatos dominados puede superar ahorro posterior. Medir optimización completa, incluyendo preparación, búsqueda y GC.

## 21. Hylomorfismos y nexos (PDF 182–193)

- Unfold construye y fold consume; eliminar intermediario suele ahorrar asignaciones, pero materializar un grafo compartido puede evitar subproblemas repetidos.
- Árbol de llamadas y grafo de subproblemas no tienen el mismo tamaño: segmentos y subsecuencias presentan grados distintos de solapamiento.
- Compartición debe preservarse durante anotación; igual contenido no implica identidad compartida automáticamente.
- Tabulación por capas puede guardar solo valores cuando el consumidor lo permite; otros consumidores necesitan estructura y caminos. Representación de capa depende del patrón de dependencias.
- Recorrer un grafo sin identificar nodos puede repetirlos; un árbol abarcador estructural evita esa necesidad en el ejemplo particular.

## 22. Determinantes (PDF 194–201)

- Número de operaciones aritméticas y costo en bits son medidas diferentes. Condensación puede tener pocas divisiones y producir enteros enormes.
- Gaussian elimination racional, condensación con división exacta y método sin división exigen dominios y leyes diferentes. Pivote cero, singularidad y signo por reordenamiento son casos indispensables.
- Intercalar división exacta limita crecimiento intermedio; no reemplazar aritmética exacta por flotantes sin reconsiderar corrección.
- El método sin división se presenta sin prueba completa local; registrar procedencia y no afirmar validación propia.

## 23. Envolvente convexa (PDF 202–211)

- Coordenadas racionales y determinantes exactos evitan ciertos errores geométricos; dimensiones y degeneración son parámetros explícitos.
- Esta especificación declara vacío el hull degenerado de dimensión inferior. No confundirla con otras definiciones habituales del problema.
- Pasar de simplexes a caras exteriores evita reconstruir información; validez de orientación y descarte de caras internas requiere invariantes.
- QuickCheck detecta fallo por and de lista vacía: la representación por caras acepta indebidamente cualquier punto cuando no hay caras. Corregir caso vacío, no filtrar silenciosamente esas entradas del generador.
- Relación con PropEr: comparar implementación con modelo independiente detecta errores en optimizaciones; muestras aprobadas no son prueba universal.

## 24. Codificación aritmética racional (PDF 212–221)

- Modelo particiona intervalo semiabierto; codificador y decodificador deben compartir modelo inicial y adaptación.
- Contrato inicial de decodificación solo garantiza prefijo. Longitud transmitida o EOF resuelve terminación; costo del protocolo debe contabilizarse.
- Streaming requiere ley que garantice que salida ya emitible permanece válida tras consumir entrada. No mover emisiones antes de cálculos sin demostrarla.
- Precisión racional arbitraria conserva exactitud pero puede crecer en tiempo/espacio. El apéndice demuestra streaming mediante leyes de unfold y listas finitas.

## 25. Codificación entera (PDF 222–234)

- Redondear intervalos puede colapsarlos y producir salida infinita inválida. La operación entera pierde asociatividad: ya no aplica directamente la equivalencia de streaming racional.
- Renormalización y expansión mantienen ancho suficiente; precisión de modelo y de estado tienen restricciones distintas. Intermedios también deben caber en la representación.
- Contador de bits pendientes puede crecer: que intervalos estén acotados no acota todo el estado. El autor reconoce el caso de desbordamiento; no convertir improbabilidad en garantía.
- Decodificación incremental elimina cómputo repetido de fracción y conserva información necesaria en ventana de bits. Probar inversa bajo contrato de prefijo y terminación acordada.
- Observación propia: revisar visualmente desigualdad del ancho en PDF 224 antes de formalizar; el texto extraído parece invertirla. No copiar fórmulas sin esa comprobación.

## 26. Schorr–Waite (PDF 235–244)

- Marcar alcanzables y restaurar grafo son ambas partes del contrato. Resultado de marcas correcto no basta si altera enlaces finales.
- Eliminar pila externa requiere invariantes sobre nodos marcados, no repetidos, encadenamiento y reemplazo seguro ante aliasing.
- Ahorro de pila usa campos del grafo y bits auxiliares; no inferir espacio constante total ni mismo costo para grafos funcionales persistentes.
- Pasos de transformación motivan cada representación; interpretación imperativa de actualizaciones requiere sus garantías específicas.

## 27. Inserción ordenada (PDF 245–255)

- Métrica objetivo cuenta elementos movidos, no distancia ni todo el tiempo de ejecución. La implementación ilustrativa por listas no hereda automáticamente esa cota temporal.
- Redistribución por densidad, balance de intervalos y fases respaldan cota amortizada. Capacidad adicional cambia costo.
- Exactitud de test de densidad puede exigir enteros grandes; su costo no desaparece por evitar flotantes.
- Conjeturas y límites inferiores para algoritmos smooth son históricos y restringidos; no presentarlos como límite universal probado.

## 28. Generación con demora constante (PDF 256–264)

- Loopless: prólogo lineal y cada transición posterior constante. Emitir transición no equivale a imprimir patrón completo.
- Evaluación perezosa puede diferir prólogo y romper demora prometida; controlar cuándo se fuerza el trabajo.
- Eliminar listas vacías de frontera evita una pausa arbitraria entre elementos. Representar concatenaciones por bosque y colas permite avances acotados.
- Demora individual menor puede aumentar tiempo total; objetivo de latencia y throughput no son intercambiables.

## 29. Johnson–Trotter (PDF 265–271)

- Transiciones de permutaciones mediante intercambio adyacente; dirección y paridad determinan desplazamientos.
- Componer dos generadores loopless no basta: el prólogo conjunto inicialmente resulta cuadrático.
- Estado explícito de generación pendiente distribuye trabajo sin construir listas completas; hay que probar tanto preparación lineal como paso constante.

## 30. Spider spinning (PDF 272–288)

- Gray path cambia un bit por paso. No todos los sistemas de restricciones admiten ese camino; la hipótesis de grafo totalmente acíclico es más fuerte que DAG dirigido.
- Estado inicial no necesariamente es todo cero; debe ser compatible con secuencia de transiciones y restricciones.
- Resúmenes de paridad evitan generar secuencias solo para medir su longitud. Tupling de recorridos directo/inverso y colas evitan reversas/copias repetidas.
- Mantener compatibilidad entre estados iniciales/finales al componer subproblemas es parte de corrección. Producir transiciones válidas aisladas no demuestra recorrido completo.
- Costear generación de seed además de transiciones cuando la tarea lo requiera; el prólogo del generador no es automáticamente todo el costo del consumidor.

## Estado

Leídos bloques 001–021: prefacio, capítulos 1–30, apéndices internos, bibliografía e índice del texto extraído. No se validaron íntegramente pruebas, figuras ni código ejecutable. Estas notas conservan premisas, límites y relaciones; no constituyen una escala de calidad ni reglas aprobadas.
