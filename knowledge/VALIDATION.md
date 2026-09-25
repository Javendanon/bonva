# Comprobaciones realizadas

Fecha: 2026-09-15.

- `python3 scripts/knowledge.py verify`: cuatro libros, sin discrepancias de hash, número de unidades, caracteres, localizadores o unidades vacías.
- `python3 -m unittest discover -s tests -v`: dos pruebas aprobadas. Cubren consulta repetida con JSON idéntico, recuperación de una página, entradas inválidas, ingestión EPUB siguiendo el spine, Unicode, exclusión de script, estado inicial no leído, rechazo de sobrescritura y detección de modificación del original.
- Cotejo visual puntual de PFDS PDF 66: símbolos del invariante de balance y calificación amortizada; registrado en sus notas.
- Reextracción de PFDS con la función PDF del script y PyMuPDF 1.28.2: las 162 unidades coinciden exactamente con el corpus almacenado.

La primera ejecución del ensayo de ingestión falló porque el directorio temporal de la prueba usaba `/var` y el archivo resuelto usaba `/private/var`. Se corrigió el montaje de la prueba resolviendo ambos caminos; las pruebas finales pasaron.

Estas comprobaciones validan integridad y operaciones concretas de la memoria local. No son pruebas del futuro evaluador, de todas las variantes de PDF/EPUB, de las demostraciones de los libros ni de sus programas. El extractor EPUB nuevo asume contenido UTF-8; otros encodings requieren adaptación explícita.
