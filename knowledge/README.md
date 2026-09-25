# bonva — memoria de los libros

## Entrada para futuras sesiones

1. Leer `READING_STATUS.md`: decisiones del usuario y límites de lo realizado.
2. Consultar `CONCEPTS.md` para encontrar temas y relaciones entre fuentes.
3. Leer `notes/{id}.md` y recuperar las unidades originales relevantes con `show`.
4. Cotejar el PDF/EPUB original antes de usar una fórmula dañada o resolver una observación pendiente.

Esta memoria vive en archivos. No depende de recordar el chat ni modifica los parámetros de un modelo. Las notas son síntesis e interpretaciones con referencias; no son reglas aprobadas ni puntajes. Los libros citados dentro de estas fuentes no se consideran leídos.

## Consulta reproducible

Desde el root, con Python 3:

```sh
python3 scripts/knowledge.py verify
python3 scripts/knowledge.py list
python3 scripts/knowledge.py search 'amortized' --book pfds --limit 5
python3 scripts/knowledge.py show pfds 66
python3 -m unittest discover -s tests -v
```

El CLI asigna un manejador a cada subcomando (Command). La extracción selecciona
un extractor PDF o EPUB por extensión (Strategy), manteniendo la consulta
independiente de las dependencias de ingestión.

La búsqueda encuentra una subcadena literal tras normalizar Unicode NFKC, mayúsculas y espacios. Ordena por identificador de libro y número de unidad. No usa embeddings, historial ni puntuaciones de relevancia. La misma entrada y el mismo corpus producen el mismo JSON. No encontrar una frase no demuestra que el concepto esté ausente: consultar el mapa bilingüe y las notas.

Los números de PDF son páginas físicas empezando en 1, no necesariamente la numeración impresa. En EPUB son entradas del spine; `href` identifica el documento. `show` devuelve el texto extraído completo de la unidad. Los extractos de `search` están normalizados: no usarlos como citas textuales sin cotejar `show` y el original.

`verify` comprueba hashes de originales y extracciones, cantidades de unidades/caracteres, unidades vacías y secuencia de localizadores. No verifica la verdad de las notas ni la fidelidad de cada símbolo matemático. Ejecutarlo antes de usar el corpus tras modificaciones.

## Incorporar otro libro

Guardar el archivo dentro del proyecto y usar un identificador nuevo:

```sh
python3 scripts/knowledge.py ingest nuevo_libro 'libros/nuevo.epub' --title 'Título y edición'
```

EPUB usa la biblioteca estándar de Python; PDF requiere PyMuPDF. La extracción PDF inicial empleó PyMuPDF 1.28.2. Las consultas no requieren esa dependencia. La versión exacta del extractor EPUB inicial no quedó registrada; el manifiesto declara esa limitación y conserva sus bytes. Las nuevas ingestiones registran extractor y versión.

El comando guarda texto, hash, localizadores y estado `unread`. Rechaza identificadores existentes para conservar la procedencia de lecturas anteriores. No sobrescribe notas ni marca automáticamente el libro como aprendido. Para otra edición usar otro ID.

Después de ingerir:

1. Leer las unidades en bloques que no se trunquen; registrar explícitamente el alcance.
2. Crear notas con capítulos, premisas, límites, contraejemplos y páginas. Separar afirmaciones de la fuente de interpretaciones propias.
3. Añadir relaciones a `CONCEPTS.md`, sin borrar discrepancias entre fuentes.
4. Actualizar manualmente el estado de lectura y la referencia a notas en el manifiesto; conservar el estado de validación separado.
5. Ejecutar `verify`. Antes de convertir conocimiento en una regla ejecutable, validar sus premisas y traducción al entorno objetivo.

## Alcance actual

Cuatro libros procesados; ver estados precisos en `READING_STATUS.md`. Hay fórmulas, figuras y posibles erratas pendientes de comprobación, identificadas en las notas. No se ejecutaron todos los programas de los libros.

El corpus fundamenta razonamiento funcional, estructuras, algoritmos y pruebas basadas en propiedades. No aporta por sí solo una escala universal de legibilidad, complejidad ciclomática o calidad, ni cubre completamente la arquitectura operativa de Elixir/OTP. Esas carencias deben quedar explícitas antes de diseñar las mediciones.

El determinismo de esta consulta es distinto del determinismo de un futuro evaluador. Un script puede calcular siempre el mismo número con la misma evidencia, pero eso no convierte una aproximación en una prueba de complejidad. No se implementó aquí el evaluador ni el optimization loop.

## Evaluador incorporado posteriormente

El MVP del revisor se documenta en `../README.md`. `evaluator_concepts.json`
conecta premisas y fuentes cotejadas con las reglas de `../rules/evaluator.json`.
Los umbrales de puntuación son política del proyecto, no afirmaciones de los
libros. Los estados de lectura y las observaciones pendientes permanecen vigentes.
