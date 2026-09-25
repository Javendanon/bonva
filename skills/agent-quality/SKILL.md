---
name: agent-quality
description: Evalúa proyectos Elixir con un núcleo Rust y evidencia calculada por scripts; genera puntuaciones según política o analiza construcción de listas. Úsala para revisiones medibles y comparaciones, no para refactorizar automáticamente.
---

# Evaluador determinista

Invoca `scripts/run.sh` relativo a esta skill. Requiere Cargo/Rust y Elixir >= 1.18 en PATH; la evaluación general también necesita Mix, dependencias y servicios de pruebas del proyecto. El wrapper compila incrementalmente el núcleo Rust con dependencias bloqueadas; no usa Python para evaluar.

Selecciona el modo según la petición:

- Evaluación general: `bash <skill>/scripts/run.sh evaluate /ruta/proyecto --output /ruta/externa/informe-nuevo.json`. Añade `--baseline /ruta/base` para comparar usando la misma política. Ejecuta compilación y pruebas en un workspace de caché: el código del proyecto y sus tests pueden tener efectos externos; respeta permisos y servicios autorizados.
- Construcción de listas: `bash <skill>/scripts/run.sh analyze-lists /ruta/proyecto --output /ruta/externa/listas-nuevo.json`. Solo lee `lib/**/*.ex` y `lib/**/*.exs`; no carga módulos del proyecto. Evalúa expresiones reconocidas de copia identidad con append de singleton o prepend seguido de reverse. No es un análisis general de complejidad.
- Si se pide una revisión integral con rendimiento de listas, ejecuta ambos modos y presenta sus resultados por separado. El informe de listas no altera la puntuación MVP.

Usa rutas absolutas. Caché e informes deben quedar fuera del proyecto evaluado; los informes no sobrescriben archivos. `--cache-dir /ruta/externa` cambia la caché. No edites el proyecto evaluado sin una petición específica.

Interpreta el JSON, no el código de salida aislado: 0 significa evaluación aceptada o análisis acotado completado sin hallazgos; 1 significa rechazo por política o hallazgos de listas; 2 significa error/incompleto. Conserva y comunica hallazgos parciales aunque el estado sea incompleto. Si falta una dependencia, un permiso o falla la integridad, informa el bloqueo; no inventes resultados ni cambies políticas para obtener aprobación.

Todas las métricas, puntuaciones y decisiones deben salir del ejecutable. Distingue:

- `findings` y `modeled_work`: conclusiones condicionadas a premisas verificadas, con fuentes en `knowledge` y huellas en `provenance`.
- `observations` y muestras en `evidence`: tiempos/reducciones sensibles al entorno. No son puntuaciones, asignaciones BEAM ni prueba empírica de Big-O.
- `evidence.files[].unsupported`, errores de parseo y `omitted_sites`: límites de cobertura. Cero hallazgos no certifica eficiencia global.

Los 366 casos de identidad constituyen comprobaciones finitas, no una prueba universal. La regla de listas no demuestra que el módulo original compile ni que una sustitución sea segura para toda la aplicación. No extrapoles la regla de append de singleton a concatenaciones de segmentos crecientes.

El wrapper localiza el repositorio por `AGENT_QUALITY_ROOT`, por `engine-root.txt` instalado o por su ubicación dentro del repositorio. Si falta el motor, pide su ruta; no busques ni instales un motor diferente. Para interpretar arquitectura o ampliar reglas, consulta `docs/list-construction.md` y `docs/rust-architecture.md` dentro de ese repositorio.
