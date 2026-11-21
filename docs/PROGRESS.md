# Progreso de MyList

## 2026-11-21 · Publicación de la versión 1.0.0

- El repositorio público incluye el código, las pruebas, los archivos de bloqueo de dependencias y la documentación. Las capturas del README usan una biblioteca de demostración y portadas creadas para la prueba.
- La publicación distribuye el instalador x64 como adjunto de la versión `v1.0.0`, con el mismo SHA-256 verificado en la Fase 8. Las bases SQLite, copias de seguridad y carpetas de compilación quedan fuera de Git.

## 2026-11-21 · Fase 8 · MyList 1.0.0

- **Empaquetado.** Tauri genera un instalador NSIS x64 para la cuenta de Windows actual. Incluye español e inglés en el asistente, los accesos directos estándar de Tauri y un desinstalador que conserva los datos por defecto. WebView2 Evergreen se incluye para instalarlo cuando falta. Se comprobó la firma del runtime de Microsoft.
- **Identidad.** MyList usa la versión 1.0.0 en Tauri, Cargo y npm. El icono provisional usa un fotograma ámbar con el monograma de MyList. El ICO contiene seis resoluciones de 16 a 256 px.
- **Datos.** Se mantienen el identificador `com.mylist.desktop`, el directorio `%APPDATA%\com.mylist.desktop` y el esquema SQLite 6. Todas las pruebas usaron directorios temporales o una identidad de aplicación de auditoría. El catálogo personal quedó fuera de las pruebas.
- **Entrega local.** El instalador está en `src-tauri/target/release/bundle/nsis/MyList_1.0.0_x64-setup.exe` (285,07 MiB). El ejecutable directo permanece en `src-tauri/target/release/mylist.exe`. Se comprobó que el ejecutable principal es PE AMD64 y se verificó el SHA-256 de ambos artefactos al copiarlos. El instalador está sin firma digital.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| ESLint, TypeScript, Vitest y Vite de producción | Correctos. 23 pruebas aprobadas |
| Formato, Cargo con lockfile y Clippy | Correctos. 35 pruebas aprobadas, incluidas migraciones, restauración transaccional, imágenes y retención |
| Regresiones en WebView2 con IPC real | Pasaron las fases 1 a 6.5, incluidos CRUD, portadas y panoramas, favoritos, categorías, orden, filtros, progreso, historial y estadísticas. Se actualizaron las aserciones históricas al esquema 6 y al formato de fecha en español. Se midió contraste mínimo de 7,99:1 en oscuro y 5,01:1 en claro para los pares de texto comprobados |
| Tauri y NSIS | `npm.cmd run tauri build -- --bundles nsis --no-sign --ci` correcto. Se comprobó en el script generado la inclusión del runtime Evergreen y el identificador de producción |
| Instalación limpia y arranque | NSIS 1.0.0 inició con un catálogo vacío y esquema 6. Se comprobaron las rutas de los accesos del escritorio y del menú Inicio y el registro de desinstalación |
| Actualización 0.9.0 → 1.0.0 | Compilaciones con el mismo código y versiones distintas del instalador de auditoría. SQLite conservó el mismo SHA-256 antes de abrir 1.0.0. La serie y su portada reaparecieron |
| Catálogo, idioma y operación local | La aplicación instalada arrancó con el PATH de Windows y sin acceso del WebView a destinos externos. Se crearon y editaron series, se importaron portadas y se verificaron ambos idiomas, los temas claro y oscuro y cinco anchos de ventana. Se recuperaron el catálogo y el idioma tras reiniciar |
| Respaldos y exportación | Verificados el respaldo ZIP, el rechazo de un ZIP corrupto, la cancelación y la restauración con copia preventiva. Se restauró además un ZIP sintético generado durante la Fase 7. Se verificaron CSV, Markdown, `integrity_check` y `foreign_key_check` |
| Desinstalación predeterminada | Se retiraron ejecutable, desinstalador, accesos y registro. Se conservaron SQLite con el mismo SHA-256, las portadas y las copias preventivas |

