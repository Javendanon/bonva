# bonva — informes locales

Este directorio contiene salidas de evaluaciones y análisis locales. Los informes
no se versionan: incluyen rutas, tiempos y evidencia propios de cada ejecución.
Git conserva únicamente este README.

Para generar una comparación reproducible desde la raíz del repositorio:

```sh
cargo run --locked -- evaluate fixtures/good --baseline fixtures/bad --output reports/comparacion.json
```

El archivo de salida debe ser nuevo; el evaluador no sobrescribe informes.
Las pruebas usan las fixtures y los 64 casos de referencia de
`fixtures/scoring_oracle.json`, sin depender de informes guardados aquí.
