# MyList — Contexto del proyecto

Versión inicial de la especificación. Plataforma objetivo: Windows 10/11. Enfoque: equilibrio entre simplicidad, calidad y extensibilidad.

## 1. Visión y alcance

MyList es una biblioteca multimedia personal de escritorio, instalable y completamente local. Permite documentar animes, series y franquicias, agrupar contenidos relacionados, registrar avances y valorar cada contenido. No reproduce vídeo ni obtiene metadatos automáticamente. No requiere cuentas, servidor HTTP, alojamiento web ni conexión permanente. Todos los títulos, imágenes, categorías y géneros son introducidos por el usuario.

## 2. Requerimientos funcionales aprobados

| ID | Requerimiento |
|---|---|
| RF-01 | Crear, editar y eliminar series principales. |
| RF-02 | Importar, sustituir y quitar portada principal. |
| RF-03 | Mostrar portada provisional si falta una imagen. |
| RF-04 | Crear géneros personalizados reutilizables. |
| RF-05 | Asignar múltiples géneros a una serie. |
| RF-06 | Advertir sobre títulos duplicados y permitir continuar. |
| RF-07 | Eliminar serie y contenido asociado solo tras confirmación explícita. |
| RF-08 | Mostrar catálogo en cuadrícula de portadas. |
| RF-09 | Marcar y desmarcar favoritas. |
| RF-10 | Filtrar favoritas. |
| RF-11 | Crear, editar y eliminar contenidos asociados. |
| RF-12 | Crear categorías personalizadas reutilizables, sin categorías multimedia predefinidas. |
| RF-13 | Reordenar contenido mediante arrastrar y soltar. |
| RF-14 | Portadas individuales opcionales con sustituto provisional. |
| RF-15 | Descripciones de texto libre multilínea. |
| RF-16 | Contenido en tarjetas horizontales. |
| RF-17 | Año de estreno opcional por contenido. |
| RF-18 | Opinión personal independiente de la descripción. |
| RF-19 | Confirmar eliminación de contenido individual. |
| RF-20 | Calificación individual opcional de 1 a 10, con un decimal. |
| RF-21 | Promedio simple automático de contenidos calificados por serie. |
| RF-22 | Excluir contenidos sin nota del promedio. |
| RF-23 | Estados Por ver, Mirando y Finalizado. |
| RF-24 | Actualización automática del estado por progreso, con edición manual. |
| RF-25 | Botones +1/-1 y edición directa de episodios vistos. |
| RF-26 | Total de episodios desconocido permitido. |
| RF-27 | Fechas de inicio y finalización opcionales. |
| RF-28 | Calificar en cualquier estado. |
| RF-29 | Contabilizar visualizaciones completas, incluidas revisualizaciones. |
| RF-30 | Buscar por título de serie o contenido asociado. |
| RF-31 | Actualizar resultados mientras se escribe. |
| RF-32 | Seleccionar múltiples géneros con modo TODOS o CUALQUIERA. |
| RF-33 | Mostrar series con al menos un contenido en el estado seleccionado. |
| RF-34 | Orden por título, promedio, fecha de registro o última actualización. |
| RF-35 | Orden ascendente y descendente. |
| RF-36 | Combinar nombre, género, estado y favoritos. |
| RF-37 | Recordar filtros y orden entre sesiones. |
| RF-38 | Filtrar contenidos dentro de una serie por categoría y estado. |
| RF-39 | Restablecer filtros con una acción. |
| RF-40 | Estética cinematográfica. |
| RF-41 | Tema claro y oscuro. |
| RF-42 | Navegación lateral. |
| RF-43 | Pestañas internas para abrir varias series. |
| RF-44 | Formularios modales. |
| RF-45 | Cuadrícula adaptable al tamaño de ventana. |
| RF-46 | Selector de tamaño de portada. |
| RF-47 | Importar imágenes desde selector de archivos y arrastrar/soltar. |
| RF-48 | Estadísticas personales. |
| RF-49 | Sección Continuar viendo. |
| RF-50 | Persistir preferencias visuales. |
| RF-51 | Almacenamiento local sin cuenta ni Internet. |
| RF-52 | Copiar imágenes a directorio propio. |
| RF-53 | Persistir automáticamente cambios confirmados. |
| RF-54 | Exportar/importar respaldo ZIP completo. |
| RF-55 | Respaldos manuales y automáticos. |
| RF-56 | Conservar las últimas cinco copias automáticas. |
| RF-57 | Instalador Windows. |
| RF-58 | Icono, acceso directo y desinstalador. |
| RF-59 | Exportar catálogo CSV y Markdown. |
| RF-60 | Idioma español/inglés seleccionable. |
| RF-61 | Conservar datos y preferencias al actualizar. |
| RF-62 | Consultar en Configuración los géneros, las categorías y la cantidad de series o contenidos que usan cada uno. |
| RF-63 | Crear y renombrar géneros y categorías en Configuración con nombres válidos y únicos dentro de cada grupo. |
| RF-64 | Eliminar géneros tras confirmar su desvinculación de las series. |
| RF-65 | Eliminar categorías tras confirmar y, si tienen contenidos, reasignarlos a otra categoría existente. |
| RF-66 | Conservar series, contenidos y sus asociaciones al renombrar un género o categoría. |
| RF-67 | Editar una descripción general opcional y multilínea de la serie y expandirla en la cabecera cuando sea extensa. |
| RF-68 | Importar, reemplazar y quitar un fondo panorámico local para la cabecera. Usar la portada como fondo alternativo y mantener nítida la portada en primer plano. |
| RF-69 | Mostrar en la cabecera el promedio de solo lectura con cinco estrellas de relleno proporcional, valor numérico y estado sin nota. Mostrar el recuento de contenidos por estado. |
| RF-70 | Abrir el detalle de un contenido dentro de la pestaña de la serie, con descripción, opinión, historial, fechas y acciones. Al volver, recuperar filtros y posición de desplazamiento. |
| RF-71 | Mejorar la jerarquía de las tarjetas horizontales con portadas un 20 % más anchas, seguimiento visible y acciones secundarias agrupadas, conservando filtros y orden manual. |
| RF-72 | Presentar estadísticas en una cuadrícula adaptable con indicadores compactos, donas de estado y categoría, barras de género e histograma de calificación. |
| RF-73 | Explorar desde los gráficos los resultados de Biblioteca y, cuando corresponda, los contenidos coincidentes. Conservar los filtros guardados al explorar. |
| RF-74 | Mostrar el ranking con portadas locales, posición y estrellas proporcionales. Abrir la serie en su pestaña al seleccionar una tarjeta. |
| RF-75 | Mostrar una vista previa del respaldo validado y crear una copia preventiva antes de sustituir el catálogo. |
| RF-76 | Activar o desactivar respaldos automáticos, consultar la fecha del último éxito y abrir su carpeta. |

