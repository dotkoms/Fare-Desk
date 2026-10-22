# Design system

**Concept: the instrument and the map.** A calm console beside a low-chroma globe: a paper globe in light mode, a night globe in dark mode. No satellite texture, no stars, no city lights. The data is the only saturated thing on screen. The interface should feel precise, quiet and trustworthy — closer to a departure board than a booking site.

**Brand attributes:** precise, candid, calm, exploratory. Never hyped, never cute, never urgent.

---

## 1. Typography

| Role | Face | Weight | Size / line | Use |
|---|---|---|---|---|
| Display | **Archivo** | 700 | 52/0.95, tracking −0.03em | Hero, empty-state headline |
| H1 | Archivo | 650 | 32/1.1, −0.02em | Screen titles |
| H2 | Archivo | 600 | 24/1.2 | Section titles |
| H3 | Archivo | 600 | 18/1.3 | Card titles |
| Body | **IBM Plex Sans** | 400 | 15/1.55 | Paragraphs, descriptions |
| UI label | IBM Plex Sans | 500 | 13/1.3 | Buttons, fields, tabs |
| Caption | IBM Plex Sans | 400 | 12/1.4 | Helper text, timestamps |
| Data | **IBM Plex Mono** | 400/500 | 12–20, tabular | Prices, times, codes, percentiles |

**Rules:**

- Prices and times use `font-variant-numeric: tabular-nums`.
- Headings use `text-wrap: balance`.
- Uppercase labels are 10–11px mono with 0.08em tracking; use sparingly.
- Body text is never below 13px on mobile.
- No more than three weights in a view.
- Fallback stacks: `Archivo, "Helvetica Neue", Arial, sans-serif`; `"IBM Plex Sans", system-ui, sans-serif`; `"IBM Plex Mono", ui-monospace, monospace`.

**Why this pairing:** Archivo gives the wordmark and headings a compact, instrument-like authority. IBM Plex Sans is highly legible at small sizes. IBM Plex Mono makes fares and times align in columns without a second display face.

---

## 2. Layout

### Desktop

```
┌──────────────────────────────────────────────────────────────┐
│ top bar: wordmark · search summary · watchlist · data        │
├───────────────┬──────────────────────────────────────────────┤
│ console 380px │ globe, fluid                                 │
│               │                                              │
│ search /      │                                              │
│ results /     │                                              │
│ route intel   │                                              │
└───────────────┴──────────────────────────────────────────────┘
```

- Console: **380px** in search and results; **460px** in route intelligence.
- Globe: fills the rest; minimum **480px** on desktop.
- Detail drawer: **360px**, overlays the globe from the right; does not resize the console.
- Max content width in the console: **none**; use the full width with 16px padding.
- Top bar: 56px; hairline bottom; no shadow.

### Breakpoints

| Width | Layout |
|---|---|
| ≥1200 | 380/460 console + globe + optional drawer |
| 900–1199 | 340px console + globe; drawer becomes a sheet |
| 640–899 | Globe + 48dvh bottom sheet; console content in the sheet |
| <640 | Full-bleed globe + bottom sheet; snap points 88px / 48dvh / 92dvh |

### Spacing

Base unit **4px**. Use only: `4, 8, 12, 16, 24, 32, 48, 64`.

- Card padding: 16px.
- Stack gap: 12px; section gap: 24px; screen gap: 32px.
- Form field gap: 12px.
- Table row: 40px; header 32px.
- Button height: 40px; small 32px; touch target 48px on mobile, 24px minimum on desktop.

### Radii

- Inputs, buttons, cards: **6px**.
- Small badges/chips: **4px**.
- Bottom sheet: **16px** top corners.
- Pills only for status, not for every control.
- No `rounded-2xl` on everything.

### Borders and elevation

- Hairline: `1px solid var(--hairline)`.
- Selected card: inset `0 0 0 1px var(--ink)` plus a 2px offset ring; do not change padding.
- Popover: `0 8px 24px rgba(16,26,28,0.12), 0 1px 2px rgba(16,26,28,0.06)`.
- No shadow on standard cards; the hairline is enough.
- No glassmorphism.

### Globe vs flat

The globe is the lobby; the route board is the desk. Use camera altitude, not route distance, as the trigger:

| Altitude | Projection | Use |
|---|---|---|
| 2.6–1.6 R | Globe | World and continent overview, Explore, route context |
| 1.6–1.35 R | Crossfade globe → flat | Regional context |
| < 1.35 R | Flat map or route board | Route detail, short routes, precise comparison |

