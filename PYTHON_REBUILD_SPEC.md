# Python Rebuild Specification for HARetroPanel

## Objective

Rebuild the existing HARetroPanel application in Python while preserving the same behavior, operational assumptions, and old-device compatibility. The Python version must remain a lightweight Home Assistant dashboard optimized for older tablets and browsers, especially iOS 9-era Safari.

This spec is the source of truth for a developer AI or implementation agent. The code should match the Rust behavior described here, not a newer or more modern interpretation unless explicitly noted.

---

## Project Goal

HARetroPanel is a small web dashboard that:

- reads Home Assistant entity state from the HA API
- filters and pages entities based on user-selected settings
- displays a simple server-rendered dashboard UI
- lets users toggle switches and scripts
- lets users adjust light brightness, color temperature, and RGB color
- is designed to work on older hardware and older browsers
- stores layout configuration in a local JSON file
- runs as a simple process with minimal dependencies

The Python version should preserve the current Rust app’s contract and likely runtime behavior.

---

## Core Constraints

### 1. Old-device compatibility is a primary requirement

The UI must remain compatible with older browsers such as Safari 9 / iOS 9.3.5.

Rules:

- Prefer server-rendered HTML over client-heavy JS
- Avoid modern JavaScript APIs such as `requestSubmit()`, `Array.from()` if not necessary, `const/let` in the page script if older compatibility is expected, and pointer events-only logic
- Prefer simple DOM + inline event handlers that work on old browsers
- For sliders, use HTML range inputs only if the old browser supports them reliably; if compatibility is uncertain, prefer a simple native select control rather than a custom JS-driven UI
- Keep CSS compatible with older WebKit and flexbox support

### 2. Minimal footprint

- No database requirement
- No Redis or external runtime services
- Local file storage for layout state only
- Single-process web server

### 3. Home Assistant integration

- Must authenticate using a Home Assistant bearer token from env config
- Must call Home Assistant services for state-changing actions
- Must parse the HA state payload from `/api/states`

---

## Existing Behavior to Preserve

The Python version should match the Rust implementation closely. The following behavior is required.

### Entity kinds supported

The app treats entities as these categories:

- `Light`
- `Switch`
- `Sensor`
- `Climate`
- `Script`

Entity classification is based on the Home Assistant entity ID prefix:

- `light.` => Light
- `switch.` => Switch
- `climate.` => Climate
- `script.` => Script
- everything else => Sensor

### Entity state mapping

For each entity returned by HA, the app derives:

- `id`
- `name` (friendly name if available, otherwise entity_id)
- `kind`
- `is_on`
- `value`
- `brightness` (only for lights)
- `color_temp_kelvin`, `min_color_temp_kelvin`, `max_color_temp_kelvin` (light only)
- `rgb_color` (light only)

Home Assistant state logic from Rust:

- Light / Switch: considered on when state is `"on"`
- Climate: considered on when state is one of `heat`, `cool`, `heat_cool`, `auto`
- Sensor: considered on when state is one of `on`, `open`, `home`, `above_horizon`
- Script: considered on when state is `"on"`

Value formatting:

- Sensor / Climate: `"{state} {unit}"` if a unit is present, otherwise plain `state`
- Script: plain `state`
- Light / Switch: no value shown

### Brightness conversion

Brightness from HA is stored as a 0..255 value. The UI calculates a percent like this:

- `brightness_pct = ((brightness * 100 + 127) / 255)` rounded down to integer
- Equivalent Rust logic: `((value as u16 * 100 + 127) / 255) as u8`

This produces percentage values roughly in the range 0..100.

### Service calls

The app must call Home Assistant services using the HA REST API.

#### Toggle

- Endpoint: `POST /toggle`
- For entity IDs such as `light.foo`, `switch.foo`, `script.foo`, etc.
- Calls: `/{domain}/toggle` with JSON payload `{ "entity_id": "..." }`

#### Brightness

- Endpoint: `POST /brightness`
- For light entities only
- Calls: `light/turn_on` with payload:

```json
{
  "entity_id": "light.foo",
  "brightness_pct": 55
}
```

#### Color Temperature

- Endpoint: `POST /light/color_temp`
- For light entities only
- Calls: `light/turn_on` with payload:

```json
{
  "entity_id": "light.foo",
  "color_temp_kelvin": 3500
}
```

#### RGB Color

