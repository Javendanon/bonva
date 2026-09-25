# Purely Functional Data Structures — registro de lectura

Fuente: `pfds`, Chris Okasaki, tesis CMU-CS-96-177, septiembre de 1996.
Las páginas PDF son posiciones físicas (base 1); la página impresa es PDF − 12 en el cuerpo.
Estado: lectura en curso. Las fórmulas y figuras tienen problemas de extracción; volver al PDF antes de usarlas como evidencia formal.

## Capítulos 1–2 (PDF 13–24): contexto y evaluación

- Distingue abstracción, implementación, objeto/versión e identidad persistente. Una regla sobre una abstracción no basta para establecer el costo de una implementación.
- Persistencia significa que sobreviven versiones previas; no implica que los costos de una estructura efímera se mantengan.
- La evaluación diferida usada en esta tesis incluye memoización. Una suspensión sin almacenamiento del resultado no cumple esa premisa.
- Distingue trabajo incremental de trabajo monolítico: suspender una operación completa no la vuelve incremental. `drop` y `reverse` pueden necesitar trabajo previo completo antes de producir salida.
- La posición de una suspensión respecto de un patrón cambia cuándo se fuerza el argumento, aunque el código parezca equivalente.
- Las suspensiones tienen sobrecosto. No se recomienda introducir pereza indiscriminadamente.

## Capítulo 3 (PDF 25–50): amortización y persistencia

- Un límite amortizado acota el costo acumulado de secuencias; no acota cada operación ni es un promedio de muestras aleatorias.
- Métodos tradicionales: créditos (banquero) y potencial (físico). Ambos dependen de las condiciones del historial de operaciones de la estructura.
- Cola de dos listas: invertir el segmento posterior al agotarse el frente permite buen costo acumulado en uso lineal. Repetir `tail` sobre la misma versión anterior a la inversión repite el trabajo y rompe la conclusión amortizada ingenua (sección 3.2, PDF 31).
- Un historial de versiones de datos no es el historial de evaluaciones del agente. Son conceptos distintos.
- La memoización permite compartir trabajo entre futuros lógicos; crear suspensiones distintas no comparte automáticamente sus resultados.
- Separa costo no compartido, compartido, completo, realizado y no realizado. Las obligaciones de deuda deben satisfacerse antes de forzar el trabajo.
- Invariantes sobre longitudes y momento de rotación forman parte de la demostración, no son detalles opcionales de implementación.
- Las variaciones aparentemente más perezosas de las colas del físico pueden romper los límites por forzar demasiado pronto o acumular eliminaciones monolíticas (PDF 44).
- Mergesort incremental de colecciones compartidas distribuye el costo entre inserciones y ordenación final; no interpretar el costo de la operación final como costo de construir y ordenar desde cero.

## Capítulo 4 (PDF 51–60): eliminar amortización

- Tiempo real, paralelismo sincronizado e interacción motivan límites por operación: un buen costo total puede ocultar pausas individuales.
- Primero reducir el costo intrínseco de cada suspensión; después programar dependencias para evitar cascadas de forzado.
- Una secuencia de pasos incrementales puede seguir ocasionando una cascada larga. Incrementalidad por sí sola no prueba un límite por operación.
- Las colas de tiempo real utilizan una agenda cuyo avance mantiene invariantes de disponibilidad. Las garantías requieren las premisas completas del algoritmo.
- La tesis advierte sobre el sobrecosto de memoizar valores que nunca se reutilizan; sus comparaciones prácticas son históricas.

## Capítulo 5 (PDF 61–72): reconstrucción

- Reconstrucción por lotes requiere suficiente separación entre reconstrucciones y actualizaciones que no degraden demasiado las operaciones entre ellas.
- Reconstrucción global reparte trabajo y conserva una copia operativa; debe incorporar las actualizaciones ocurridas durante la reconstrucción antes del reemplazo.
- Reconstrucción perezosa difiere la ejecución tras distribuir su costo; no equivale a ejecutarla en paralelo.
- Deques mantienen equilibrio entre extremos y requieren invariantes distintos de las colas FIFO. Los parámetros admisibles de la versión de tiempo real están restringidos por la demostración.
- Las fórmulas y código de este capítulo requieren inspección visual antes de transcribirlos; el texto extraído omite símbolos.

## Capítulo 6 (PDF 73–96): representaciones numéricas