SHA-256 del instalador `829257C7232046E8E7C2A30B41E681334CC323F2A67A6AA6A5D397FF6010ECFE`.

Las pruebas de instalación y desinstalación se ejecutaron en Windows 11 x64 con una variante de auditoría de identidad propia y datos sintéticos. El equipo ya tenía WebView2 y Windows Sandbox estaba deshabilitado. Queda sin ejecutar la instalación del runtime en un Windows que carezca de él. También queda pendiente la validación final del usuario. Windows puede advertir o bloquear la instalación de un ejecutable sin firma según la política del equipo. [Reputación de aplicaciones de Windows](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation)

## 2026-11-21 · Fase 7

- **Respaldos.** El ZIP usa un snapshot SQLite, las imágenes referenciadas y un manifiesto con tamaños y SHA-256. Los respaldos automáticos se comprueban al abrir la aplicación y cada 15 minutos mientras está abierta. Se conservan las cinco últimas copias automáticas verificadas.
- **Restauración.** MyList prepara el ZIP en un directorio temporal y muestra fecha, esquema y recuentos antes de confirmar. Valida las entradas, los hashes y las relaciones. Genera una copia preventiva y usa una transacción SQLite para publicar el catálogo con las imágenes ya instaladas.
- **Exportaciones e idioma.** Configuración permite guardar CSV con BOM UTF-8 y Markdown jerárquico. Las traducciones de interfaz se mantienen en recursos tipados para español e inglés. Idioma y copias automáticas se guardan en `settings` con valores predeterminados compatibles con el esquema 6.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript y Vitest | Correctos. 23 pruebas aprobadas, incluido cambio de idioma en vivo, persistencia y confirmación de restauración |
| Formato, Cargo con lockfile y Clippy | Correctos. 35 pruebas aprobadas, incluidos snapshot con WAL, imágenes e historial, rechazo de ZIP inválidos, reversión durante la copia SQLite, retención y contenido de CSV y Markdown |
| WebView2 y selectores nativos de Windows | Verificados el respaldo ZIP, las exportaciones CSV y Markdown, el rechazo de un ZIP corrupto, la cancelación, la restauración confirmada con copia preventiva y la recuperación de imágenes e idioma después de reiniciar. Pasaron las regresiones de estadísticas, seguimiento y detalle de las fases 6.5, 6 y 5.5 |
| Compilación nativa sin instalador | Correcta. Se actualizó `src-tauri/target/release/mylist.exe` en la ruta habitual y se verificó su SHA-256 |

La Fase 7 se aprobó antes del empaquetado de la versión 1.0.0.

## 2026-11-21 · Fase 6.5

- **Dashboard.** Indicadores compactos, cuadrícula adaptable con columnas independientes, donas de estado y categoría, cinco géneros con opción de ampliar e histograma con diez intervalos. Los recuentos y porcentajes se pueden consultar desde los gráficos. Los intervalos vacíos permanecen visibles.
- **Exploración.** Un dato con resultados abre Biblioteca con un filtro temporal. Para estado, categoría o nota, también se filtra la lista de contenidos al abrir una serie desde esos resultados. La búsqueda, los filtros y el orden guardados se recuperan al quitar el filtro.
- **Ranking.** Usa las miniaturas locales y las mismas estrellas de la cabecera. Las tarjetas abren la pestaña de la serie. Los gráficos se dibujan con SVG y CSS y respetan la preferencia de movimiento reducido.
- **Datos.** Se reutilizan las consultas estadísticas y sus reglas. SQLite permanece en el esquema 6 y las pruebas se ejecutan con perfiles y bases temporales.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 21 pruebas de interfaz y filtros aprobadas |
| Formato, Cargo con lockfile y Clippy | Correctos. 30 pruebas aprobadas, incluidas la proyección de identificadores, notas y portada del ranking |
| WebView2 con IPC real | Se verificaron temas claro y oscuro, cinco anchos de ventana, movimiento reducido, leyendas largas, cero resultados, portadas, tooltips y navegación filtrada. Pasaron las regresiones de filtros de Biblioteca y seguimiento de las fases 4 y 6 |
| Compilación nativa sin instalador | Correcta. Se reemplazó `src-tauri/target/release/mylist.exe` y se comprobó su SHA-256. La ventana que estaba abierta conservó su sesión |