- Endpoint: `POST /light/color`
- For light entities only
- Parses a hex color string and sends `rgb_color` as `[r, g, b]`

#### Run Script

- Endpoint: `POST /run_script`
- For script entities only
- Calls: `script/turn_on` with `{ "entity_id": "script.foo" }`

---

## HTTP Endpoints

The Python app must provide the same routes as the Rust app.

### Public views

- `GET /` - Dashboard page
- `GET /settings/entities` - Settings page

### POST actions

- `POST /toggle`
- `POST /brightness`
- `POST /light/color_temp`
- `POST /light/color`
- `POST /run_script`
- `POST /settings/entities`

### Redirect behavior

The Rust app intentionally redirects GET requests to the same root path for action routes, e.g.:

- `GET /brightness` redirects to `/`
- `GET /toggle` redirects to `/`
- `GET /light/color_temp` redirects to `/`
- `GET /run_script` redirects to `/`

The Python version must preserve that behavior.

### WebSocket

- `GET /ws?page=<page>` for dashboard refresh stream
- Sends JSON snapshots every 5 seconds
- Snapshot structure must match the dashboard view model JSON enough for the frontend update logic to work

---

## Dashboard Rendering Behavior

The dashboard is a server-rendered page based on Askama templates in Rust; Python should use Jinja2 templates or equivalent.

### Requirements

- Show each entity as a card
- Each card has:
  - title
  - kind metadata
  - value or on/off status
  - toggle button for togglables
  - brightness control if light has brightness
  - color temperature slider/select if supported
  - color picker if RGB supported
  - run button for script entities

### Layout behavior

- Entities are filtered by a selected “visible” list
- Entities are assigned to pages (page numbers start at 1)
- The dashboard should display only entities assigned to the current page
- `total_pages` is computed from the highest assigned page number if any items exist
- If no page assignments exist, default to page 1 and show all entities

### Pagination

- UI includes previous / next links
- Current page info shown in the middle
- Routing supports `/?page=<n>` and `/?page=<n>&force_refresh=1`

### Force refresh

When `force_refresh=1` is present, the app must invalidate any cached state and reload fresh data before rendering.

---

## Settings Page Behavior

The settings page allows users to:

- choose which entities are visible on the dashboard
- assign each entity to a page

### Data stored

Two file-backed collections are persisted in the layout repository:

- visible entity IDs: `Vec<EntityId>`
- page assignment map: `HashMap<String, usize>`

The JSON repository file path is:

- `./data/dashboard_layout.json`

This file should be created automatically if it does not exist.

### Rules

- `visible` values are entity IDs from `name=value` form fields
- `page_<entity_id>` values are numeric and should be `>= 1`
- Save both visible IDs and page assignments on submit
- Redirect back to `/`

---

## Data Model

The Python app should keep the same logical model as the Rust app.

### Entities

```python
class EntityKind(Enum):
    LIGHT = "light"
    SWITCH = "switch"
    SENSOR = "sensor"
    CLIMATE = "climate"
    SCRIPT = "script"

class Entity:
    id: str
    name: str
    kind: EntityKind
    is_on: bool
    value: Optional[str]
    brightness: Optional[int]
    color_temp_kelvin: Optional[int]
    min_color_temp_kelvin: Optional[int]
    max_color_temp_kelvin: Optional[int]
    rgb_color: Optional[Tuple[int, int, int]]
```

### Dashboard state

```python
class DashboardState:
    entities: list[Entity]
```

### View model for UI

The UI layer needs a model equivalent to the Rust `EntityViewModel`:

```python
class EntityViewModel:
    id: str
    name: str
    kind_label: str
    is_on: bool
    has_value: bool
    value: str
    can_toggle: bool
    brightness_pct: Optional[int]
    supports_color_temp: bool
    color_temp_kelvin: Optional[int]
    min_color_temp_kelvin: Optional[int]
    max_color_temp_kelvin: Optional[int]
    supports_rgb: bool
    rgb_hex: str
    can_run_script: bool
```

---

## Home Assistant API Contracts

### GET /api/states

The app expects a list of HA state objects shaped like this:

```json
[
  {
    "entity_id": "light.lamp",
    "state": "on",
    "attributes": {
      "friendly_name": "Lamp",
      "brightness": 128,
      "color_temp_kelvin": 3000,
      "min_color_temp_kelvin": 2200,
      "max_color_temp_kelvin": 6500,
      "rgb_color": [255, 128, 64],
      "unit_of_measurement": "°C"
    }
  }
]
```

