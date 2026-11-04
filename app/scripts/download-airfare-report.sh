#!/usr/bin/env bash
# Download the latest US DOT Consumer Airfare Report (Table 1) as CSV.
# Free, public domain data from data.transportation.gov.
# Use as a quarterly baseline for average fares before own history accumulates.
#
# Usage: ./download-airfare-report.sh [output.csv]
set -euo pipefail
OUT="${1:-airfare-report-table1.csv}"
# SODA API endpoint for Table 1 (top 1000 city-pairs). Adjust $limit as needed.
URL="https://data.transportation.gov/resource/4f3n-jbg2.csv?\$limit=10000&\$order=year DESC,quarter DESC"
echo "Downloading US DOT Consumer Airfare Report (Table 1)..."
curl -fsSL -o "$OUT" "$URL"
echo "Saved to $OUT ($(wc -l < "$OUT") lines)"
echo "Columns include year, quarter, city1, city2, fare, passengers, nsmiles, carrier_lg..."