La autorización para la Fase 7 llegó después de esta verificación.

## 2026-11-21 · Fase 6

- **Estadísticas.** Rust obtiene los totales y distribuciones de una lectura transaccional de SQLite. Cada sesión aporta una vez su contador de episodios y su marca de finalización. La interfaz distingue episodios actuales y anteriores, muestra el promedio de contenidos con nota y presenta un ranking de hasta diez series. La distribución de notas agrupa por parte entera y muestra el 10 por separado.
- **Continuar viendo.** La lista consultada en SQLite incluye solo sesiones activas en Mirando. Usa los mismos controles y el mismo comando de seguimiento que el detalle. Al confirmar un cambio se actualizan la lista y la proyección del catálogo. El acceso directo reutiliza la pestaña de la serie.
- **Datos.** La migración 0006 añade un índice parcial y una marca que cambia cuando se modifica el estado o el avance. Para las sesiones activas previas usa como aproximación la última actualización del contenido. En caso de empate se ordena por identificador de sesión descendente. Las verificaciones usan perfiles o bases temporales.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 19 pruebas de interfaz aprobadas |
| Formato, Cargo con lockfile y Clippy | Correctos. 30 pruebas aprobadas, incluidas migración 5 → 6, orden de actividad y recuento de sesiones activas y anteriores |
| WebView2 con IPC real | Verificados el estado vacío, los totales, el orden de actividad, salir al finalizar, volver al avanzar en una revisualización, el acceso al detalle y la persistencia al reiniciar. Se comprobó la interfaz en ambos temas y a 360, 720 y 1180 px. Pasaron las regresiones nativas de seguimiento, filtros y detalle de las fases 3, 4 y 5.5 |
| Compilación nativa sin instalador | Correcta. El ejecutable de Fase 6 se instaló en la ruta habitual `src-tauri/target/release/mylist.exe` y se verificó por SHA-256 |

La autorización para la Fase 6.5 llegó después de esta verificación.

## 2026-11-21 · Fase 5.5

- **Cabecera de serie.** La portada se mantiene nítida a la izquierda. El panorama local o la portada de respaldo se difuminan hacia el fondo del tema. La información sigue el orden de título, géneros, descripción ampliable, promedio, recuentos por estado y acciones. El promedio conserva las reglas existentes y cada estrella se rellena según su fracción real.
- **Detalle individual.** El título de cada tarjeta abre el contenido dentro de la pestaña de su serie. El detalle muestra los textos completos, el seguimiento, las fechas y el historial. Al volver se recuperan los filtros, el desplazamiento y el foco. Las tarjetas mantienen las acciones frecuentes visibles y agrupan el resto en **Más acciones**, incluida la alternativa al arrastre.
- **Datos y recursos.** La migración 0005 añade dos columnas a las series existentes y reutiliza el almacenamiento local. Los panoramas se cargan solo al entrar en la serie mediante una vista WebP de hasta 1280 × 720 píxeles. La portada y el panorama se administran de forma independiente. Las pruebas usan perfiles y bases temporales.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 18 pruebas de interfaz, incluidas estrellas fraccionadas, expansión de la descripción y regreso con foco, filtros y desplazamiento |
| Formato, pruebas Rust con lockfile y Clippy | Correctos. 28 pruebas aprobadas, incluidas la migración 4 → 5, la conservación del historial y las preferencias, y la limpieza de imágenes después de un guardado fallido |
| WebView2 con IPC real y perfiles temporales | Verificados los flujos de las fases 1 a 5.5. Pasaron el reemplazo por arrastre, la persistencia tras reiniciar, el regreso a la lista y la integridad referencial |
| Apariencia y carga de recursos | Se comprobó que la cuadrícula solicita la portada y que el panorama acotado se carga al abrir la serie. Cabecera y detalle se revisaron a 360, 540, 721, 920, 1180 y 1600 px. El contraste mínimo de los pares de texto medidos fue 7,99:1 en oscuro y 5,01:1 en claro |
| Compilación nativa sin instalador | Correcta. Se definió `CARGO_TARGET_DIR` para generar `src-tauri/target/phase55/release/mylist.exe` porque la sesión abierta de MyList mantenía en uso el ejecutable anterior |

