# gz-claude - Documento de Diseño

> Binario Rust que orquesta Zellij + Web Client + Claude Code con un panel de Workspaces.

## Objetivo

Al ejecutar `gz-claude`:

1. Se abre Zellij con un layout predefinido
2. Se levanta opcionalmente el Web Client de Zellij (solo red local)
3. Panel izquierdo: TUI de Workspaces con navegación drill-down
4. Panel central: panes de terminal dinámicos (Claude, Bash, etc.)

---

## Arquitectura General

### Binario único `gz-claude`

Modos de ejecución:

```
gz-claude              → Inicia Zellij con el layout, opcionalmente web client
gz-claude panel        → Corre dentro de Zellij, renderiza el TUI
gz-claude --web        → Override para forzar web client
gz-claude --no-web     → Override para deshabilitar web client
```

### Flujo de arranque

1. `gz-claude` (sin argumentos) valida la configuración
2. Si hay paths inválidos → error con mensaje claro y exit code 1
3. Si todo OK → genera/actualiza el layout KDL en `~/.config/zellij/layouts/`
4. Lanza `zellij --layout=gz-claude`
5. Opcionalmente lanza el web server según config/flags
6. Zellij ejecuta `gz-claude panel` en el pane izquierdo

### Dependencias Rust

- `ratatui` - TUI framework
- `crossterm` - terminal backend
- `tokio` - async runtime (para procesos externos)
- `toml` + `serde` - parsing de config
- `git2` - información de Git nativa (sin shell out)
- `clap` - CLI argument parsing

### Estructura de crates

```
gz-claude/
├── src/
│   ├── main.rs
│   ├── cli.rs          # clap args
│   ├── config/         # parsing y validación
│   ├── tui/            # componentes ratatui
│   ├── zellij/         # interacción con zellij CLI
│   └── git/            # wrappers git2
```

---

## Configuración

### Archivo principal

`~/.config/gz-claude/config.toml`

```toml
# Acciones globales (disponibles en todos los proyectos)
[global]
editor = "$EDITOR"  # comando para abrir archivos, default $EDITOR
git_info_level = "minimal"  # minimal | standard | detailed

[global.actions]
c = { name = "Claude", command = "claude", icon = "🤖" }
b = { name = "Bash", command = "bash", icon = "💻" }
g = { name = "Lazygit", command = "lazygit", icon = "󰊢" }

[web_client]
auto_start = false
bind_address = "0.0.0.0"  # o IP específica
port = 8082
# token se genera automáticamente y se guarda en ~/.config/gz-claude/web_token

# Workspaces
[workspace.fanki]
name = "Fanki"

[workspace.fanki.actions]
t = { name = "Tests", command = "mvn test", icon = "🧪" }
d = { name = "Deploy", command = "make deploy", icon = "🚀" }

[[workspace.fanki.projects]]
name = "API Gateway"
path = "/Users/emiliano/dev/fanki/api-gateway"

[[workspace.fanki.projects]]
name = "Payments"
path = "/Users/emiliano/dev/fanki/payments"
[workspace.fanki.projects.actions]
t = { name = "Tests", command = "gradle test", icon = "🧪" }  # override del workspace
s = { name = "Swagger", command = "make swagger", icon = "📋" }  # acción extra
```

### Resolución de acciones (herencia)

1. Se cargan acciones globales
2. Se mergean acciones del workspace (override por key)
3. Se mergean acciones del proyecto (override por key)

### Validación al iniciar

- Todos los paths deben existir y ser directorios
- Keys de acciones deben ser un único caracter
- Comandos no pueden estar vacíos

---

## TUI - Navegación y Vistas

### Navegación jerárquica (drill-down)

- Vista 1: Lista de workspaces → Enter entra al workspace
- Vista 2: Lista de proyectos del workspace → Enter entra al proyecto (file browser)
- Vista 3: Git info + file tree del proyecto + acciones
- Backspace/Esc para volver atrás

### Vista 1: Workspaces

```
┌─ gz-claude ─────────────────────┐
│                                 │
│  Workspaces                     │
│                                 │
│  > Fanki                        │
│    Helios                       │
│    Personal                     │
│                                 │
├─────────────────────────────────┤
│ Enter: select  q: quit          │
└─────────────────────────────────┘
```

### Vista 2: Proyectos del workspace

```
┌─ Fanki ─────────────────────────────┐
│                                     │
│  Projects                           │
│                                     │
│  > API Gateway      main *  🤖 💻   │
│    Payments         develop 🤖 💻   │
│    Tickets          main    🤖 💻   │
│                                     │
├─────────────────────────────────────┤
│ Enter: browse  🤖c:Claude  💻b:Bash │
│ Esc: back  q: quit                  │
└─────────────────────────────────────┘
```

- `Enter` en un proyecto: entra a Vista 3 (file browser del proyecto)
- `c` con proyecto seleccionado: abre Claude en nuevo pane con `cwd = project.path`
- `b` con proyecto seleccionado: abre Bash en nuevo pane con `cwd = project.path`
- Los iconos son configurables con emojis por default

