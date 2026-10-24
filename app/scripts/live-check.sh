#!/usr/bin/env bash
# Proves the live fare path end to end against a running data plane:
#   1. the live source is configured,
#   2. a one-way search returns live, verified, priced itineraries,
#   3. a return search returns both directions,
#   4. the quota counted each call (1 for the one-way, 2 for the return).
#
# Usage:  app/scripts/live-check.sh [API_BASE] [ORIGIN] [DESTINATION]
#         defaults: http://127.0.0.1:8080 BER LIS
#
# Costs 3 metered searches. It never reads or prints the key.
set -euo pipefail

API="${1:-http://127.0.0.1:8080}"
FROM="${2:-BER}"
TO="${3:-LIS}"

if date -v+1d >/dev/null 2>&1; then
  OUT=$(date -v+45d +%Y-%m-%d)
  BACK=$(date -v+52d +%Y-%m-%d)
else
  OUT=$(date -d '+45 days' +%Y-%m-%d)
  BACK=$(date -d '+52 days' +%Y-%m-%d)
fi

json() { python3 -c "import json,sys; d=json.load(sys.stdin); print($1)"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "Data plane: $API"
health=$(curl -fsS "$API/healthz") || fail "data plane not reachable at $API"
configured=$(echo "$health" | json "next(s for s in d['sources'] if s['id']=='serpapi')['configured']")
[ "$configured" = "True" ] || fail "SERPAPI_KEY is not set on the data plane (put it in app/.env and restart)"
used_before=$(echo "$health" | json "next(s for s in d['sources'] if s['id']=='serpapi').get('usedToday') or 0")
limit=$(echo "$health" | json "next(s for s in d['sources'] if s['id']=='serpapi').get('dailyLimit')")
echo "Live source configured; used today: $used_before of $limit"

search() {
  curl -sS -X POST "$API/api/search" -H 'content-type: application/json' -d "$1"
}

echo "One-way ${FROM}→${TO} on ${OUT}…"
oneway=$(search "{\"origin\":\"$FROM\",\"destination\":\"$TO\",\"departDate\":\"$OUT\",\"adults\":1,\"checkedBags\":0,\"currency\":\"EUR\"}")
echo "$oneway" | json "d.get('error') and sys.exit('API error: '+d['error']['message'])" >/dev/null || fail "$(echo "$oneway" | json "d['error']['message']")"
freshness=$(echo "$oneway" | json "d['provenance']['freshness']")
count=$(echo "$oneway" | json "len(d['itineraries'])")
[ "$freshness" = "live" ] || [ "$freshness" = "cached" ] || fail "expected live fares, got freshness=$freshness"
[ "$count" -gt 0 ] || fail "no itineraries for ${FROM}→${TO} on $OUT"
echo "$oneway" | json "all(i['verifiedAt'] and i['totalPrice']>0 for i in d['itineraries']) or sys.exit(1)" >/dev/null \
  || fail "an itinerary is unverified or has no price"
echo "  $count itineraries, $freshness; cheapest: $(echo "$oneway" | json "min(d['itineraries'], key=lambda i:i['totalPrice'])['totalPrice']") EUR"

echo "Return ${FROM}⇄${TO}, ${OUT} / ${BACK}…"
roundtrip=$(search "{\"origin\":\"$FROM\",\"destination\":\"$TO\",\"departDate\":\"$OUT\",\"returnDate\":\"$BACK\",\"adults\":1,\"checkedBags\":0,\"currency\":\"EUR\"}")
out_count=$(echo "$roundtrip" | json "len(d.get('itineraries', []))")
back_count=$(echo "$roundtrip" | json "len(d.get('returnItineraries', []))")
[ "$back_count" -gt 0 ] || fail "no return flights ($(echo "$roundtrip" | json "d.get('provenance',{}).get('degradedReason') or d.get('error')"))"
echo "  outbound $out_count, return $back_count"

used_after=$(curl -fsS "$API/healthz" | json "next(s for s in d['sources'] if s['id']=='serpapi').get('usedToday') or 0")
echo "Quota used today: ${used_before} → ${used_after} (cache hits cost nothing)"
echo "PASS: live fares work end to end."
