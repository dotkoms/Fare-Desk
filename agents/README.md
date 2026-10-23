# Installing the fleet

These are Claude Code subagent definitions — Markdown files with YAML frontmatter. They are prompts, not daemons: they run when you ask them to, and they share files on disk between runs.

## Install

From the project where you do travel planning:

```bash
mkdir -p .claude/agents
cp /path/to/fleet/agents/desk-ops.md \
   /path/to/fleet/agents/air-scout.md \
   /path/to/fleet/agents/rail-scout.md \
   /path/to/fleet/agents/coach-scout.md \
   /path/to/fleet/agents/arbitrage-analyst.md \
   /path/to/fleet/agents/fare-verifier.md \
   /path/to/fleet/agents/watchkeeper.md \
   /path/to/fleet/agents/points-strategist.md \
   .claude/agents/
```

Do **not** copy this README into `.claude/agents/`; only the eight role files.

Create the shared data directory once:

```bash
mkdir -p data/briefs data/candidates
```

## Order of operations

```
desk-ops
  ├─ air-scout      ─┐
  ├─ rail-scout      ├─→ candidates/*.jsonl
  └─ coach-scout    ─┘
            ↓
     arbitrage-analyst
            ↓
       fare-verifier
            ↓
          desk-ops  → ranked shortlist
            ↓
       watchkeeper  → ledger + alerts
```

`points-strategist` runs on demand, after a shortlist exists.

## Example invocation

> Use desk-ops to plan Berlin → Lisbon, 1–8 March 2027, one checked bag, flexible ±2 days, value my time at €15/hour. Give me the three best itineraries on total cost, and verify the top one.

The orchestrator will produce a trip brief, dispatch the mode scouts, combine the candidates, and return a shortlist. Nothing is booked.

## Shared files

| Path | Written by | Read by |
|---|---|---|
| `data/briefs/<trip>.md` | desk-ops | all |
| `data/candidates/<trip>-air.jsonl` | air-scout | arbitrage-analyst |
| `data/candidates/<trip>-rail.jsonl` | rail-scout | arbitrage-analyst |
| `data/candidates/<trip>-coach.jsonl` | coach-scout | arbitrage-analyst |
| `data/verified/<trip>.jsonl` | fare-verifier | desk-ops, watchkeeper |
| `data/ledger.jsonl` | watchkeeper | all |
| `data/watchlist.yaml` | you, desk-ops | watchkeeper, air-scout |

## Interface rules

- Cut taglines, helper text and secondary labels. If a screen needs them to make sense, fix the layout.
- Every change should lift the user experience (UX), the developer experience (DX) and the agent experience (AX) together. If a change trades one away, find the version that does not — and never break a working flow to get there.

## Ground rules

- No agent books, changes or cancels anything.
- No price leaves the fleet without a verifier stamp.
- One quota broker. Discovery does not eat the verification reserve.
- Link to the source; never republish fare content.
- Fares are not guaranteed until ticketed.

See [../docs/08-agent-fleet.md](../docs/08-agent-fleet.md) for the schemas, cadence and budget.
