# Prompt inicial para Codex — MyList / Fase 0

Actúa como ingeniero de software responsable de construir MyList, una aplicación de escritorio personal. Lee detenidamente `PROJECT_CONTEXT.md` y `AGENTS.md`. Ambos contienen las decisiones funcionales y técnicas aprobadas. No implementes todo el catálogo en esta sesión.

## Objetivo

Implementar exclusivamente la Fase 0: establecer una base técnica funcional, verificable y fácil de ampliar para Tauri 2 + React + TypeScript + Vite + Rust + SQLite. Inspecciona primero el directorio actual. Si está vacío, inicializa el proyecto sin sobrescribir los documentos entregados. Si ya existe un proyecto, adapta su estructura preservando cambios del usuario.

## Tareas

1. Comprueba herramientas y prerrequisitos locales (Node/npm, Rust, cargo, dependencias nativas de Tauri y WebView2 en Windows). No instales componentes globales ni cambies configuración del sistema sin autorización; si falta algo crítico, explica el comando o procedimiento necesario.
2. Crea el scaffold oficial compatible con Tauri 2, React y TypeScript estricto. Configura nombre MyList, identificador estable (propuesta `com.mylist.desktop`), lockfiles, scripts y `.gitignore`. No fijes versiones inventadas; usa versiones compatibles verificadas.
3. Organiza la estructura modular indicada en el contexto sin generar capas vacías innecesarias. Configura permisos mínimos de Tauri. Usa un icono provisional propio, reemplazable en una fase posterior.
4. Implementa shell visual inicial oscuro: sidebar con Biblioteca, Favoritos, Continuar viendo, Estadísticas y Configuración; área principal con estado vacío, estructura preparada para pestañas internas y un encabezado. Los módulos de fases futuras deben estar marcados como pendientes y no simular operaciones. Centraliza tokens visuales y textos de navegación para futura traducción.
5. Implementa inicialización de SQLite en el directorio de datos de la aplicación, con migración numerada mínima y versionada (metadatos/configuración inicial), foreign keys activadas, apertura idempotente y errores claros. No implementar todavía todas las tablas de dominio; documenta cómo se agregarán con migraciones.
6. Implementa un comando Tauri tipado de diagnóstico que compruebe apertura y versión del esquema; UI informa fallo sin exponer rutas privadas. No crear API REST ni un servidor en producción.
7. Añade pruebas Rust para inicialización repetible y migración usando una base temporal; prueba UI del estado inicial y del diagnóstico simulado. Configura lint/typecheck/test/build y ejecuta cuanto sea viable. Si el entorno no permite abrir ventana nativa o compilar Windows, indícalo sin simular resultados.
8. Documenta requisitos, instalación, comandos de desarrollo, pruebas, build, ubicación general de datos, arquitectura y limitaciones en README. Crea `docs/PROGRESS.md` con el estado de Fase 0 y una lista de pendientes vinculada al roadmap. Conserva los documentos fuente.

## Criterios de aceptación

- Se inicia una ventana de escritorio llamada MyList mediante el comando documentado de desarrollo en un entorno compatible.
- La interfaz base se adapta al tamaño de ventana y no depende de recursos remotos.
- SQLite se inicializa sin destruir datos al reiniciar y el diagnóstico IPC funciona.
- Los comandos de verificación y pruebas disponibles pasan; cualquier bloqueo está documentado.
- No existen funcionalidades de fases 1–8 implementadas parcialmente como si estuvieran terminadas. No hay backend web ni cuentas.

## Entrega

Trabaja de manera autónoma dentro del alcance. Al finalizar entrega resumen, arquitectura realmente implementada, archivos relevantes, comandos de ejecución, resultados de pruebas reales, limitaciones y propuesta de Fase 1. Detente y espera aprobación antes de continuar.