La autorización para la Fase 6 llegó después de esta verificación.

## 2026-11-21 · Fase 5

- **Configuración.** Se pueden consultar, crear, renombrar y eliminar géneros y categorías con sus recuentos de uso. Al renombrar se conservan las asociaciones. El borrado confirmado de un género lo desvincula de sus series. El de una categoría en uso requiere seleccionar otra categoría, y la reasignación y el borrado se confirman en la misma transacción. Al retirar un elemento se limpia su selección en los filtros y se conservan los demás criterios.
- **Aspecto y navegación.** Los temas claro y oscuro y los tres tamaños de portada se guardan en las preferencias existentes. La cuadrícula se adapta al ancho disponible. La barra lateral, las pestañas, los formularios y los estados de arrastre comparten los nuevos estilos. Las pestañas abiertas se mantienen al visitar Configuración. Las portadas siguen usando miniaturas con carga diferida y las transiciones respetan la preferencia de movimiento reducido.
- **Datos.** SQLite permanece en el esquema 4. Las preferencias anteriores se abren con los valores visuales predeterminados. Las pruebas utilizaron perfiles y bases temporales.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 16 pruebas de interfaz aprobadas, incluidas la restauración de preferencias visuales y la confirmación de borrado con limpieza de filtros |
| Formato, pruebas Rust con lockfile y Clippy | Correctos. 26 pruebas aprobadas, incluidas validación de nombres, conservación de asociaciones, reasignación y reversión ante fallo |
| WebView2 con IPC real y perfiles temporales | Verificados los flujos de las fases 1 a 5, la persistencia tras reiniciar y la integridad referencial. Revisados contraste de texto en ambos temas, foco por teclado, movimiento reducido y redimensionamiento a 360, 720 y 1180 px |
| Build nativo sin instalador | Ejecutable de producción en `src-tauri/target/release/mylist.exe` |

La autorización para la Fase 5.5 llegó después de esta verificación.

## 2026-11-21 · Fase 4

- **Búsqueda y filtros.** La lógica usa una proyección del catálogo cargado con títulos de contenidos y estados de sus sesiones activas. La búsqueda parcial normaliza mayúsculas y tildes y devuelve una tarjeta por serie. Búsqueda, géneros, estado y favoritos se combinan. Los géneros admiten TODOS y CUALQUIERA.
- **Orden.** Título, promedio, fecha de registro y última actualización admiten ambas direcciones y desempate estable por ID. Las series sin nota quedan al final al ordenar por promedio.
- **Filtros internos.** Categoría y estado seleccionan una parte de la lista sin cambiar el orden manual. Hay un restablecimiento para el catálogo y otro en cada serie.
- **Preferencias.** MyList guarda la búsqueda, los filtros y el orden como JSON validado en la tabla existente `settings`, con una clave versionada y filtros internos por ID de serie. SQLite permanece en el esquema 4. El plan de consulta utiliza los índices existentes de título, relación de contenidos y sesión activa. Los cambios de búsqueda no vuelven a consultar el catálogo.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 14 pruebas aprobadas, incluidas búsqueda parcial con resultados únicos, combinaciones, estados vacíos y todos los criterios de orden |
| Formato, pruebas Rust con lockfile y Clippy | Correctos. 23 pruebas aprobadas, incluidas la persistencia de preferencias y la proyección de estados de sesiones activas |
| WebView2 con IPC real y perfiles temporales | Verificados la búsqueda, los filtros combinados, el orden por calificación, el restablecimiento y la restauración tras reiniciar. Los filtros internos mantuvieron el orden manual. Pasaron los flujos de las fases 1 a 3 |
| Build nativo sin instalador | Ejecutable de producción en `src-tauri/target/release/mylist.exe` |