Below 1.35 R, a globe shows one hemisphere and the destination can sit behind the horizon. Switch to a flat route board with the origin, connection and destination on a linear strip. Keep the globe for Explore and for the first orientation.

### Labels and arcs

- Labels are **HTML in screen space**, positioned from `getScreenCoords`, collision-tested and ranked. Do not use 3D text meshes.
- Budgets: 12 labels / 40 arcs at world view; 24 / 120 at continent; 40 / 300 at region; endpoints plus three connections at route view. Halve on mobile.
- Pinned labels (origin, destination, selection, hover) are never hidden. Lower-priority labels hide when they collide.
- Deduplicate arcs by airport pair. Twelve BER–LIS itineraries are one arc with a “12 options” badge.
- A one-stop itinerary is a polyline with a connection dot, not a direct arc.
- Arc width encodes state: 1 px base, 1.5 px hover, 2.5 px selected plus a 2 px casing. Price is colour and opacity, not width.
- Dimming: 60% for context, 35% for non-hovered during a hover, 15% for non-selected. Never 8%; the route structure disappears.
- No auto-rotation, no draw-on, no pulsing rings, no plane animation. A fare search is not live tracking.

---

## 3. Components

### Buttons

| Variant | Fill | Text | Border | Use |
|---|---|---|---|---|
| Primary | `--action` | `--action-ink` | none | Search, “Open on provider” |
| Secondary | transparent | `--ink` | 1px `--hairline` | Watch, compare |
| Ghost | transparent | `--ink-2` | none | Tertiary actions |
| Destructive | transparent | `--status-danger` | 1px `--status-danger` | Remove watch, clear data |

- Height 40px; padding 0 16px; radius 6px; font 13/500.
- Hover: primary darkens 6%; secondary fill `--surface-2`.
- Focus: 2px `--focus` ring, 2px offset.
- Disabled: 40% opacity, no pointer.
- Loading: 16px spinner replaces the label; width locked.

### Inputs and combobox

- 40px height; 1px `--hairline`; radius 6px; background `--surface`.
- Label above, 12px/500; helper below, 12px `--ink-3`.
- Focus: 2px `--focus` border + 2px ring at 20%.
- Error: `--status-danger` border + message with an icon and fix.
- Combobox: 320px max dropdown; 8 results; airport code right-aligned in mono; city and country secondary.
- Date range: two-month calendar; selected range in `--ink` at 8%; today marked with a dot and a label, not color alone.

### Tabs

- Underline tabs, 2px `--ink` for active; no pill tabs.
- 40px height; 13px/500; count badge in mono.

### Result card

```
┌────────────────────────────────────────────┐
│ 06:40 → 09:00            BER → LIS   6h 20 │
│ Air Baltic · 1 stop via MAD · 1h 05        │
│ €89 total · 1 checked bag included         │
│ Verified 4 min ago                 [Watch]  │
└────────────────────────────────────────────┘
```

- 16px padding; 6px radius; hairline.
- Price: 20px mono/500; “total” label 11px.
- Airline and stops: 13px `--ink-2`.
- Selected: 2px inset ink ring; the card does not move.
- Hover: background `--surface-2`, arc lifts.
- Risk badge: 10px mono, 4px radius; “Self-transfer” or “OTA only”.

### Status badges

| Badge | Background | Text |
|---|---|---|
| Good price | `--price-good` at 12% | `--price-good` |
| Typical | `--surface-2` | `--ink-2` |
| Above typical | `--price-high` at 12% | `--price-high` |
| Not enough data | transparent + dashed border | `--ink-3` |
| Verified | `--status-ok` at 12% | `--status-ok` |

- 10px mono, uppercase, 0.06em; 2px 6px padding; 4px radius.
- Include an icon or word; never color alone.

### Route intelligence

- Current total: 32px mono/600.
- Range bar: 8px tall; middle-80% band `--ink-3` at 15%; current marker `--ink`, 12px, with a 2px casing.
- Verdict: 24px Archivo/650; one line of explanation.
- Evidence row: 3 columns (typical range, sample, track record) with 12px labels above 14px values.
- Chart: 160px tall; one line, one band, one marker; no legend; direct labels.
- “What would change our mind”: 13px `--ink-2`, one sentence.

### Provider row

| Provider | Country | Total | Status | Action |
|---|---|---|---:|---|
| Airline DE | DE | €119 | Eligible | Open |
| OTA | UK | €124 | Listed | Open |

- 13px; 40px row; hairline separators.
- Status uses the badge system.
- “Open” is a secondary button, not a full-width primary on every row.
- The primary action is the best eligible row.

