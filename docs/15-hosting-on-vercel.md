# Hosting on Vercel

**Recommendation:** Vercel hosts the web app and the static data dashboard. The Rust API, the daily collectors, the database and Obscura run on a container host. Vercel is excellent at the first two and a poor fit for the rest.

This is not a compromise. It is the split that keeps the free tier useful and the data plane honest.

---

## 1. Why not everything on Vercel

Vercel Functions are short-lived, stateless invocations. That is a good fit for rendering and cached reads. It is a bad fit for:

- **Long searches.** A multi-market, multi-date fare search can exceed a function's execution window.
- **Obscura.** A headless browser needs memory, a writable temp directory and time. It cannot run inside a standard Vercel Function.
- **Daily collectors.** They need a scheduler, retries and a place to write.
- **SQLite.** There is no persistent local disk between invocations.
- **Rust.** Rust is not a first-class Vercel runtime. A community runtime exists, but a container is simpler and better supported.

Vercel can still front the API with a rewrite, so the browser sees one origin.

---

## 2. The split

```
┌──────────────────────────────────────────────┐
│ Vercel                                       │
│  · React + Vite web app                      │
│  · static "State of Airfare" dashboard       │
│  · /api/* rewrite to the data plane          │
└──────────────────────┬───────────────────────┘
                       │ HTTPS
┌──────────────────────▼───────────────────────┐
│ Container host — Fly.io / Render / Railway   │
│  · Rust Axum API                             │
│  · scheduler (daily history, Explore)        │
│  · Obscura (optional, sandboxed)             │
│  · SQLite on a volume, or Postgres           │
└──────────────────────┬───────────────────────┘
                       │
          ┌────────────▼────────────┐
          │ Neon / Supabase Postgres │
          │  (if not using SQLite)   │
          └──────────────────────────┘
```

### What runs where

| Component | Host | Why |
|---|---|---|
| Web app | **Vercel** | Static build, CDN, preview deployments |
| State of Airfare dashboard | **Vercel** | Generated at build time from exported data |
| API (`/places`, `/search`, `/route-intel`) | **Container** | Long searches, secrets, database |
| Daily collector | **Container or GitHub Actions** | Cron, retries, writes to the database |
| Obscura | **Container** | Memory, browser lifetime, untrusted JS sandbox |
| Database | **Neon/Supabase or a volume** | Persistence |
| Cache | **Postgres/Redis on the data plane** | Shared across invocations |

---

## 3. Vercel configuration

### Web build

- Framework preset: Vite.
- Build command: `npm run build`.
- Output directory: `dist`.
- Environment: only public configuration, such as `VITE_API_BASE_URL`. Never put a SerpAPI key in a `VITE_` variable; Vite inlines those into the client bundle.

### Proxy the API

Expose the API on the same origin with a rewrite:

```json
{
  "rewrites": [
    {
      "source": "/api/(.*)",
      "destination": "https://api.your-domain.com/api/$1"
    }
  ]
}
```

This avoids CORS for the common path and keeps the frontend simple. If you prefer direct calls, set `VITE_API_BASE_URL` and configure CORS on the API to allow the Vercel preview and production domains.

### Cron

Vercel Cron is convenient, but the collector belongs with the data plane. Two workable patterns:

1. **Recommended:** GitHub Actions runs the Rust collector on a schedule and writes to the database. Free, visible logs, easy retries, no function time limit.
2. **Vercel Cron calls a secured endpoint** on the API host. The endpoint checks a shared secret and returns immediately; the actual work runs in the container.

Do not run a long collection inside a Vercel Function.

### Caching

- Static assets: immutable, long `Cache-Control`.
- The dashboard: rebuild on data export, not on every request.
- API reads: set `Cache-Control` on `/explore` and `/route-intel`; a 5–15 minute CDN cache is appropriate for prices, and a data-freshness header tells the UI how old they are.

### Preview deployments

Preview URLs are useful for design review. Add them to the API's CORS allowlist, or keep the rewrite so previews hit the same API origin. Do not give previews production write access.

---

## 4. Data plane

### Container host

Fly.io, Render or Railway all work. Requirements:

- a persistent volume for SQLite, or a managed Postgres
- a scheduled job or an in-process scheduler
- enough memory for Obscura if it is enabled (start at 1 GB; measure)
- a health endpoint and a readiness check that includes data freshness
- a non-root user and a read-only root filesystem

### Database

| Choice | Use when |
|---|---|
| **SQLite on a volume** | One instance, one writer, simple and cheap. Best for the first launch. |
| **Neon / Supabase Postgres** | Multiple instances, serverless reads, or Vercel Functions need direct read access. |

Use SQLx migrations and run them as a release step, not on every request. Back up the fare history daily; it is the asset.

### Scheduler

- **GitHub Actions:** cron, manual dispatch, logs, artifacts. Best for the collector.
- **Container cron:** simpler if the collector and API share code and volume.
- **Vercel Cron:** use only as a trigger, not as the worker.

The collector should be idempotent: one row per route, travel date, observation day and source. Re-running it must not duplicate history.

---

## 5. Obscura on the data plane

- Run it in the same container as the API only if memory allows; otherwise a separate worker container.
- Do not run it in Vercel Functions.
- Non-root, read-only root, writable storage only for cookies.
- Egress allowlist; one proxy per source where needed.
- No airline credentials; public pages only.
- A kill switch: if the browser pool fails, the API still serves cached data and the UI says “live search unavailable.”

The browser is optional in v1. The licensed API path must work without it.

---

## 6. The fully-Vercel alternative

It is possible if the product is **read-mostly and precomputed**:

```
Vercel static web
  + Vercel Functions (TypeScript) for cached reads
  + Neon Postgres
  + GitHub Actions (Rust collector) writes to Neon
```

What you give up:

- live multi-market searches inside the request path
- Obscura and any browser-based source
- long-running jobs in functions
- a single Rust API surface

If the app becomes “cached views over a precomputed dataset,” this is a reasonable cost-saving architecture. If it searches live, keep the container data plane.

---

## 7. Costs

| Item | Monthly |
|---|---:|
| Vercel Hobby / Pro | $0–20 |
| Fly.io / Render / Railway | $5–25 |
| Neon / Supabase | $0–25 |
| GitHub Actions | $0 |
| SerpAPI | $25–75 |
| Domain | ~$1–2 |

A realistic launch is **$30–100/month** before any sponsorship, depending on search volume and whether Obscura runs continuously.

---

## 8. Checklist

- [ ] Vite app builds to `dist`; no secrets in `VITE_*`
- [ ] `/api/*` rewrite or CORS allowlist for production and previews
- [ ] API health and readiness endpoints, including data freshness
- [ ] SQLite volume or Postgres; migrations run as a release step
- [ ] Collector in GitHub Actions or container cron; idempotent writes
- [ ] Daily backup of fare history
- [ ] Obscura in a sandboxed container, or disabled
- [ ] Rate limits and a shared secret on write endpoints
- [ ] Data-freshness badge in the UI
- [ ] Rollback plan for the web and the data plane

---

## 9. Sources

- [Vercel Functions](https://vercel.com/docs/functions)
- [Vercel Cron](https://vercel.com/docs/cron-jobs)
- [Vercel rewrites](https://vercel.com/docs/rewrites)
- [Fly.io](https://fly.io/docs/)
- [Render](https://render.com/docs)
- [Neon](https://neon.tech/docs)
- [GitHub Actions](https://docs.github.com/en/actions)
