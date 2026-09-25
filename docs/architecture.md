# Arquitectura y decisiones del MVP

Documento histórico del MVP Python v0.1. La arquitectura vigente está en
`rust-architecture.md`; los indicadores descritos aquí se conservan.
El código del evaluador Python y sus pruebas se retiraron; este documento
describe decisiones históricas, no instrucciones de ejecución actuales.

## Evaluación del repositorio inicial

El repositorio contenía el master prompt, cuatro fuentes, notas consultables,
`scripts/knowledge.py` y dos pruebas Python. No había aplicación Mix ni CI propia.
Se conserva esa estructura y se añade un núcleo Python sin dependencias externas
y un adaptador Elixir. La unidad de entrada inicial es un proyecto Mix individual.
Umbrellas y código generado requieren ampliar la selección de fuentes.

## Separación de responsabilidades

1. El collector copia el proyecto, registra hashes, analiza fuente y ejecuta herramientas.
2. Los adaptadores producen métricas canónicas; no producen notas.
3. El motor puro valida métricas y política, normaliza y aplica gates.
4. El diagnóstico conecta hechos con reglas y conceptos por identificador exacto.
5. El revisor puede explicar el resultado, pero la aprobación pertenece al script.

El collector analiza antes de ejecutar hooks del proyecto. `Code.string_to_quoted`
parsea sin expandir ni ejecutar macros; esta decisión se apoya en la
[documentación del parser Elixir](https://elixir.hexdocs.pm/1.18.4/Code.html#string_to_quoted/2).
ExUnit se integra por eventos estructurados, no extrayendo números del resumen de consola.
Se conserva además el resumen nativo para inspección y errores de setup.

## Definición de los indicadores

Unidad: cada cláusula escrita como `def`, `defp`, `defmacro` o `defmacrop` con cuerpo.
Una declaración sin cuerpo no cuenta. Cada cláusula de una función multicláusula
se mide por separado: `function_count` es cantidad de cláusulas, no de funciones únicas.
Se omite contenido de `quote`, incluidos `unquote`, y no se expande `use` ni sigilos.
No se analiza código generado, templates HEEx ni Erlang. Funciones anónimas dentro
de una cláusula aportan sus decisiones a esa cláusula, pero no se cuentan por separado.

Indicador de decisiones: base 1; sumar 1 por nodo `if`, `unless`, `and`, `or`, `&&`, `||`;
sumar `max(cantidad de alternativas do - 1, 0)` por `case`, `cond` o `receive`.
No es complejidad ciclomática de un grafo de control: no cubre `with`, `try`, guards
arbitrarios, expansión de macros ni combinación de cláusulas. Se reporta como
`decision_indicator`, nunca como complejidad temporal ni Big-O.

Longitud: última línea con metadatos sintácticos (incluido `end`) menos línea del
`def`, más uno. Incluye líneas físicas intermedias; para cuerpos en una línea da 1.
Percentil 95: ordenar longitudes y seleccionar posición `ceil(0.95 × n)`, base 1.
Parámetros: aridad del encabezado tras retirar `when`; defaults cuentan una vez.

Normalización de coste:

`max(0, min(10, 10 × (bad - valor) / (bad - good)))`

Se aplica a máximos de decisiones y parámetros, y p95 de longitud. Pasa por 10 en
`good`, 0 en `bad`, e interpola entre ambos. Las operaciones usan Decimal;
solo la salida se redondea a seis decimales. Los gates de aprobación usan valores
sin redondear. La comparación usa las notas publicadas con precisión de seis decimales.

## Gates y faltantes

Se requiere compilación exitosa, comando de pruebas exitoso, mínimo de pruebas
ejecutadas, cero fallos, ausencia de casos omitidos salvo política explícita,
parseo completo, al menos una cláusula, decisiones bajo umbral y nota objetivo.
Un gate fallido rechaza aunque la nota sea alta. Ausencia de evidencia no es éxito.
Si hay fallos conocidos y faltantes, el estado global es rejected; los gates
unknown y dimensiones null preservan los faltantes. Sin fallos pero con gates
desconocidos, el estado es incomplete.

El gate separado `source_unchanged` compara la instantánea tras ejecutar las
herramientas. Los directorios de salida configurados (por defecto `tmp/`) pueden
cambiar en la copia; todas las diferencias quedan registradas. Los cambios a
entradas fuera de esos directorios bloquean aprobación sin falsear el resultado
del compilador. El inventario del original se comprueba completo, incluyendo tmp.
Se excluyen caches `_build` y metadatos Git del proyecto; los metadatos Git de
dependencias se conservan porque Mix los necesita para resolver dependencias Git.

El MVP no exige gates de cobertura, propiedades, mutación o rendimiento porque
todavía no los mide. Esos límites aparecen en cada informe.

## Reproducibilidad, integridad y confianza

El cálculo y el AST son deterministas respecto de entradas y versión de herramientas.
Compilación y pruebas son sensibles al entorno. Semilla 0 y max-cases 1 reducen
variaciones, pero no convierten procesos, red o servicios externos en funciones puras.
No se incorporan tiempos de reloj a la nota.

Los originales y extracciones citados se verifican por hash. Las fuentes de
política señalan secciones del master prompt. Ningún umbral se atribuye a Okasaki
o Hebert. Los hash permiten detectar diferencias; no son firmas de autenticidad.

No hay interfaz que acepte una nota del LLM. La API pura admite métricas para
pruebas internas: un llamador con control del proceso puede fabricar entradas.
Este MVP no protege contra código malicioso que manipule ExUnit, aliases de Mix,
variables, archivos externos o el propio evaluador. Para ese escenario harían falta
ejecución aislada, pruebas de aceptación externas y una política de confianza.

## Comparación y optimización

Cada baseline se vuelve a medir con la misma política. La comparación requiere
nota del candidato aprobada, delta positivo suficiente, ausencia de retroceso en
correctness y límite de descenso por dimensión. Se rechazan mediciones perdidas,
cambios de runtime y cualquier modificación a los archivos `test/`.
Igualdad de tests no demuestra igualdad de comportamiento fuera de su dominio.

El evaluador no refactoriza, revierte ni escribe en los proyectos. Se proporciona
un protocolo futuro de optimización, deshabilitado, para mantener separado el
revisor solicitado de un agente que cambie código.