La autorización para la Fase 5 llegó después de esta verificación.

## 2026-11-21 · Fase 3

- **Progreso y estados.** Las tarjetas admiten +1, −1 y edición directa. El total es opcional. Rust aplica los límites, deduce el estado a partir del avance y valida los cambios manuales. Reiniciar el progreso al cambiar a Por ver requiere confirmación cuando ya hay episodios vistos.
- **Sesiones e historial.** La migración 0004 añade una sesión inicial a cada contenido existente y conserva los datos anteriores. Cada sesión cuenta como completa una sola vez, incluso si se corrige su estado. Volver a ver se habilita tras completar la sesión activa e inicia otra con contador y fechas nuevos. Las fechas son opcionales y Rust valida el calendario y su orden. El borrado de un contenido elimina sus sesiones.
- **Calificaciones.** SQLite guarda décimas enteras o un valor nulo. El promedio simple se calcula desde los contenidos calificados, se muestra en Biblioteca y Favoritos, y se actualiza al guardar o retirar una nota o al eliminar contenido.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 9 pruebas de interfaz aprobadas |
| Formato, pruebas Rust con lockfile y Clippy | Correctos. 21 pruebas aprobadas, incluidas la migración 3 → 4, fechas inválidas, límites, prevención de sesiones duplicadas, reversión ante fallo y recálculo del promedio |
| WebView2 con IPC real y perfiles temporales | Verificado el avance conocido y desconocido, los estados automáticos y manuales, el historial, las fechas y calificaciones tras reiniciar, el promedio después de cambios y borrado, y los flujos de las fases 1 y 2 |
| Build nativo sin instalador | Ejecutable de producción en `src-tauri/target/release/mylist.exe` |

La autorización para la Fase 4 llegó después de esta verificación.

## 2026-11-21 · Fase 2

- **Pestañas.** Una por ID de serie. Se pueden alternar y cerrar durante la sesión. Los cambios de título se reflejan en la pestaña y el borrado de una serie la retira.
- **Contenidos.** Formularios para registrar y editar título, categoría, portada, descripción, año y opinión. Las tarjetas horizontales conservan los saltos de línea al expandir los textos. Cada serie mantiene su orden, modificable con arrastre o con **Subir** y **Bajar**.
- **Datos.** La migración 0003 conserva la configuración y el catálogo de las fases anteriores y añade categorías normalizadas reutilizables y contenidos con claves foráneas. Cada cambio de contenido actualiza la fecha de modificación de su serie. El borrado de una serie elimina sus contenidos en la misma transacción y conserva las categorías para reutilizarlas.
- **Portadas.** Se comparte el flujo de copias administradas y miniaturas con la Fase 1. SQLite confirma el cambio antes de retirar copias anteriores. Un cierre inesperado durante esa limpieza puede dejar una copia administrada sin referencia.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 7 pruebas de interfaz aprobadas |
| Formato, pruebas Rust con lockfile y Clippy | Correctos. 14 pruebas aprobadas, incluidas la migración 2 → 3, persistencia, validación, aislamiento entre series y eliminación de copias administradas |
| WebView2 con IPC real y perfiles temporales | Verificados ambos flujos de Fase 1 y 2. Se comprobó el arrastre real, categorías compartidas, reapertura con orden, textos, año y portada, y borrado en cascada con `PRAGMA foreign_key_check` sin violaciones |
| Build nativo sin instalador | Ejecutable de producción en `src-tauri/target/release/mylist.exe` |

