# Property-Based Testing with PropEr, Erlang, and Elixir

Fred Hebert, P1.0, enero de 2019. Fuente `proper`; localizadores por archivo XHTML del EPUB, sin inventar páginas.
Estado: lectura en curso. Estas notas no habilitan reglas ni asignan puntuaciones.

## Capítulos 1–2: fundamentos y estructura (f_0013–f_0023)

Una propiedad combina una regla sobre comportamiento, un dominio de entradas y un mecanismo que la ejecuta. El marco genera casos y reduce contraejemplos. El éxito solo cubre lo ejercitado. El ejemplo de máximo descubre tanto una entrada fuera del dominio (lista vacía) como un fallo real por igualdad no cubierta. Distinguir error del programa, error de la propiedad y especificación incompleta.

## Capítulo 3: pensar en propiedades (f_0024–f_0030)

- Modelar con una implementación más simple e independiente; copiar la implementación al test replica sus errores. Un oráculo externo también tiene premisas.
- Generalizar ejemplos construyendo entradas junto a resultados conocidos.
- Combinar invariantes parciales; una comprobación de orden acepta vacíamente una salida siempre vacía.
- Ida y vuelta prueba compatibilidad entre operaciones, pero dos funciones identidad también la satisfacen. Añadir anclajes independientes del formato y ejemplos de referencia.
- Propiedades pequeñas separan causas de fallo y son más fáciles de mantener que una condición enorme.

Observación propia: pertenencia, longitud y orden no garantizan conservación de multiplicidades; una futura regla debe verificar multiconjuntos cuando los duplicados importen. No atribuir esa suficiencia al libro.

## Capítulo 4: generadores (f_0031–f_0036)

- Variedad de valores no asegura variedad de operaciones: claves casi siempre distintas no ejercitan sobrescritura. `collect`/`aggregate` ayudan a comprobar qué se ejecuta.
- Tamaño, transformación, rechazo y frecuencia son controles distintos. Un rechazo excesivo puede impedir generar datos; construir datos válidos puede ser más eficiente.
- Un tamaño relativo preserva variación; un tamaño fijo cambia el dominio. Los límites deben ser explícitos.
- Recursión probabilística bajo evaluación estricta puede construir ramas antes de elegirlas. La macro de diferimiento evita esa evaluación anticipada, pero no impone por sí sola un límite duro de profundidad.
- Recursión con contador o presupuesto aporta una condición de terminación independiente del azar.
- Llamadas simbólicas conservan la secuencia de construcción de valores opacos y efectos, facilitando reproducir un contraejemplo.

Posible errata, interpretación propia pendiente de comprobación contra original: el texto sobre `min(100, Size)*35` describe un mínimo sin techo aunque la expresión impone un techo al factor. No convertir ejemplos en scripts sin validarlos.

## Capítulo 5: uso responsable (f_0039–f_0047)

- Separar transformaciones puras de lectura de archivos, reloj y envío de mensajes facilita pruebas de unidades. Los efectos requieren pruebas de integración.
- Mantener adaptadores de negocio fuera del parser genérico y ocultar representación mediante interfaces reduce dependencias de implementación.
- Propiedades y ejemplos se complementan: los ejemplos del formato descubren casos que el generador no puede representar.
- El parser CSV mostrado tiene restricciones declaradas de columnas y encabezados duplicados. Sus pruebas no demuestran compatibilidad universal con CSV.
- Un dominio finito y abordable puede enumerarse exhaustivamente; el autor usa fechas de un intervalo y registros de cumpleaños. La exhaustividad es respecto de ese dominio, no de todas las entradas posibles.
- El ejemplo exhaustivo todavía usa referencias y años aleatorios en los registros; no copiarlo y afirmar determinismo estricto sin controlar esos elementos (observación propia).
- Cobertura completa es señal de ejecución, no prueba de ausencia de errores.
- Tipos opacos y especificaciones ayudan a sostener límites de abstracción; no equivalen a garantías completas del comportamiento.

## Capítulo 6: desarrollo guiado por propiedades (f_0048–f_0054)