### Vista 3: File Browser del proyecto

```
┌─ API Gateway ───────────────────────┐
│ main * │ +2 -1 │ 3 staged           │
├─────────────────────────────────────┤
│  > src/                             │
│      main/                          │
│      test/                          │
│    pom.xml                          │
│    README.md                        │
├─────────────────────────────────────┤
│ 🤖c 💻b 󰊢g 🧪t │ Enter: open/expand │
│ Esc: back                           │
└─────────────────────────────────────┘
```

- `Enter` en carpeta: expande/colapsa
- `Enter` en archivo: abre en nuevo pane con `$EDITOR`
- Las acciones configuradas siguen disponibles

### Controles

- `↑/↓` o `j/k`: navegar
- `Enter`: seleccionar / abrir archivo / expandir carpeta
- `Esc` o `Backspace`: volver atrás
- `r`: refrescar git info
- `q`: salir (solo en Vista 1)
- Teclas de acciones: ejecutan el comando en nuevo pane

### Git Info Levels

Configurable con `git_info_level`:

- **minimal**: Branch actual + indicador dirty (`main *`)
- **standard**: Branch + dirty + ahead/behind + staged/unstaged count
- **detailed**: Todo lo anterior + lista de archivos modificados

---

## Integración con Zellij

### Generación del layout

Al ejecutar `gz-claude`, se genera `~/.config/zellij/layouts/gz-claude.kdl`:

```kdl
layout {
    pane size=1 borderless=true {
        plugin location="zellij:tab-bar"
    }

    pane split_direction="vertical" {
        pane size=40 {
            command "gz-claude"
            args ["panel"]
        }
        pane focus=true {
            command "bash"
        }
    }

    pane size=1 borderless=true {
        plugin location="zellij:status-bar"
    }
}
```

### Acciones desde el TUI

Abrir pane con comando:

```bash
zellij action new-pane --cwd "/path/al/proyecto" -- claude
```

Abrir archivo con editor:

```bash
zellij action new-pane --cwd "/path/al/proyecto" -- $EDITOR archivo.rs
```

### Web Client

Si `web_client.auto_start = true` o se usa `--web`:

```bash
zellij web --listen 192.168.1.100:8082
```

El token se genera una vez y se guarda en `~/.config/gz-claude/web_token`. El TUI muestra la URL completa cuando el web server está activo.

### Detección de Zellij

- `gz-claude panel` verifica que corre dentro de Zellij (variable `ZELLIJ`)
- Si no está en Zellij, muestra error y sugiere ejecutar `gz-claude` sin argumentos

---

## Validación y Manejo de Errores

### Al iniciar `gz-claude`

1. **Buscar config:** `~/.config/gz-claude/config.toml`
   - Si no existe → crear config de ejemplo y mostrar mensaje

2. **Validar estructura TOML:**
   - Syntax errors → mostrar línea y columna del error

3. **Validar paths de proyectos:**
   ```
   Error: Invalid configuration

   The following project paths do not exist:

     • Fanki / API Gateway
       /Users/emiliano/dev/fanki/api-gateway

     • Helios / Backend
       /Users/emiliano/dev/helios/backend

   Please fix these paths in ~/.config/gz-claude/config.toml
   ```

4. **Validar acciones:**
   - Keys duplicadas en mismo nivel → error
   - Key no es un solo caracter → error
   - Comando vacío → error

5. **Validar Zellij instalado:**
   ```
   Error: Zellij not found

   gz-claude requires Zellij to be installed.
   Install it from: https://zellij.dev/documentation/installation
   ```

### Dentro del TUI

- Errores de git → mostrar `[git error]` en lugar de branch
- Error al ejecutar acción → mostrar notificación temporal en el TUI
- Refresh (`r`) falla → mostrar mensaje, mantener último estado conocido

---

## Plan de Implementación

### Etapa 0: Bootstrap del proyecto
- Crear `Cargo.toml` con dependencias
- Estructura de directorios
- CLI básico con clap (`gz-claude`, `gz-claude panel`)

### Etapa 1: Configuración
- Structs de config con serde
- Parsing de `config.toml`
- Validación completa (paths, acciones, keys)
- Generación de config de ejemplo
- Tests unitarios de parsing y validación

### Etapa 2: Git
- Wrapper sobre `git2`
- Obtener: branch, dirty status, ahead/behind, staged/unstaged count
- Niveles de detalle configurables
- Tests con repos de prueba

### Etapa 3: TUI
- Setup ratatui + crossterm
- Vista 1: Workspaces
- Vista 2: Proyectos con iconos
- Vista 3: File browser
- Navegación drill-down
- Barra de acciones dinámica

### Etapa 4: Integración Zellij
- Generar layout KDL
- Ejecutar `zellij action new-pane`
- Abrir archivos con editor
- Detectar entorno Zellij

### Etapa 5: Web Client
- Gestión del token
- Arranque condicional del web server
- Mostrar URL en TUI cuando activo

Cada etapa termina con funcionalidad testeable y commit.