## 3. Reglas de negocio y decisiones operativas propuestas

Estas reglas concretan los casos límite de la especificación. Si se requiere cambiar una decisión, documentarlo antes de implementar el módulo afectado.

- Serie: título obligatorio no vacío; descripción general opcional de hasta 10 000 caracteres; portada y fondo panorámico opcionales e independientes; cero o más géneros; favorito booleano. Nunca editar manualmente su promedio. Una serie puede existir sin contenidos. Advertir de duplicados usando título normalizado; cada serie mantiene su propio identificador.
- Contenido: pertenece a exactamente una serie; título y una categoría personalizada obligatorios; descripción, opinión, portada, año, nota y fechas opcionales. El orden pertenece a cada serie. No confundir categoría con género. Al crear una categoría o género existente con nombre normalizado, reutilizarlo. No eliminar una categoría en uso sin reasignar sus contenidos; eliminar género solo tras confirmar la desvinculación.
- Administración en Configuración: el alta o renombrado explícito rechaza nombres vacíos y duplicados según la normalización existente. Renombrar conserva el identificador y todas las relaciones. El borrado exige confirmación. La desvinculación de un género o la reasignación a una categoría existente se ejecuta en la misma transacción que el borrado del término. Estas operaciones conservan las series y sus contenidos.
- Calificación: almacenar enteros en décimas (10–100), NULL si no se ha calificado; promedio simple sin ponderación, mostrar hasta dos decimales, sin nota si no hay calificaciones. Ejemplo: 9, 7 y 10 => 8.67.
- Progreso: episodios vistos entero >=0; total entero >=1 o NULL; nunca exceder total conocido. Películas pueden tener total 1. Total desconocido muestra contador sin porcentaje. No reducir total por debajo del avance sin corrección explícita.
- Estado automático: 0 => Por ver; avance positivo parcial o total desconocido => Mirando; avance igual al total conocido => Finalizado. Se permite edición manual; marcar Finalizado con total conocido completa el avance, con total desconocido conserva el contador; marcar Por ver solicita confirmar reinicio si hay avance; marcar Mirando no incrementa episodios. Una edición explícita del progreso recalcula el estado. Evitar cambios silenciosos incoherentes; el estado manual puede diferir hasta la siguiente edición del progreso.
- Visualizaciones: una sesión de visualización activa por contenido; finalizarla la cuenta una sola vez. Reabrir por corrección modifica esa misma sesión, no crea otra. Acción explícita 'Volver a ver' inicia sesión nueva y conserva sesiones anteriores. No contar dos veces por alternar estados. Fechas opcionales, inicio <= finalización si ambas existen.
- Búsqueda: insensible a mayúsculas y tildes; coincidencia parcial. Coincidencia en título hijo muestra su serie una sola vez. Combinar grupos de filtros con AND; géneros con AND/OR seleccionable. El estado de catálogo corresponde a existencia de algún hijo en estado elegido; serie sin hijos no coincide. Orden estable con desempate por ID; notas inexistentes al final.
- Guardado: confirmación de formulario persiste en transacción; cancelar no cambia datos. Acciones rápidas persisten inmediatamente. Mostrar error y conservar estado anterior si falla. Fechas de creación/actualización automáticas; cambios de contenido actualizan la marca de tiempo de su serie.
- Eliminación: confirmar cascada antes de borrar serie; limpiar únicamente imágenes administradas sin referencias. Nunca borrar la imagen original del usuario. Pestañas abiertas deben reflejar edición/eliminación. No pedir confirmación para cerrar una pestaña sin cambios pendientes.
- Respaldos: se comprueban al iniciar y periódicamente durante el uso. Si han pasado 24 horas desde el último éxito, se verifica una nueva copia antes de conservar las cinco más recientes. Los respaldos manuales y preventivos quedan fuera de esta retención. La restauración valida el manifiesto y el esquema, requiere confirmación explícita y crea un respaldo preventivo. La publicación usa una transacción SQLite con las imágenes ya instaladas. ZIP incluye un snapshot consistente, imágenes, preferencias y manifiesto con SHA-256.

