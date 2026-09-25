# Estado de lectura y decisiones del usuario

## Instrucciones vigentes

Antes de formular el diseño del agente, leer los cuatro libros del root y procesarlos en memoria consultable y extensible. No confundir extracción, lectura, comprensión y validación de una regla.

- Puntaje estricto, sin asignación de números por el LLM.
- Agente revisor: devuelve instrucciones detalladas; no modifica el código evaluado.
- Escalabilidad incluye concurrencia.
- El usuario preguntó qué significan pruebas y cargas de aceptación. Se explicó: comportamiento esperado y escenarios de entradas/concurrencia. No se ha acordado un contrato ni un dominio de negocio.
- La unidad de entrada (fragmento/proyecto) aún no está definida; no bloquear la lectura por ello.

## Fuentes

`sources/manifest.json` identifica archivos originales por SHA-256. `sources/*.json` conserva el texto extraído por página física PDF o entrada del spine EPUB.

La extracción inicial con pypdf tuvo símbolos matemáticos defectuosos. PyMuPDF tampoco resuelve todos los símbolos de la tesis. Los originales son la autoridad; no usar fórmulas extraídas como código ejecutable ni prueba formal sin verificación visual.

## Progreso comprobable

- `pfds`: leído el texto de capítulos 1–9, apéndice A y bibliografía/índice; releídos PDF 41–42, 66–68 y 150–154 para cerrar salidas truncadas. Notas completas por capítulos.
- `proper`: leído todo el texto extraído, bloques 001–027; unidades 32–33 releídas. Notas de capítulos 1–11 y apéndices.
- `pearls`: leído todo el texto extraído, bloques 001–021, incluidos 30 capítulos, referencias e índice.
- `haskell`: leído todo el texto extraído, bloques 001–034, incluidos prefacio, 16 capítulos, ejercicios, respuestas, referencias e índice.

Los localizadores permanentes son las unidades de `sources/*.json`; los bloques temporales solo ayudaron a repartir la lectura. Las notas están en `notes/{id}.md`. La lectura no certifica cada demostración, fórmula, figura ni ejecución de código. El estado del manifiesto describe lectura del texto extraído, no validación formal.

## Observaciones a preservar

- La fuente `pfds` es la tesis CMU-CS-96-177, septiembre de 1996, no asumir la edición comercial.
- Las afirmaciones históricas de rendimiento relativo de estructuras/herramientas no son evaluaciones actuales de Elixir.
- En PropEr capítulo 3: modelado simple independiente; generalización de ejemplos; invariantes; propiedades de ida y vuelta. Ida y vuelta no demuestra conformidad del formato. La pertenencia y longitud tampoco comprueban por sí solas multiplicidades de duplicados (observación propia a verificar).
- En PropEr capítulo 4: variedad de entradas no implica variedad de operaciones, medir distribución; controlar tamaño; transformación frente a rechazo; probabilidades; recursión diferida y con límites; llamadas simbólicas como evidencia legible.
- Posible errata del ejemplo `min(100, Size)*35`: el texto dice mínimo sin techo; esa expresión hace lo contrario. Marcar como observación propia, no enseñar sin validación.
- En PropEr capítulo 5: separar transformaciones puras y efectos; combinar propiedades con ejemplos de referencia; el modelo puede omitir casos por su representación. Enumeración exhaustiva de un dominio finito puede ser más apropiada que muestreo aleatorio. El ejemplo CSV tiene limitaciones explícitas de columna única y encabezados duplicados.

## Procesamiento y siguiente etapa

Notas por capítulo, manifiesto con hashes, mapa de conceptos y herramienta de ingestión/consulta disponibles; ver `README.md`. Los libros nuevos entran como `unread`. Persistir contenido permite recuperarlo, no convierte por sí mismo la extracción en aprendizaje validado.

Antes de convertir una afirmación en regla del evaluador: cotejar su fuente y premisas, resolver símbolos/posibles erratas, comprobar su aplicabilidad al modelo de ejecución y definir evidencia ejecutable. No se ha implementado el evaluador ni reescrito el master prompt. Las notas contienen pendientes explícitos; ningún script debe tratarlos como reglas aprobadas.