- Separar casos de precios ordinarios y promociones permite construir un resultado esperado sin duplicar el algoritmo evaluado. Las dependencias entre generadores deben fijarse antes de reutilizarse.
- Comprobar que una propiedad falla sobre una implementación incorrecta es una validación útil del test.
- Generadores restringidos al caso feliz pueden ocultar requisitos omitidos, incluso con cobertura completa.
- Pruebas negativas necesitan distribución calibrada; si casi todo falla por la misma validación inicial, no exploran capas posteriores.
- Relajar restricciones descubre ambigüedades: duplicados, valores desconocidos, cantidades cero, precios negativos. Decidir su semántica exige un contrato de negocio, no una suposición del revisor.
- Fallos de tipo y fallos de comportamiento no se sustituyen entre sí. Las herramientas/versiones del libro son históricas.

## Capítulo 7: reducción de contraejemplos (f_0055–f_0058)

- Reducir simplifica una entrada mientras conserva el fallo; no garantiza un mínimo global.
- Un punto neutro depende del dominio. `SHRINK` sugiere alternativas relevantes y `LETSHRINK` permite sustituir una composición por partes válidas.
- Reducir valores dentro de una estructura no reduce necesariamente su forma. La sustitución por una rama debe conservar el tipo aceptado por la propiedad.
- Registrar el contraejemplo es distinto de registrar solamente una semilla.

## Capítulo 8: búsqueda dirigida (f_0059–f_0063)

- La búsqueda usa retroalimentación numérica y vecinos; el recocido simulado puede salir de óptimos locales, sin garantizar encontrar el óptimo global.
- Vecinos demasiado similares reducen diversidad. Separar exploración amplia de búsqueda específica tiene costos y beneficios.
- La métrica optimizada puede ser solo un indicador aproximado: la medida de balance del ejemplo no demuestra altura balanceada.
- El ejemplo de quicksort demuestra que más recorridos locales pueden evitar comportamientos globales malos; contar recorridos sin condiciones puede recomendar mal.
- Los experimentos que usan tiempo de reloj son sensibles al equipo. No convertir sus umbrales en un puntaje estricto.
- Observación propia: el texto alterna «quadratic» y «exponential» al describir el mal caso del quicksort presentado. Conservar la distinción; una demora experimental no demuestra ninguna clase asintótica.
- Las limitaciones de API y soporte de PropCheck descritas corresponden a enero de 2019.

## Capítulo 9 — Estado y concurrencia (unidades 68–74, f_0066–f_0072)

- Modelo: estado inicial determinista, comandos, precondiciones, transiciones y postcondiciones. La fase simbólica construye secuencias sin ejecutar el sistema; la real compara resultados con el modelo.
- El resultado usado en una transición es opaco durante generación: guardar referencias simbólicas a PID/socket puede ser necesario, inspeccionarlas como resultados reales no es válido.
- Las restricciones del generador deben estar también en precondiciones; shrinking y paralelización modifican secuencias y pueden eliminar la operación que introducía un recurso.
- Preparar y limpiar cada iteración evita arrastrar estado. SETUP alrededor de la propiedad tiene otro ciclo de vida; no equivale a reiniciar cada caso.
- Caché: modelo FIFO sencillo, claves repetidas deliberadamente, lecturas posteriores para observar escrituras y borrados. Retornar éxito no prueba que se haya actualizado el estado.
- Inyectar un defecto conocido permite comprobar si la propiedad detecta ese defecto. La distribución de llamadas no garantiza lecturas relevantes ni cobertura de todos los escenarios.
- La prueba paralela divide una secuencia en prefijo común y ramas; busca una intercalación compatible con el modelo. Pasar miles de casos no demuestra ausencia de carreras.
- El ejemplo tiene escrituras ETS compuestas: comprobar ausencia e insertar no forman una operación atómica. Dos escritores pueden crear claves duplicadas. Serializar escrituras por el proceso propietario corrige ese caso bajo la interfaz descrita.
- El planificador no es determinista. yield favorece ciertas intercalaciones, sin garantizar exploración exhaustiva. Las herramientas/flags citados son información histórica de 2019.
- Interpretación para este proyecto: semilla fija no fija la planificación concurrente. No equiparar reproducibilidad de entradas con reproducibilidad de toda la ejecución.

## Capítulo 10 — Integración y modelo independiente (unidades 75–83, f_0073–f_0081)