La autorización para la Fase 3 llegó después de esta verificación.

## 2026-11-21 · Fase 1

- **Catálogo.** Biblioteca y Favoritos usan una cuadrícula adaptable. Los formularios de alta y edición guardan al confirmar. El borrado requiere confirmación y los favoritos se actualizan al completar la escritura.
- **Géneros y duplicados.** Una serie admite hasta 50 géneros. El guardado reutiliza los nombres normalizados y conserva los géneros para otras series. Los títulos duplicados se advierten y pueden guardarse como registros distintos.
- **Persistencia.** La migración 0002 añade series, géneros y relaciones mediante una transacción y conserva la configuración del esquema anterior. El diagnóstico de la Fase 0 sigue visible y muestra el esquema 2.
- **Portadas.** Rust valida el contenido JPEG, PNG o WebP y genera una miniatura WebP de hasta 360 × 540 píxeles junto con la copia administrada. El protocolo de Tauri entrega únicamente miniaturas por identificador. Al reemplazar o quitar una portada, el catálogo se confirma antes de limpiar la copia anterior. Un cierre inesperado o un archivo administrado bloqueado puede dejar una copia sin referencia hasta su limpieza.

| Comprobación en Windows x64 | Resultado |
| --- | --- |
| Lint, TypeScript, Vitest y build de Vite | Correctos. 5 pruebas de interfaz aprobadas |
| Formato Rust, pruebas con Cargo bloqueado al lockfile y Clippy | Correctos. 10 pruebas aprobadas, incluida la migración 1 → 2, errores de guardado y copias administradas |
| WebView2 con IPC real y perfil temporal | Alta, edición, borrado confirmado, géneros, duplicados, favoritos, portadas por selector y arrastre, conservación de originales y persistencia tras reiniciar verificados |
| Build nativo sin instalador | Ejecutable de producción en `src-tauri/target/release/mylist.exe` |

La autorización para la Fase 2 llegó después de esta verificación.

## 2026-11-21 · Entorno de desarrollo y verificación nativa

Se completó la instalación de Visual Studio Build Tools 2022 17.14.45 con MSVC v143 y el SDK de Windows 10.0.26100. Rust y Cargo 1.100.0, rustfmt y Clippy quedaron instalados en el perfil del usuario. Se descargaron las dependencias de Cargo para Windows x64 y se verificaron las dependencias instaladas de npm. Tauri detectó todos los prerrequisitos, incluido WebView2.

