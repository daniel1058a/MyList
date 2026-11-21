# MyList — Instrucciones para agentes

1. Leer `PROJECT_CONTEXT.md`, `README.md` y `docs/PROGRESS.md` (si existen) antes de modificar código. El contexto aprobado es fuente de verdad; no cambiar alcance o stack sin autorización.
2. Implementar únicamente la fase solicitada. Inspeccionar repositorio y cambios existentes, preservar trabajo del usuario. Presentar plan breve, implementar y verificar; no avanzar a otra fase por cuenta propia.
3. Stack: Tauri 2, React, TypeScript estricto, Vite, Rust, SQLite local. No servidores HTTP en producción, APIs externas ni cuentas. Evitar dependencias o abstracciones innecesarias; seguir convenciones del repositorio.
4. Separar presentación, IPC, servicios, dominio, repositorios e infraestructura según necesidad. Validar entradas en Rust; devolver errores comprensibles y tipados. Usar parámetros SQL, transacciones y claves foráneas. Nunca admitir acceso arbitrario a archivos desde frontend.
5. Proteger datos: utilizar directorio de datos de aplicación, migraciones versionadas, respaldos antes de restauraciones o cambios de riesgo. No borrar datos reales durante pruebas; usar directorios y bases temporales. No acceder a secretos ni subir información personal a git.
6. UI cinematográfica y accesible, con textos internacionalizables. No inventar funcionalidades ni mostrar botones que simulen estar implementados; identificar secciones futuras claramente.
7. Agregar pruebas para reglas de negocio, fallos y regresiones. Ejecutar los comandos documentados de lint, typecheck, test y build que el entorno soporte. Reportar con honestidad bloqueos, comprobaciones no ejecutadas y diferencias de plataforma.
8. Actualizar README e historial de avance en `docs/PROGRESS.md` con fases, decisiones, pruebas, pendientes y próxima tarea. Mantener instrucciones de ejecución reproducibles y lockfiles. No afirmar que se probó Windows sin haberlo hecho.
9. Al finalizar: resumir qué se implementó, archivos relevantes, comandos ejecutados, resultados, riesgos y siguiente acción propuesta. No hacer push, publicar ni eliminar archivos ajenos sin autorización.
10. Entregar el ejecutable actualizado siempre en `src-tauri/target/release/mylist.exe`. Usa rutas temporales para verificaciones aisladas. Si Windows mantiene el ejecutable en uso, coordina el cierre de MyList y actualiza esa misma ruta.