- Analogía: insertar/eliminar/combinar contenedores con incrementar/decrementar/sumar representaciones numéricas.
- La organización de árboles debe soportar enlace/desenlace y el orden exigido por la abstracción, no solo tamaños adecuados.
- En representaciones binarias, cadenas de acarreo/préstamo explican costos que el número de ramas sintácticas no revela.
- Listas de acceso aleatorio: búsqueda por árbol y dentro del árbol; actualizar persistentemente reconstruye el camino y comparte el resto.
- Heaps combinables necesitan comparadores compatibles. Elegir un comparador distinto por objeto hace ambigua la combinación (PDF 82).
- Representaciones segmentadas y skew reducen ciertas cascadas, con compromisos de complejidad de implementación. No hay una estructura óptima para todas las operaciones.
- Las listas skew combinan acceso de lista y acceso aleatorio; esta es una condición de uso, no una recomendación universal.
- Conservar metadatos redundantes puede simplificar código; eliminarlos no es automáticamente una mejora.

## Capítulo 7, secciones 7.1–7.2.1 (PDF 97–110)

- Bootstrapping por descomposición extiende representaciones acotadas con componentes recursivos; por abstracción obtiene operaciones eficientes a partir de una estructura primitiva.
- Recursión no uniforme puede expresar invariantes que desaparecen al convertirla a una representación más permisiva. En ese caso deben preservarse por otras vías.
- Concatenaciones asociadas a la izquierda pueden repetir trabajo, pero el crecimiento geométrico de segmentos puede evitar el costo cuadrático global (PDF 99–100). Un detector puramente sintáctico no puede ignorar esto.
- La representación debe conservar elementos y caso vacío: las ecuaciones de tipos sirven para descubrir diseños incompletos.
- Listas concatenables: enlazar hijos en el orden adecuado y suspender con memoización permite compartir trabajo bajo persistencia; la estructura primitiva debe cumplir las garantías requeridas.

## Aplicación al proyecto (interpretación, no afirmación del autor)

No trasladar ninguna garantía a Elixir sin comprobar representación, semántica de evaluación, compartición, operaciones y alcance de la prueba. Estos conceptos todavía no son reglas ejecutables ni puntajes. La fuente aporta fundamentos de costos e invariantes, no una escala universal de calidad ni pesos de evaluación.


## Sección 7.2.2 — PDF 111–116

- Bootstrapping de heaps mantiene mínimo en raíz y usa una estructura primitiva para subheaps. Las garantías de unión/inserción dependen de las operaciones primitivas y del comparador.
- Los módulos recursivos y restricciones de compartición expresan dependencias que deben conservarse al cambiar de lenguaje.

## Capítulo 8 — PDF 117–138

- Recursive slowdown distribuye propagaciones entre niveles; dígitos redundantes e invariantes de color limitan cascadas. No basta con escribir recursión para obtener esas garantías.
- Incrementar y decrementar pueden tener análisis amortizados separados que no admiten mezcla: alternar en una frontera produce trabajo repetido. La prueba debe cubrir la secuencia de operaciones permitida.
- Las representaciones con dígitos 1, 2, 3 y recursión no uniforme preservan invariantes compartidos. Las deques concatenables ilustran el compromiso entre una representación sencilla y cotas más fuertes.
- Las operaciones auxiliares, como reemplazar una cabeza, tienen precondiciones. Las pruebas y constructores deben preservarlas.

## Capítulo 9 — PDF 139–142

- Abstracción y pattern matching exponen un compromiso de interfaz. Una representación más compleja puede tener mejores cotas y peores constantes.
- El autor pide mediciones empíricas y estudia expresividad de lenguajes. Sus comparaciones y problemas abiertos pertenecen a 1996: no son resultados actuales ni benchmarks de BEAM.

## Apéndice A — PDF 143–148

- Formaliza evaluación y suspensiones de la notación usada. Patrones, efectos y excepciones importan al forzar resultados.
- En la semántica elegida, las excepciones de suspensiones fallidas no quedan memoizadas; volver a forzar puede repetir efectos. Hay que distinguir esa decisión de otras implementaciones de pereza.

## Relecturas y cierre

- PDF 41–42: potencial agregado no identifica qué suspensión tiene deuda; puede exigir pagar toda la deuda antes de forzar. Las consultas también pueden forzar trabajo aunque no produzcan un objeto actualizado.
- PDF 66–68: balance simétrico de deque admite singleton en cualquiera de los lados; invertir intercambia representaciones de frente/fondo. Las cotas presuponen reconstrucción y suspensiones de este modelo.
- Bibliografía e índice PDF 149–162 leídos; no se han leído automáticamente los trabajos citados.
- Se completaron los fragmentos antes truncados. Texto leído no significa validación visual de símbolos dañados ni ejecución de los programas.

### Cotejo visual puntual

PDF 66 (página impresa 54), cotejada directamente con render del original: el parámetro cumple `c > 1`; el invariante es `|F| <= c|R| + 1` y `|R| <= c|F| + 1`. La cota indicada es O(1) **amortizado**, no peor caso por operación. Este cotejo resuelve esos símbolos de esa página; no valida automáticamente el resto del capítulo.