### Charts

- **Price history:** 1.5px `--ink` line; 12% `--ink-3` band; dashed median; ink dot with casing for current.
- **Volatility:** range bar with p10–p90; label the band.
- **Seasonality:** 12-column strip, one cell per month; value encoded by fill opacity plus a numeric label on hover/focus.
- **Access rings:** 30/60/90-minute rings on the globe; legend in the panel.
- Axis labels 11px mono `--ink-3`; no gridline unless it carries a value.
- Every chart has a one-sentence summary and a table alternative.

### Globe controls

- Bottom-left: zoom +/−, reset north, “list only” toggle.
- Bottom-right: legend, 240px max, collapsible.
- Top-right: “Explore” / “Search” mode switch.
- Tooltip: 12px, dark surface, max 240px, 8px offset; dismissable with Esc.

### Bottom sheet

- Top radius 16px; drag handle 36×4px `--hairline`.
- Snap points: 88px, 48dvh, 92dvh.
- Sticky search summary in the collapsed state.
- The sheet never covers the selected arc’s midpoint; camera padding accounts for it.

---

## 4. Motion

| Motion | Duration | Easing | Notes |
|---|---:|---|---|
| Hover / focus | 120ms | ease-out | Colour and 1px lift only |
| Panel / drawer | 240ms | cubic-bezier(0.2, 0.8, 0.2, 1) | Transform + opacity |
| Arc fade-in | 200ms | ease-out | After the camera settles |
| Camera flight | 350–800ms, scaled by distance | ease-in-out-cubic | Interruptible; suppressed for 8s after manual movement |
| Bottom sheet | 240ms | same | Follows the finger while dragging |
| Selection ring | 600ms, once | ease-out | Then a static halo, or none |

- Respect `prefers-reduced-motion`: no auto-rotation, no camera flight, no arc animation, no pulse.
- Never animate a price change with a slot-machine roll; cross-fade and label the change.
- No parallax, no floating particles, no constant motion.

---

## 5. Iconography

- 1.5px stroke, 16/20/24px, rounded joins.
- Use a consistent set (Lucide or a custom 20-icon set), not emoji.
- Icons always pair with a label except in the globe controls, which have tooltips and `aria-label`.
- Airline logos are not used; text names avoid licensing and visual noise.
- The wordmark is text. No globe icon; the product already has one.

---

## 6. Content density

- One idea per card.
- Two lines of metadata maximum before a divider.
- Prices always right-aligned in lists; times left-aligned.
- Use sentence case, not Title Case.
- Dates: “12–19 Jun 2027”; times: “06:40”; durations: “6h 20”.
- Currencies: `€412`, `$412`, `£412`; never `EUR 412.00` in the UI.
- Percentiles: “7 in 10 paid more”, with the percentile in the analyst view.

---

## 7. Accessibility

- WCAG 2.2 AA; 4.5:1 body text; 3:1 large text and graphical objects.
- Focus visible on every control; 2px focus ring.
- 48×48px touch targets on mobile; 24×24px minimum on desktop.
- Form labels visible; no placeholder-only labels.
- `aria-live` for result count, selection and price changes.
- Charts: accessible name, summary, data table.
- Globe: list-only mode; keyboard path through results.
- Reduced motion honored.
- No color-only encoding.
- Test at 200% zoom and with a screen reader.

---

## 8. Do and don’t

**Do**

- Use hairlines, whitespace and type hierarchy before boxes and shadows.
- Keep one amber action per screen.
- Put the total price in the largest type in the result.
- Let the globe be dark in both themes.
- Use mono for anything that aligns in a column.
- Make “not enough data” look intentional, not broken.

**Don’t**

- No purple gradients, neon-on-black, or glass panels.
- No cream + serif + terracotta “editorial AI” look.
- No emoji as icons or section markers.
- No `rounded-xl` on every element.
- No stock photography of people on beaches.
- No drop shadows on every card.
- No more than one accent colour.
- No fake urgency in type or motion.

---

## 9. Component inventory for the first build

1. Top bar
2. Search console
3. Place combobox
4. Date range picker
5. Passenger / bag selector
6. Filter chips
7. Result card
8. Status badge
9. Itinerary detail
10. Provider row
11. Route-intelligence card
12. Range bar
13. Price history chart
14. Alert sheet
15. Watchlist row
16. Globe controls
17. Legend
18. Bottom sheet
19. Empty / loading / error states
20. Data and privacy panel

Build these in that order. Each one needs light, dark, hover, focus, disabled, loading and empty states.
