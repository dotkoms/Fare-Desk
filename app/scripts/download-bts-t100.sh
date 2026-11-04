#!/usr/bin/env bash
# Notes and helper for free BTS T-100 Domestic Segment data.
# Public domain. Monthly. Gives which US routes fly and traffic volume
# (useful for Explore coverage beyond our own history).
#
# Data is available via TranStats or the BTS data portal.
# Example direct-style download (adjust year/month; large files):
#   https://transtats.bts.gov/PREZIP/T_T100D_SEGMENT_ALL_CARRIER_YYYY_M.zip
#
# Or use the SODA/API interfaces at https://www.transtats.bts.gov/
# and https://data.bts.gov/
#
# This script prints the typical URL pattern and a suggested workflow.
set -euo pipefail
YEAR="${1:-2025}"
MONTH="${2:-12}"
echo "BTS T-100 Domestic Segment (all carriers) for ${YEAR}-${MONTH}:"
echo "Typical zip pattern:"
echo "  https://transtats.bts.gov/PREZIP/T_T100D_SEGMENT_ALL_CARRIER_${YEAR}_${MONTH}.zip"
echo ""
echo "Workflow:"
echo "  1. Download the monthly zip from TranStats."
echo "  2. Extract the CSV."
echo "  3. Aggregate origin-destination pairs with passenger counts."
echo "  4. Use as a route list / coverage prior for Explore (see docs/25)."
echo ""
echo "Same host as the existing On-Time Performance ingest."