## 4. Stack aprobado

- Tauri 2, React, TypeScript, Vite; Rust para lógica local; SQLite mediante una biblioteca Rust mantenida (propuesta: rusqlite con SQLite incluido). Dependencias fijadas con lockfiles; no agregar frameworks innecesarios.
- Windows utiliza WebView2 para presentar la interfaz local; no hay despliegue web ni servidor en producción. Empaquetar instalador con herramientas de Tauri y proveer opción de instalación offline del runtime cuando corresponda.
- CSS con tokens de diseño y componentes accesibles. React state/context inicialmente; agregar gestión de estado externa solo si se justifica. i18n con diccionarios es/en y claves estables. Pruebas: Rust unitarias/integración, Vitest/Testing Library para UI.
- Referencias oficiales: https://v2.tauri.app/ ; https://react.dev/ ; https://www.sqlite.org/ ; https://www.typescriptlang.org/ .

## 5. Arquitectura

Monolito modular por capas. La UI llama comandos Tauri tipados; estos validan entrada y delegan a servicios. Los servicios aplican reglas de negocio y coordinan repositorios/transacciones. El dominio no depende de React ni SQLite. Infraestructura maneja archivos, respaldos y exportaciones. No crear REST, microservicios ni autenticación.

```text
React UI -> comandos Tauri -> servicios -> dominio/repositorios -> SQLite
                                     -> infraestructura -> archivos locales
```