Required handling:

- `attributes` may be missing entirely
- `brightness` may be absent
- `color_temp_*` values may be missing
- `rgb_color` may be absent
- `unit_of_measurement` may be absent
- values should be defaulted safely

### JSON payloads for service calls

#### Toggle or script

```json
{ "entity_id": "switch.foo" }
```

#### Light turn_on with brightness

```json
{ "entity_id": "light.foo", "brightness_pct": 55 }
```

#### Light turn_on with color temp

```json
{ "entity_id": "light.foo", "color_temp_kelvin": 3500 }
```

#### Light turn_on with rgb

```json
{ "entity_id": "light.foo", "rgb_color": [255, 128, 64] }
```

---

## Input Validation Rules

### Form validation

- `entity_id` must be non-empty
- `brightness_pct` must be in range 0..100
- `color_temp_kelvin` must be a positive integer
- `rgb` value must be a 6-digit hex string like `#ff8040`
- `page` and page assignments must be >= 1

### Error handling

- Invalid config should fail fast at startup
- Invalid color strings should return a clear error response
- HA service failures should surface as a server error response with sensible message
- Missing token should be treated as a startup configuration error

---

## Configuration

The Python app must support env-based configuration mirroring the Rust app.

### Required env vars

- `HARETROPANEL_PORT` (default `8080`)
- `HA_BASE_URL` (default `http://localhost:8123`)
- `HA_TOKEN` (required for runtime)
- `HARETROPANEL_TITLE` (default `HARetroPanel - Home Assistant panel`)
- `HARETROPANEL_LOG_DIR` (default `./logs`)
- `HARETROPANEL_LOG_ROTATION` (default `daily`)
- `HARETROPANEL_LOG_LEVEL` (default `haretropanel=info,tower_http=info`)
- `HARETROPANEL_CACHE_TTL_DEFAULT_SECS` (default `5`)
- optional per-kind TTL overrides:
  - `HARETROPANEL_CACHE_TTL_LIGHT_SECS`
  - `HARETROPANEL_CACHE_TTL_SWITCH_SECS`
  - `HARETROPANEL_CACHE_TTL_SENSOR_SECS`
  - `HARETROPANEL_CACHE_TTL_CLIMATE_SECS`

### .env support

- Load `.env` if present
- Allow env values to override file values

---

## Caching Behavior

The Rust app maintains an in-memory dashboard cache with TTLs by entity kind.

The Python version must implement equivalent cache behavior:

- cache the complete dashboard state
- refresh when expired
- invalidate cache after any state-changing action
- use the minimum TTL among entity kinds in the current dashboard state for effective cache duration
- default TTL applies when no per-kind override is configured

Important rule: after `toggle`, `set_brightness`, `set_color_temp`, `set_color`, or `run_script`, the cache must be invalidated before the next read.

---

## Persistence and Layout Repository

The app should persist user configuration to a local JSON file without external services.

### File

`./data/dashboard_layout.json`

### JSON schema

```json
{
  "visible_entities": ["light.kitchen", "switch.fan"],
  "page_assignments": {
    "light.kitchen": 1,
    "switch.fan": 2
  }
}
```

### Repository behavior

- If the file does not exist, create it with empty defaults
- Load visible IDs from `visible_entities`
- Load page assignments from `page_assignments`
- Save updated values atomically when settings are saved

---

## UI and Template Requirements

The app uses a custom dark theme with cards, toolbar, pagination, and settings table.

### Required visual style

- dark background
- white or near-white text
- green action accents for buttons/links
- rounded cards with borders
- flexible cards grid
- mobile-friendly sizing
- must be legible on older tablets

### Template files to match

- `templates/base.html`
- `templates/dashboard.html`
- `templates/entities_settings.html`

The Python version should preserve the existing layout structure and CSS patterns closely, with only necessary updates for Python templating syntax.

---

## JavaScript and Browser Strategy

The current Rust UI includes some client-side JS for:

- WebSocket refresh
- live brightness label updates
- RGB color wheel interaction
- color picker toggle behavior

The Python port should maintain the same features only if they remain compatible with old devices.

### Compatibility rule

Keep JavaScript minimal and safe for older browsers. If a feature relies on modern browser APIs, either:

- rewrite it in older-compatible syntax, or
- simplify it to avoid the dependency