Durante el arranque nativo, una solicitud de CSS quedaba pendiente en Vite. Se declaró una configuración PostCSS vacía en `vite.config.ts` para omitir la búsqueda automática de configuración. La interfaz cargó al iniciar con esta configuración. [Opciones de CSS de Vite](https://vite.dev/config/shared-options#css-postcss)

| Comprobación | Resultado |
| --- | --- |
| `npm.cmd run tauri info` | Entorno Windows x64 completo |
| `npm.cmd run lint`, `npm.cmd run typecheck`, `npm.cmd test` y `npm.cmd run build` | Correctos. Pasaron las 3 pruebas de interfaz |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` y `cargo test --locked` | Correctos. Pasaron las 5 pruebas nativas, incluida la reapertura con datos conservados |
| Ventana de MyList en desarrollo | WebView2 cargó la biblioteca y mostró **SQLite disponible · Esquema 1** mediante IPC real. Se comprobó la base de un perfil temporal |
| `npm.cmd run tauri build -- --no-bundle` | Correcto. Ejecutable en `src-tauri/target/release/mylist.exe` |

La autorización para la Fase 1 llegó después de esta verificación.

## 2026-11-21 · Fase 0

Implementación base completada. La primera verificación quedó limitada por los prerrequisitos de compilación en Windows.

- Plantilla oficial `create-tauri-app` 4.7.5 para React y TypeScript adaptada a MyList. Se incluyeron ambos lockfiles y un icono provisional propio. El identificador de aplicación es `com.mylist.desktop`.
- Interfaz oscura y adaptable con biblioteca vacía, barra preparada para pestañas internas, navegación en español y secciones futuras identificadas como pendientes.
- SQLite incluido mediante rusqlite. La migración `0001_settings.sql` crea la tabla `settings`. Cada conexión activa claves foráneas y las migraciones pendientes se ejecutan en una transacción. Las versiones incompatibles devuelven un error.
- El comando `storage_health` comprueba que se pueda abrir la base y leer el esquema. La ventana muestra el estado, admite reintentos ante un error y mantiene las rutas de almacenamiento dentro de Rust.

### Verificación

Equipo Windows x64, compilación 26200, con Node.js 26.13.1, npm 11.20.0 y WebView2 157.0.4341.60. Rust y Cargo 1.100.0 se ejecutaron desde una caché local del proyecto para generar `Cargo.lock` y verificar el formato.

| Comprobación | Resultado |
| --- | --- |
| `npm.cmd ci --offline --no-audit` | Correcto con la caché local y el lockfile |
| `npm.cmd run lint` y `npm.cmd run typecheck` | Correctos |
| `npm.cmd test` | 3 pruebas aprobadas con IPC simulado. Cubren estado vacío, diagnóstico correcto, error sin detalles privados y reintento |
| `npm.cmd run build` | Correcto. Cliente de producción en `dist/` |
| Edge en Windows con el cliente compilado e IPC simulado | Correcto a 1180, 760 y 360 px de ancho, sin desbordamiento horizontal ni solicitudes externas |
| Migración ejecutada con SQLite 3.50.4 mediante Python | Correcta en una base temporal. Se verificaron la versión, la conservación de datos al reabrir, las claves foráneas, la restricción JSON y la integridad |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Correcto |
| `cargo test --locked` y `cargo clippy --locked` | Bloqueados por la ausencia de `link.exe`. Se añadieron cinco pruebas Rust para inicialización, reapertura, datos inválidos y fallos de apertura o migración |
| `npm.cmd run tauri dev` y `npm.cmd run tauri build -- --no-bundle` | Vite arrancó y compiló el cliente. La compilación nativa se detuvo por ausencia de `link.exe`. La ventana y el IPC real quedan pendientes |

### Pendientes de la primera verificación

1. Completar los [prerrequisitos de Windows](../README.md#requisitos-de-desarrollo-en-windows), ejecutar las verificaciones Rust y compilar el ejecutable. Abrir la ventana MyList, comprobar el diagnóstico IPC real y verificar la reapertura.
2. Con autorización del usuario, comenzar la Fase 1 con una migración para series y géneros. Implementar el catálogo, la gestión de portadas, favoritos y las pruebas de persistencia al reiniciar.

Continúa vigente el [roadmap aprobado](../PROJECT_CONTEXT.md#9-roadmap-y-criterios-de-aceptación).

| Fase | Entrega pendiente |
| --- | --- |
| 1 | Series, géneros, portadas y favoritos (entregados) |
| 2 | Contenidos asociados, categorías y orden (entregados) |
| 3 | Progreso, sesiones, estados, calificaciones y promedios (entregados) |
| 4 | Búsqueda, filtros, orden y preferencias (entregados) |
| 5 | Administración de géneros y categorías, pulido visual, temas, pestañas y accesibilidad (entregados) |
| 5.5 | Cabecera cinematográfica y detalle individual (entregados) |
| 6 | Estadísticas y Continuar viendo (entregados) |
| 6.5 | Refinamiento del dashboard de estadísticas (entregado) |
| 7 | Respaldos, restauración, exportación e idioma (entregados) |
| 8 | QA, instalación y actualización de Windows |
