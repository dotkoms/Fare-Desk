# 2D poster map — replacing the globe

**Status:** proposal. No code written. The current surface is the WebGL globe in [21-globe-implementation.md](21-globe-implementation.md); this document is the plan to replace it with a flat, poster-styled map inspired by Hiroshi Nagai.

**Decision:** move from a 3D sphere to a 2D SVG map drawn like a 1980s travel poster — flat colour, hard edges, hard shadows, an airbrushed sky band, and a lot of empty space. The product stays list-first; the map is still the lobby and Explore surface. The route board stays exactly as it is.

---

## 1. What "inspired by Hiroshi Nagai" means here

Nagai's work is the visual language of Japanese city-pop sleeves and American West Coast leisure: pools, palms, highways, wide horizons, saturated flat colour, crisp geometry, hard-edged shadows, airbrushed skies, almost no people, and a deliberate stillness.

The rules we take:

| Rule | Translation |
|---|---|
| **Flat colour, no texture** | Ocean, land, sky and route strokes are solid fills. No relief, no grain, no paper texture, no gradients on data. |
| **A horizon and a sky** | The map is framed as a poster: a sky band with a sun disc above the map, the map below it. |
| **Hard shadows, never blur** | Label cards and airport dots use an offset duplicate at 1–2 px, not `box-shadow` blur. |
| **Crisp geometry** | Thin coast line, round-capped routes, circular dots, no glow. |
| **Empty space** | The plot area carries only data. Decoration lives in the margins and the sky band. |
| **Restraint in motion** | Nothing floats, pulses or rotates. Camera glides only when a route needs it. |

What we do **not** do: reproduce or trace specific paintings, use album covers as assets, or ship AI imagery prompted with the artist's name. The inspiration is a set of compositional and colour rules, not a copy of a body of work.

The palette is a derived direction, not sampled from a particular piece:

| Token | Day | Dusk | Use |
|---|---|---|---|
| `--map-sky-top` | `#6FC9E4` | `#2B3A67` | Sky band, top |
| `--map-sky-low` | `#CFE9E3` | `#E8846B` | Sky band, horizon |
| `--map-sea` | `#1E86A8` | `#1B2A4A` | Ocean fill |
| `--map-sea-deep` | `#14627F` | `#141F38` | Ocean edge, dim routes |
| `--map-sand` | `#F0E2C4` | `#3E4A63` | Land fill |
| `--map-sand-hi` | `#F7EEDB` | `#4A5773` | Land at the coast |
| `--map-ink` | `#12324B` | `#F5E6CC` | Text, coast line, selected route |
| `--map-coral` | `#F2795C` | `#F2A65A` | Sun, hover, accent |
| `--map-white` | `#FDF8EE` | `#FDF8EE` | Label cards, route casing |
| `--map-shadow` | `rgba(18,50,75,0.18)` | `rgba(5,10,20,0.35)` | Hard offset shadows |

Day is the default. Dusk exists so the existing light / dark / system toggle keeps a purpose; it is a time of day, not a generic dark mode.

---

## 2. What changes, what stays

| Stays | Changes |
|---|---|
| The list-first shell, search console, results, route board, tabs | `GlobeStage` becomes `MapStage` |
| The itinerary contract and `Itinerary[]` input | Projection, drawing, camera and labels |
| Selection semantics: selected / base / dimmed, 2.5 px selected | Colour from `--map-*` instead of the slate palette |
| Label budgets (12 desktop / 6 phone) and pinned-first collision | Label cards get hard shadows on poster backgrounds |
| No auto-rotation, no draw-on, no pulse | Zoom/pan replaces orbit; a route-fit glide replaces the camera flight |
| The API, fixtures, ranking, history | No backend change at all |
| `world-atlas` land-110m + `topojson-client` | `react-globe.gl`, `three`, `three-globe`, `@types/three` removed |

Removed with the globe: `GlobeFallback.tsx`, `GlobeBoundary.tsx`, `lib/webgl.ts`, and the lazy Three chunk. The web bundle loses roughly 570 kB gzipped and gains a small projection library.

---

## 3. Rendering approach

**Use SVG + `d3-geo`.** Not MapLibre, not Canvas.

- `d3-geo` gives projections, `geoPath` (geodesic resampling and antimeridian clipping) and `geoInterpolate`. It is small and has no WebGL requirement.
- SVG gives crisp poster edges at any zoom, CSS-driven theming, real DOM hit targets for airports and routes, and inspectable output.
- Canvas would need manual hit-testing and offers nothing at this data volume.
- MapLibre brings tiles, styles, WebGL and ~800 kB for a look we do not want: the beauty here is flat vector, not raster cartography.

### Projection

