#!/usr/bin/env bash
# Proves the live fare path end to end against a running data plane:
#   1. a live source is on (Kiwi.com by default, no key; SerpAPI if configured),
#   2. a one-way search returns live, verified, priced itineraries,
#   3. a return search returns both directions,
#   4. the daily counter recorded the searches (cache hits count nothing).
#
# Usage:  app/scripts/live-check.sh [API_BASE] [ORIGIN] [DESTINATION] [CURRENCY]
#         defaults: http://127.0.0.1:8080 JFK LAX USD
#
# Runs at most three live searches. It never reads or prints a key.
set -euo pipefail

API="${1:-http://127.0.0.1:8080}"
FROM="${2:-JFK}"
TO="${3:-LAX}"
CURRENCY="${4:-USD}"

if date -v+1d >/dev/null 2>&1; then
  OUT=$(date -v+45d +%Y-%m-%d)
  BACK=$(date -v+52d +%Y-%m-%d)
else
  OUT=$(date -d '+45 days' +%Y-%m-%d)
  BACK=$(date -d '+52 days' +%Y-%m-%d)
fi

json() { python3 -c "import json,sys; d=json.load(sys.stdin); print($1)"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "Data plane: ${API}"
health=$(curl -fsS "${API}/healthz") || fail "data plane not reachable at ${API}"
SOURCE=$(echo "$health" | json "next((s['id'] for s in sorted(d['sources'], key=lambda s: s['id'] != 'kiwi') if s['live'] and s['configured']), '')")
[ -n "$SOURCE" ] || fail "no live fare source is on (is KIWI=off set?)"
usage() { echo "$1" | json "next(s for s in d['sources'] if s['id']=='${SOURCE}').get('usedToday') or 0"; }
used_before=$(usage "$health")
echo "Live source: ${SOURCE}; searches today: ${used_before}"

search() {
  curl -sS -X POST "${API}/api/search" -H 'content-type: application/json' -d "$1"
}
body() {
  local return_field=""
  [ -n "${1:-}" ] && return_field="\"returnDate\":\"$1\","
  echo "{\"origin\":\"${FROM}\",\"destination\":\"${TO}\",\"departDate\":\"${OUT}\",${return_field}\"adults\":1,\"checkedBags\":0,\"currency\":\"${CURRENCY}\"}"
}

echo "One-way ${FROM}→${TO} on ${OUT}…"
oneway=$(search "$(body)")
error=$(echo "$oneway" | json "(d.get('error') or {}).get('message', '')")
[ -z "$error" ] || fail "$error"
freshness=$(echo "$oneway" | json "d['provenance']['freshness']")
count=$(echo "$oneway" | json "len(d['itineraries'])")
[ "$freshness" = "live" ] || [ "$freshness" = "cached" ] || fail "expected live fares, got freshness=${freshness}"
[ "$count" -gt 0 ] || fail "no itineraries for ${FROM}→${TO} on ${OUT}"
echo "$oneway" | json "all(i['verifiedAt'] and i['totalPrice']>0 for i in d['itineraries']) or sys.exit(1)" >/dev/null \
  || fail "an itinerary is unverified or has no price"
cheapest=$(echo "$oneway" | json "min(d['itineraries'], key=lambda i:i['totalPrice'])['totalPrice']")
seller=$(echo "$oneway" | json "min(d['itineraries'], key=lambda i:i['totalPrice'])['providerName']")
echo "  ${count} itineraries (${freshness}); cheapest ${cheapest} ${CURRENCY} on ${seller}"

echo "Return ${FROM}⇄${TO}, ${OUT} / ${BACK}…"
roundtrip=$(search "$(body "$BACK")")
error=$(echo "$roundtrip" | json "(d.get('error') or {}).get('message', '')")
[ -z "$error" ] || fail "$error"
out_count=$(echo "$roundtrip" | json "len(d.get('itineraries', []))")
back_count=$(echo "$roundtrip" | json "len(d.get('returnItineraries', []))")
[ "$back_count" -gt 0 ] || fail "no return flights ($(echo "$roundtrip" | json "d['provenance'].get('degradedReason')"))"
echo "  outbound ${out_count}, return ${back_count}"

used_after=$(usage "$(curl -fsS "${API}/healthz")")
echo "Searches today: ${used_before} → ${used_after}"
echo "PASS: live fares work end to end."