```text
MyList/
  src/                  # React + TypeScript
    components/          # UI reutilizable
    features/            # library, series, content, favorites, statistics, settings
    services/            # cliente tipado de comandos IPC
    hooks/
    types/
    locales/             # es / en
    styles/
  src-tauri/
    src/
      commands/
      domain/
      services/
      repositories/
      infrastructure/
    migrations/
    icons/
  tests/
  docs/
  AGENTS.md
  PROJECT_CONTEXT.md
  README.md
```

Evitar capas vacías excesivas: implementar archivos según necesidad. Los contratos IPC devuelven errores estructurados; no exponer consultas SQL o rutas arbitrarias a la UI. Permisos Tauri mínimos. No bloquear UI con tareas largas (importar imagen, respaldar). Las rutas reales se resuelven usando el directorio de datos de aplicación de Tauri, jamás el directorio de instalación.

## 6. Modelo lógico propuesto

| Entidad | Campos principales |
|---|---|
| Series | id, title, description, cover_key nullable, panorama_key nullable, favorite, created_at, updated_at |
| Genre | id, name, normalized_name unique |
| SeriesGenre | series_id, genre_id (clave compuesta) |
| Category | id, name, normalized_name unique |
| Content | id, series_id, category_id, title, cover_key nullable, description, opinion, release_year nullable, rating_tenths nullable, total_episodes nullable, sort_order, created_at, updated_at |
| WatchSession | id, content_id, watched_episodes, status, started_on nullable, finished_on nullable, completion marker, active, activity_at nullable |
| Settings | key, value (JSON validado según esquema) |

Relaciones: Series 1:N Content; Series N:M Genre; Category 1:N Content; Content 1:N WatchSession. Estado y avance actuales se obtienen de sesión activa; conteo de visualizaciones deriva de sesiones completadas. Usar IDs estables; claves foráneas habilitadas; índices para relaciones, títulos normalizados y búsquedas relevantes. Migraciones numeradas, transaccionales y no destructivas; no recrear base en arranque. Los campos calculados no se almacenan innecesariamente.

## 7. UX

Sidebar: Biblioteca, Favoritos, Continuar viendo, Estadísticas, Configuración. Biblioteca: búsqueda instantánea, filtros compactos, orden, tamaño de tarjeta, botones de añadir; tarjetas 2:3 con nombre, géneros y nota promedio. Vista de serie: portada y encabezado, pestaña reutilizable por ID, lista de tarjetas horizontales con título, categoría, año, estado, nota, progreso y acciones. Modales de edición con validación y cancelación; detalle de texto largo accesible. Arrastrar y soltar permite importar portada y reordenar contenido con alternativa de teclado. Tema claro/oscuro, guardar idioma y tamaño; contraste, foco visible, escalado DPI y estados vacío/carga/error. Estética cinematográfica sobria, no copiar marcas ni incorporar imágenes de terceros de forma automática.

Estadísticas: totales de series, contenidos y favoritos, episodios y visualizaciones completas, promedio de contenidos calificados, distribuciones por estado, género, categoría y nota, y las diez series mejor calificadas. Los episodios vistos suman cada sesión una vez y distinguen el aporte de las sesiones activas y anteriores. Cada serie con varios géneros cuenta una vez en cada género asignado. Continuar viendo enumera sesiones activas Mirando, ordenadas por el último cambio de estado o avance, y permite ajustar episodios y abrir el detalle en su pestaña de serie.