One projection across all zoom levels, `geoNaturalEarth1`: flat, gently curved, poster-friendly, and it keeps continuity when the camera fits a route. Equirectangular is the runner-up if the style lab wants a harder horizon line; Mercator is rejected because it drags the eye toward the poles and reads as a web map.

Route-fit: compute the projected bounding box of origin, connections and destination, add 64 px padding (96 px when the mobile sheet is open), clamp scale to 1–6, and animate the transform over 500 ms (instant under reduced motion).

### Layers

```
<svg>
  poster frame      sky band, sun disc, ocean wash, thin frame rule
  land              sand fill + 1 px ink coast at 35%
  routes (idle)     ink at 25%, 1 px
  route (selected)  white casing 4.5 px + ink 2.5 px, round caps
  connections       geodesic polyline through the connection airports, dot at each stop
  airports          ink dots, radius by log traffic; origin/destination as 2.2 px ink disc with white core
  hit targets       transparent 12 px stroke over each route, 44 px circle per airport
</svg>
<labels>            HTML cards positioned from the same transform
<controls>          zoom, reset, list-only, legend
```

- **Routes are geodesics.** Build a GeoJSON `LineString`, let `geoPath` resample it, and the antimeridian is handled for us. No hand-rolled bezier, no bulge: a great-circle line on Natural Earth is already the gentle curve a poster wants.
- **One-stop itineraries are polylines**, never a direct line. A dot marks the connection.
- **Selected route** is drawn under everything else so its casing reads cleanly; hovered routes get ink at 80% and 1.5 px.
- **Self-transfer** is dashed (6 4), hidden-city dotted, OTA-only carried by the list tag. Dash pattern plus legend, never colour alone.
- **Land** is one `<path>` from the existing 110m TopoJSON. The coast line is a second pass with `fill="none"`.

### Labels and interaction

