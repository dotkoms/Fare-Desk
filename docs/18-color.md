# Color

> **Light only (23 October 2026).** The dark theme is retired; the app always renders the paper palette. Dark columns below are kept for history.

**Idea: three channels, never mixed.** Price, certainty and status are different questions. If green means “cheap” and green also means “verified”, the user cannot tell whether a card is a good fare or a trustworthy fare. The palette keeps them separate.

The stage follows the theme: a paper globe on a paper stage in light mode, a night globe on a night stage in dark mode. Both are low-chroma and untextured; the data is the only saturated thing on screen.

---

## 1. The three channels

| Channel | Question | Encoding |
|---|---|---|
| **Opportunity** | Is this fare cheap for this route? | Blue / neutral / amber, plus the printed price and percentile |
| **Certainty** | How much history and volatility are behind this? | Pattern, band width, opacity and a segmented meter. No hue. |
| **Status** | Is it verified, changed, sold out, risky? | Green / amber / red, always with an icon and text |

Selection is a fourth state, but it is **not a hue**: stroke weight, casing, endpoint rings and a left border.

Why separate them: Google Flights uses green for emissions reductions, not low prices. Kayak uses a green-to-red price grid but prints every price. About 1 in 12 men has a color-vision deficiency. Hue alone is not a channel.

---

## 2. Tokens

Contrast is against `--surface` unless noted. WCAG requires 4.5:1 for normal text and 3:1 for graphical objects.

| Token | Light | Dark | Light / dark contrast | Use |
|---|---|---|---|---|
| `--page` | `#F6F8FB` | `#071019` | — | Page background |
| `--surface` | `#FFFFFF` | `#0E1A26` | — | Panel, card |
| `--surface-2` | `#EDF2F7` | `#142331` | — | Secondary fill |
| `--ink` | `#0B1B2B` | `#E8EEF5` | 17.4 / 15.1 | Primary text |
| `--ink-2` | `#4A5A6A` | `#A7B5C4` | 7.1 / 8.4 | Body text |
| `--ink-3` | `#637383` | `#8294A6` | 4.9 / 5.6 | Captions, axis labels |
| `--hairline` | `#D6DEE7` | `#223547` | — | Borders, grid |
| `--action` | `#0B1B2B` | `#E8EEF5` | — | Primary button fill |
| `--action-ink` | `#FFFFFF` | `#0B1B2B` | — | Primary button text |
| `--focus` | `#0B1B2B` | `#F8FAFC` | — | Focus ring |
| `--price-good` | `#0B6E99` | `#67C7F0` | 5.7 / 9.2 | ≤ p25 |
| `--price-typical` | `#7C828B` | `#A3A9B2` | 3.9 / 6.9 | p25–p75 |
| `--price-high` | `#8A5A2B` | `#E0A458` | 5.9 / 8.1 | > p75 |
| `--status-ok` | `#067647` | `#6EE7B7` | 5.7 / 11.5 | Verified, on time |
| `--status-warn` | `#B54708` | `#FDB022` | 5.4 / 9.6 | Price changed, risk |
| `--status-danger` | `#B42318` | `#FDA29B` | 6.6 / 9.1 | Sold out, error |

`--price-typical` passes as a graphical fill but not as normal text. Use it for dots and arcs; use `--ink-2` for small text labels.

---

## 3. Price classes

| Percentile | Label | Globe treatment | List treatment |
|---|---|---|---|
| ≤ p10 | **Exceptional** | solid `--price-good`, 100% | filled blue pill: `€89 · 8th pct` |
| p10–p25 | **Good** | solid `--price-good`, 90% | filled blue pill: `€104 · 22nd pct` |
| p25–p75 | **Typical** | `--price-typical`, 60% | outline pill: `€151 · typical` |
| p75–p90 | **High** | `--price-high`, 55% | outline amber pill: `€190 · 78th pct` |
| > p90 | **Very high** | `--price-high`, 40% | amber text + up arrow: `€240 · 94th pct` |

On the globe, collapse to three classes: Good (≤p25), Typical (p25–p75), High (>p75). The five-step scale lives in the chart and card, where there is room for numbers.

Every price pill includes the price and the percentile or the word “typical”. Color is never the only signal. The numbers carry the meaning; color speeds the scan.

---

## 4. Certainty

- **Volatility** is a glyph, not a color: one bar = stable, two bars = variable, three wavy bars = erratic. Text follows: “Fare weather: variable (CV 0.18; 7-day range €82–€128).” Do not encode volatility on the globe; it turns the map into a Christmas tree.
- **Track record** is text plus a sample: “Right 7 of 10 similar calls (n=214)”. Never show a raw confidence score. If there is no track record, say so; fewer than 14 observations gets a dotted 1px border and no advice.
- **Expected range** is a horizontal bar from p10 to p90 with a median tick, not a single predicted price. Label both ends and the median.
- **Low sample** is a progress state: “Collecting baseline — 12 of 14 days.” No advice.
- Never use a traffic light for certainty. Green/amber/red already mean status; certainty is not a warning.

---

## 5. Selection, hover and dimming

| State | Arc | List card | Other arcs |
|---|---|---|---|
| Default | 1 px, `--arc-base`, 60% | surface, hairline | 60% |
| Hover / focus preview | 1.5 px, 100%, tooltip | hover border + 1 px lift | 35% |
| Selected | 2.5 px + 2 px casing, endpoint rings | 3 px left border, tinted surface, “Selected” | 15% |
| Stale (query changed) | 1 px dashed, 30% | muted | 30% |
| Dimmed by intelligence | selected 100%, others 10% | n/a | 10% |