The safest approach is:

- keep WebSocket for live refresh
- do not rely on `requestSubmit()`
- keep event handlers simple and inline or add listeners with older-compatible patterns
- avoid pointer events unless absolutely necessary
- prefer simple DOM updates and native form submission

The app must still be functional in old browsers even if the color wheel is simplified.

---

## Recommended Python Stack

This is the recommended implementation stack for the rebuild.

### Preferred technologies

- Python 3.11+
- FastAPI
- Uvicorn
- Jinja2
- httpx
- pydantic
- python-dotenv

### Why this stack

- Fast and modern enough for a small app
- easy route/controller structure similar to Axum
- Jinja2 matches the Rust template usage well
- httpx is good for HA API requests
- fits the simple server-rendered pattern of the current app

### Alternative acceptable stacks

- Flask + Jinja2 + requests
- Starlette + Jinja2 + httpx

However, the implementation should stay simple, explicit, and maintainable. FastAPI is the preferred default.

---

## Proposed Project Layout

```text
app/
  __init__.py
  config.py
  models.py
  state.py
  cache.py
  repo.py
  ha_client.py
  services.py
  dashboard.py
  routes/
    __init__.py
    dashboard.py
    settings.py
  templates/
    base.html
    dashboard.html
    entities_settings.html
  static/
    .keep
main.py
requirements.txt
.env.example
README.md
```

This layout is only a suggestion; the main requirement is that responsibilities stay cleanly separated and match the Rust app’s architecture.

---

## Recommended Module Responsibilities

### config.py

- load env vars
- validate token and server settings
- provide config object for app runtime

### ha_client.py

- build authenticated HA client
- fetch entity states from `/api/states`
- send service calls to HA
- map payloads to internal entity model

### models.py

- data classes for entity kinds, entity state, page assignments, layout state
- view models used by templates

### repo.py

- reads/writes `./data/dashboard_layout.json`
- manages visible entity selection and page map

### cache.py

- in-memory dashboard cache with TTL logic
- invalidation and cache refresh logic

### services.py

- orchestrates dashboard retrieval, toggle actions, brightness changes, color changes, and script execution
- integrates repo + HA client + cache

### routes/dashboard.py

- `GET /`
- `GET /ws`
- `POST /toggle`
- `POST /brightness`
- `POST /light/color_temp`
- `POST /light/color`
- `POST /run_script`

### routes/settings.py

- `GET /settings/entities`
- `POST /settings/entities`

### main.py

- app startup
- config load
- create app and start uvicorn server

---

## Acceptance Criteria

The Python rebuild is considered complete when all of the following are true:

1. The app starts from a simple Python process and serves the dashboard on the configured port.
2. It authenticates against Home Assistant using a bearer token from environment config.
3. It fetches and displays Home Assistant entity data in a server-rendered dashboard.
4. It shows visible entities only, grouped by configured page assignment.
5. Toggle actions work for lights, switches, and scripts.
6. Brightness actions work for light entities.
7. Color temperature actions work for light entities.
8. RGB color actions work for light entities.
9. The settings page persists visible entities and page assignments to the local JSON layout file.
10. The dashboard uses simple, old-browser-safe HTML/CSS/JS patterns.
11. The app remains lightweight and does not require a database.
12. The project passes a basic test suite covering the main logic: config parsing, view model conversion, and entity filtering/page assignment.

---

## Implementation Notes for a Dev AI

When implementing the rebuild:

- keep the feature set aligned with the Rust app, not a more elaborate dashboard
- preserve old-device compatibility over modern polish
- reuse the existing Home Assistant semantics whenever possible
- do not assume a frontend framework
- treat HTML templates as the primary UI layer
- minimize JavaScript and keep it non-modern
- preserve the dark card layout and general dashboard structure

---

## Notes on Differences from a Typical Modern App

This is intentionally not a modern React or Vue dashboard. It is a static, server-rendered dashboard inspired by legacy enterprise or kiosk systems. The runtime should stay simple and robust.

That is a core product requirement and should not be changed unless the business requirement explicitly changes.

---

## Final Implementation Rule

The Python implementation must be a faithful functional equivalent of the Rust app. A developer AI should not add unrelated features such as user auth, a database, advanced UX orchestration, or a modern SPA unless explicitly requested.

The goal is reliability, compatibility, and fidelity to the current working app behavior.