- Labels are HTML (`position: absolute`) positioned from `projection([lng, lat])` plus the current transform. Recompute on pan/zoom via `requestAnimationFrame`, throttled; at most 12 labels the cost is trivial.
- Label card: white fill, 1 px ink border, 2 px offset hard shadow, ink type. No blur.
- Pinned labels (origin, destination, selection, hover) never hide; others are collision-tested and dropped.
- Airport hover raises its label; route hover raises a small route chip (`TAP · €119 · 3h20`), no camera movement.
- Clicking a route selects its itinerary in the list; clicking empty map clears selection; clicking an airport fills origin or destination depending on what is empty.
- Pan: pointer drag; zoom: wheel or pinch to cursor; double-click resets; `+ / − / reset` buttons follow [17-design-system.md](17-design-system.md#globe-controls).
- Keyboard: the map is `aria-hidden`; the list is the keyboard path. A visible first-focus target on the map offers “use the list instead”.
- Mobile: full-bleed map, bottom sheet unchanged, pinch enabled, labels at 6.

### Accessibility and failure

The map has no WebGL path, so the WebGL fallback disappears entirely; a map that fails to mount is a map with no land drawn, not a blank stage. Contrast is checked for ink-on-sand, white-on-ink and label cards in both themes. No information is map-only: every airport, route and price is in the list, and “list only” remains a first-class toggle.

---

## 4. Poster composition

The stage is a print, not a dashboard:

- **Frame:** 16 px inner margin of the stage colour, then a 1 px ink rule, then the sky band (14% of height) and the map. On desktop the map keeps a 3:2 field inside a wider stage; on mobile it fills the sheet area.
- **Sky:** a vertical two-stop gradient, one coral sun disc, optionally two flat cloud bars. The sun sits above the map's upper edge and never overlaps land.
- **Sea:** the map's ocean fill continues past the land into the frame's lower margin, so the map reads as a scene rather than a diagram.
- **Decoration budget:** one sun, at most two clouds, no palms, no cars, no people. Palms would compete with the airport dots and read as clip art.
- **Type:** the only poster type is the route line already in the stage head (`BER → LIS`, dates, total). Labels stay small and utilitarian; the poster look must not turn IATA codes into display type.

If a treatment needs more than that to feel like Nagai, the palette or the composition is wrong, not the decoration count.

---

## 5. Interaction parity checklist

Every behaviour in [16-ui-ux.md](16-ui-ux.md) and [21-globe-implementation.md](21-globe-implementation.md) that must survive:

- [ ] Result card selects its route; route highlight and list stay in sync both ways.
- [ ] Route click selects the matching itinerary and scrolls the card into view.
- [ ] One-stop itinerary renders as a polyline with a connection dot.
- [ ] Duplicate services merge to one line; the selected one is visually distinct.
- [ ] Airport hover labels; origin/destination labels are always pinned.
- [ ] No camera movement on hover.
- [ ] Fit-to-route on first load and on selection change; no jump when the route is already in view.
- [ ] Empty-map click clears selection.
- [ ] `+ / − / reset` and “list only” controls.
- [ ] Reduced motion: instant fit, no transitions.
- [ ] Mobile: pinch, six labels, sheet never covers the selected midpoint.
- [ ] No WebGL dependency; keyboard path works without the map.

---

## 6. Files

New:

```
app/web/src/components/MapStage.tsx        the component
app/web/src/map/projection.ts              projection, fit, transform maths
app/web/src/map/land.ts                    TopoJSON → one land path
app/web/src/map/routes.ts                  itineraries → geodesic paths, casing, dash
app/web/src/map/labels.ts                  screen coords, collision, budgets
app/web/src/map/palette.ts                 day/dusk tokens for canvas-driven values
```

Changed: `Stage.tsx` (mount `MapStage`, drop `GlobeBoundary`/`Suspense`), `styles.css` (map tokens and frame; remove `.globe-*`), `App.tsx` (no change expected), `docs/16`, `docs/17`, `docs/18`, `PLAN.md` §4 and §14, `README.md`.

Removed: `GlobeStage.tsx`, `GlobeFallback.tsx`, `GlobeBoundary.tsx`, `lib/webgl.ts`, and the dependencies `react-globe.gl`, `three`, `@types/three`.

---

## 7. Build plan

| Step | Work | Effort | Gate |
|---|---|---:|---|
| **A — style lab** | A private Artifact comparing four treatments at real size: day poster, dusk, data-first flat, full-bleed. Same BER–LIS data in each | 0.5–1 day | One treatment picked; palette, projection and framing fixed |
| **B — skeleton** | SVG, projection, land, pan/zoom, fit-to-route, day/dusk tokens | 2–3 days | 60 fps pan on an M1 Air; renders with JS and no WebGL; correct at 3:2 and phone |
| **C — data** | Airports, routes, casing, connections, selection/dim, labels, collision | 2–3 days | Parity checklist green; AKL–SCL and NRT–HNL render without a seam |
| **D — interaction** | Click both ways, controls, keyboard path, list-only, reduced motion, mobile sheet | 1–2 days | A user can complete a search without touching the map; the map never traps focus |
| **E — cleanup** | Remove globe deps and CSS, update docs and screenshots, bundle check | 1–1.5 days | Bundle −500 kB gzip; axe has no critical issues; docs no longer describe a globe |

Total: **7–10 days**, plus a day of buffer. This replaces Step 2 of the planner; the data plane and Step 4 are unaffected.

**Order:** finish the style lab and Step B before wiring the web app to the API, so the API client is written against the final stage rather than twice.

---

## 8. Risks

| Risk | Mitigation |
|---|---|
| The poster fights the data | Decoration only outside the plot area; hard budget of one sun and two clouds; label cards stay plain |
| Bright map clashes with the paper console | The map is framed inside the stage; the console stays paper in both themes. If it still clashes, the style lab shows a muted-day variant |
| Style pastiche or misattribution | Derive rules, never trace or reuse artwork; no artist-name prompts for generated assets; credit the inspiration in the doc only |
| Projection distortion at high latitude | Natural Earth 1 is mild; test BER–LIS, LIS–GRU, NRT–HNL and AKL–SCL |
| Antimeridian seams | `geoPath` clips them; the parity gate includes an AKL–SCL case |
| Explore scale (300 routes) | One `<path>` per route, no filters, hard shadows as duplicate paths; LOD hides idle routes below zoom 1.5 |
| SVG pan jank on low-end phones | All layers live in one transformed `<g>`; labels throttled to rAF; test at 4× CPU throttle |
| The word “globe” everywhere | Rename the surface to “map” in the docs and UI copy during Step E; the route board name is unchanged |

---

## 9. Decisions before building

1. **Projection:** Natural Earth 1 (recommended) · equirectangular for a harder horizon · Mercator for familiarity.
2. **Framing:** framed poster inside the stage (recommended) · full-bleed edge-to-edge.
3. **Themes:** day + dusk tied to the existing toggle (recommended) · day only.
4. **Decoration:** sky band, one sun, up to two clouds (recommended) · frame only, no motifs · none.
5. **The old globe:** remove it (recommended) · keep it behind a toggle.

---

## 10. Sources

- [21-globe-implementation.md](21-globe-implementation.md) — current implementation, budgets and label rules
- [16-ui-ux.md](16-ui-ux.md) — screens, interaction model, accessibility
- [17-design-system.md](17-design-system.md) — globe vs flat, controls, motion
- [18-color.md](18-color.md) — current palette and semantics
- [PLAN.md](../PLAN.md) §4 — the globe and route board as they stand