The selected stroke is theme-aware: `--ink` on light, `#F8FAFC` on dark. A fixed white arc is invisible on the paper globe.

Casing is `--surface` at 2 px, so the arc reads against both land and ocean.

---

## 6. Status

| Status | Color | Icon | Text |
|---|---|---|---|
| Verified | `--status-ok` | check | “Verified 4 min ago” |
| Price changed | `--status-warn` | triangle | “Price changed from €84 to €89” |
| Sold out | `--status-danger` | octagon | “Sold out at this fare” |
| Collecting baseline | `--price-good` (info) | info | “Collecting baseline — 12 of 14 days” |
| Self-transfer risk | `--status-warn` | shield | “Self-transfer; re-check bags” |

Status badges are small and always text-bearing. They never become the dominant fill of a card. Do not use the status palette for price, or the price palette for status.

---

## 7. The globe

Low-chroma, no textures, no stars, no city lights.

| Element | Light — paper globe | Dark — night globe |
|---|---|---|
| Ocean | `#C2DBEC` | `#050D14` |
| Land | `#EFF3F7` | `#101F2E` |
| Coastline | `#9FB6C8` | `#223547` |
| Graticule | `#C9D4DF` at 6% | `#2A4054` at 6% |
| Arc base | `#5F7488` at 60% | `#6F879D` at 60% |
| Selected arc | `--ink`, 2.5 px + casing | `#F8FAFC`, 2.5 px + casing |
| Airport point | `--ink` endpoints, `#5F7488` others | `#F8FAFC` endpoints, `#7F93A6` others |
| Atmosphere | 2 px rim at 12% | 2 px rim at 12% |

- No NASA Blue Marble, night lights, clouds or starfield. They cost 1–4 MB, fight label contrast, imply a time of day the fare search does not have, and make the globe feel like a screensaver.
- No glow or bloom. A thin atmosphere rim is enough.
- Graticule at world view only; hide at route scale.
- Same semantics in both themes. Do not swap blue and amber.

---

## 8. Color-vision check

The three price colors were tested with Machado (2009) protanopia, deuteranopia and tritanopia matrices. CIE ΔE76 separation:

| Pair | Protanopia (light/dark) | Deuteranopia (light/dark) | Tritanopia (light/dark) |
|---|---|---|---|
| Good vs High | 63 / 72 | 76 / 83 | 66 / 82 |
| Good vs Typical | 28 / 24 | 40 / 31 | 26 / 36 |
| Typical vs High | 40 / 49 | 41 / 53 | 43 / 48 |

Good-vs-High is strong. Good-vs-Typical is the weakest pair, which is why the pill always prints the price and the percentile. Do not rely on the color difference alone.

---

## 9. CSS tokens

```css
:root {
  --page: #F6F8FB;
  --surface: #FFFFFF;
  --surface-2: #EDF2F7;
  --ink: #0B1B2B;
  --ink-2: #4A5A6A;
  --ink-3: #637383;
  --hairline: #D6DEE7;
  --action: #0B1B2B;
  --action-ink: #FFFFFF;
  --focus: #0B1B2B;
  --price-good: #0B6E99;
  --price-typical: #7C828B;
  --price-high: #8A5A2B;
  --status-ok: #067647;
  --status-warn: #B54708;
  --status-danger: #B42318;
  --globe-ocean: #DCE9F5;
  --globe-land: #E9EEF4;
  --globe-coast: #C9D4DF;
  --arc-base: #5F7488;
  color-scheme: light;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --page: #071019;
    --surface: #0E1A26;
    --surface-2: #142331;
    --ink: #E8EEF5;
    --ink-2: #A7B5C4;
    --ink-3: #8294A6;
    --hairline: #223547;
    --action: #E8EEF5;
    --action-ink: #0B1B2B;
    --focus: #F8FAFC;
    --price-good: #67C7F0;
    --price-typical: #A3A9B2;
    --price-high: #E0A458;
    --status-ok: #6EE7B7;
    --status-warn: #FDB022;
    --status-danger: #FDA29B;
    --globe-ocean: #0A1724;
    --globe-land: #152536;
    --globe-coast: #2A4054;
    --arc-base: #6F879D;
    color-scheme: dark;
  }
}

:root[data-theme="dark"] {
  --page: #071019;
  --surface: #0E1A26;
  --surface-2: #142331;
  --ink: #E8EEF5;
  --ink-2: #A7B5C4;
  --ink-3: #8294A6;
  --hairline: #223547;
  --action: #E8EEF5;
  --action-ink: #0B1B2B;
  --focus: #F8FAFC;
  --price-good: #67C7F0;
  --price-typical: #A3A9B2;
  --price-high: #E0A458;
  --status-ok: #6EE7B7;
  --status-warn: #FDB022;
  --status-danger: #FDA29B;
  --globe-ocean: #0A1724;
  --globe-land: #152536;
  --globe-coast: #2A4054;
  --arc-base: #6F879D;
  color-scheme: dark;
}
```

---

## 10. What not to do

- No green for “cheap”. Green means verified or on time.
- No red for “expensive”. Red means sold out or error.
- No rainbow price scale. It is not perceptually ordered.
- No color-only price, status or selection.
- No NASA texture, night lights, clouds or starfield on the globe.
- No pure black and pure white for large surfaces; use the tokens.
- No more than one action color per screen.
- No amber primary button; amber belongs to price-high and status-warn.
- No colored airline logos; they fight the data and require licensing.
