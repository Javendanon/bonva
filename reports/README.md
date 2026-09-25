# Informes de validación

## Regla de listas V1

- `list-bad.json`: un patrón append verificado, 366 casos de identidad aprobados;
  hallazgo por coste lógico cuadrático. A n=1024, 524800 constructores frente a 2048.
- `list-good.json`: prepend + reverse reconocido, 366 casos aprobados, sin hallazgo
  dentro del alcance de esta regla.
- `deejai-lists.json`: 39 archivos inspeccionados, cero sitios reconocidos y seis
  concatenaciones no soportadas. Resultado **incompleto**, no un certificado de eficiencia.

Los tres se ejecutaron mediante el wrapper de la skill instalada. El modo listas
lee el original sin copiarlo ni cargar sus módulos. Tiempos y reducciones son
observaciones de expresiones ejecutadas bajo el mismo mecanismo de evaluación;
no intervienen en el diagnóstico ni modifican el score MVP.

Validación: cinco pruebas ExUnit del detector, dos integraciones Rust de listas,
14 pruebas Rust del núcleo/modelo y tres integraciones Mix aprobadas. Se verifican
rechazo de callbacks arbitrarios y bindings ambiguos, casos finitos, modelo,
evidencia alterada, cobertura incompleta y ausencia de cambios al original.
La skill pasó `quick_validate.py` y su wrapper instalado fue ejecutado.

## Núcleo Rust vigente

- `deejai-rust-cold.json`: primera compilación en espacio reutilizable.
- `deejai-rust-warm.json`: misma evaluación con compilación reutilizada.

Ambos conservan las métricas, notas, gates, diagnóstico y hashes de `deejai.json`.
Ver `../docs/rust-validation.md` para tiempos y alcance de las comprobaciones.

## Referencias del MVP Python

Los resultados finales de la implementación anterior son:

- `deejai.json`: ejecución sobre una copia temporal de `deejai/web`; compilación exitosa, 85 pruebas aprobadas, score 6.533333, rechazo por `decision_max` y `target_score` del perfil inicial.
- `example.json`: comparación reproducible entre fixtures, 6.683333 → 10, delta 3.316667; misma suite y correctness sin descenso.

Los archivos `deejai-initial.json`, `deejai-check-2.json` y `deejai-check-3.json`
son diagnósticos intermedios del desarrollo del adaptador, **no evaluaciones finales**.
El primero documenta la omisión accidental de metadatos Git de una dependencia en
la copia. Los otros dos documentan la clasificación incorrecta de salidas `tmp/`
como cambios de fuente. Se conservan para explicar las correcciones, no deben
usarse para valorar deejai ni como ejemplos del contrato final.

Los hashes de implementación incluidos identifican qué código produjo cada informe.