- La librería con PostgreSQL se prueba con un mapa como modelo; abstraer el acceso facilita mantener expectativas cuando cambia el almacenamiento.
- Empezar con llamadas amplias detecta excepciones, pero una postcondición siempre verdadera no verifica resultados. Añadir contratos por operación: nuevo/existente, disponible/agotado, encontrado/ausente.
- Un shim da nombres distintos a escenarios que invocan la misma función: más repetición puede hacer el modelo más explícito, medible y fácil de depurar. No asumir que reducir líneas/repetición siempre mejora calidad.
- El shim ejecuta solo en fase real y puede adaptar efectos o sincronización; el modelo conserva expectativas propias. No derivar el resultado esperado del resultado que se pretende comprobar.
- Solo normalizar orden cuando el contrato lo declare irrelevante. Representaciones distintas de texto pueden significar lo mismo; comparar bytes/listas sin normalización genera falsos fallos.
- Separar propiedades de caracteres especiales SQL y transiciones de inventario permite enfocar pruebas. Excluir caracteres de un generador deja una obligación pendiente; no demuestra soporte de esos caracteres.
- Las precondiciones deben ser totales ante estados alcanzados por shrinking: una consulta a una clave que desapareció no debe romper el propio test.
- El caso de devolver un libro nunca prestado descubre una condición faltante en SQL. Distinguir inexistencia de falta de disponibilidad exige un contrato explícito de errores.
- El análisis de concurrencia del libro depende de las consultas concretas y sus transacciones. La comprobación previa y actualización separadas aún pueden devolver una categoría de error incorrecta; no generalizar seguridad a toda consulta SQL.
- Fallos de conectividad quedan fuera de esa propiedad, explícitamente. Los ejemplos de conexión por petición y limpieza no constituyen recomendaciones universales para producción.

## Capítulo 11 — Máquinas de estados y tiempo (unidades 84–91, f_0082–f_0089)

- Usar modelo FSM cuando los estados son observables para el usuario del sistema. Distinguir nombre del estado de datos contenidos; implementación interna FSM por sí sola no obliga a modelar así.
- Estado y datos iniciales deben ser deterministas para repetir y reducir casos. Precondiciones deben hacer inequívoco el destino de una llamada en un estado dado.
- Circuit breaker: estados no registrado, operativo, disparado y bloqueado manualmente. Las llamadas ignoradas, errores, timeouts y operaciones manuales tienen efectos distintos.
- El modelo inicial falla porque la biblioteca reduce contadores tras éxitos. Corregir una interpretación documental errónea no equivale a aceptar cualquier comportamiento del sistema como correcto.
- Medir visitas a transiciones detecta estados casi inexplorados; ajustar frecuencias mejora exploración, no proporciona prueba exhaustiva.
- El ejemplo excluye recuperación temporal. Esperas reales, mocks e inyección de eventos tienen alcances diferentes; inyectar eventos de timeout permite controlar escenarios cuando la interfaz lo permite.
- Retrasar timers reales una hora es una hipótesis de duración del ejemplo, no una garantía matemática para una ejecución arbitrariamente larga.

## Apéndices (unidades 93–107)

- Soluciones: verificar estabilidad de ordenación y valores/conflictos al fusionar mapas exige propiedades adicionales a orden/pertenencia de claves.
- Generadores recursivos: LAZY evita expansión anticipada; presupuesto de tamaño repartido entre ramas limita crecimiento. No confundir terminación probable con límite garantizado.
- Shrinking estructural requiere que alternativas mantengan el tipo esperado. Las referencias opacas del sistema son útiles cuando los identificadores no pueden predecirse.
- Traducciones Elixir contienen versiones iniciales intencionalmente defectuosas, además de correcciones; no copiar el apéndice completo como implementación final.
- Observaciones propias de posibles erratas a comprobar antes de reutilizar: en unidad 103 las precondiciones de autor/título desconocido llaman has_isbn; en unidad 90 weight discrimina error aunque el shim se llama err; en unidad 94 la explicación de orden parece invertida. Mantenerlas como sospechas, no reglas.
- La instalación PostgreSQL y la tabla de generadores corresponden a la edición; publicidad final no es fundamento técnico.

## Estado

Leídos bloques 001–027 y releídas unidades 32–33 que habían quedado truncadas. Síntesis de capítulos y apéndices completa para el texto extraído. Diagramas y exactitud ejecutable de ejemplos no han sido validados íntegramente. No trasladar estas notas a una política de aprobación antes de completar los otros libros.