La cabecera de serie presenta la portada a la izquierda. A la derecha muestra título, géneros, descripción general, estrellas, recuentos de los estados actuales y acciones de edición y favorito. El fondo panorámico se difumina hasta el color de la vista en ambos temas. El detalle individual se abre en la misma pestaña y concentra los textos completos, el seguimiento y las fechas de cada sesión.

## 8. Datos locales y distribución

Directorio de aplicación: database.sqlite3, covers/, backups/, archivos temporales de operaciones. Copiar imágenes JPEG/PNG/WebP validadas usando nombres únicos y rutas relativas; límites razonables de tamaño y resolución; miniaturas/cache regenerables. No conservar archivos originales como referencias externas. Exportación CSV con UTF-8 y escape correcto, Markdown legible con series y contenidos; no exportar rutas absolutas privadas. El instalador incluye icono y desinstalador; actualizaciones mantienen el directorio de datos y ejecutan migraciones. El desinstalador no borra catálogo por defecto.

La entrega 1.0.0 usa **MyList** como nombre de producto y conserva el identificador `com.mylist.desktop`. En Windows, el catálogo se guarda en `%APPDATA%\com.mylist.desktop` para el usuario actual. Se mantiene el esquema SQLite 6 y la compatibilidad con los respaldos ZIP de la Fase 7. La entrega local usa un instalador NSIS x64 por usuario, con accesos en el escritorio y el menú Inicio, y WebView2 Evergreen incluido para instalación sin conexión. El ejecutable directo conserva la ruta `src-tauri/target/release/mylist.exe`.

## 9. Roadmap y criterios de aceptación

| Fase | Entrega verificable |
|---|---|
| 0 | Scaffold, shell UI, SQLite y migración inicial, comando IPC de salud, README, pruebas y compilación donde el entorno permita. |
| 1 | CRUD series, géneros, portadas y favoritos; datos sobreviven reinicio. |
| 2 | CRUD contenidos, categorías reutilizables, portadas, orden persistente, descripción/opiniones/año. |
| 3 | Progreso, sesiones, estados manuales y automáticos, puntuaciones y promedio; pruebas de límites. |
| 4 | Búsqueda combinada, filtros, orden y preferencias persistentes; MVP funcional. |
| 5 | Administración de géneros y categorías, pulido cinematográfico, pestañas, temas, tamaño de tarjeta y accesibilidad. |
| 5.5 | Cabecera cinematográfica con descripción general y panorama local, estrellas proporcionales y detalle individual con retorno a filtros y desplazamiento. |
| 6 | Estadísticas y Continuar viendo. |
| 6.5 | Refinamiento del dashboard con gráficos interactivos, ranking visual y navegación con filtros temporales. |
| 7 | ZIP respaldo/restauración, retención, CSV/Markdown, idioma bilingüe. |
| 8 | Auditoría final, icono provisional, instalador Windows x64 de MyList 1.0.0, verificación de instalación, actualización y desinstalación con datos aislados y guía de recuperación. |

Cada fase requiere pruebas positivas y negativas, actualización de documentación y reporte de cambios. No avanzar automáticamente sin aprobación. Validar instalador en Windows real antes de dar por cumplido RF-57/58/61.

## 10. Criterios no funcionales

- Operación offline, privacidad local y ausencia de telemetría por defecto.
- Inicio y navegación fluidos; evitar cargar todas las portadas originales simultáneamente, usar miniaturas y carga diferida.
- Integridad transaccional de datos y limpieza segura de archivos; respaldos comprobables.
- Arquitectura comprensible, módulos cohesivos, pruebas reproducibles y documentación actualizada.
- No incluir secretos, bases personales, respaldos, imágenes personales ni artefactos de compilación en git.
