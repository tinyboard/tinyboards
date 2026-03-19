#!/bin/bash
# tinyboards manual partition creation script
#
# Creates monthly partitions for the notifications table.
# The backend's scheduled task handles this automatically, but this script
# lets operators create partitions manually if needed.
#
# Usage:
#   ./create-partitions.sh 2026 04                       # Uses DATABASE_URL from env
#   ./create-partitions.sh 2026 04 "postgresql://..."    # Uses provided connection string
#
# This creates: notifications_2026_04

set -euo pipefail

YEAR="${1:-}"
MONTH="${2:-}"
DATABASE_URL="${3:-${DATABASE_URL:-}}"

if [ -z "$YEAR" ] || [ -z "$MONTH" ]; then
    echo "Usage: $0 <year> <month> [DATABASE_URL]"
    echo "Example: $0 2026 04"
    exit 1
fi

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: No database URL provided."
    echo "Usage: $0 <year> <month> [DATABASE_URL]"
    echo "Or set the DATABASE_URL environment variable."
    exit 1
fi

# Zero-pad month
MONTH=$(printf "%02d" "$MONTH")

# Calculate next month for the range end
if [ "$MONTH" = "12" ]; then
    NEXT_YEAR=$((YEAR + 1))
    NEXT_MONTH="01"
else
    NEXT_YEAR="$YEAR"
    NEXT_MONTH=$(printf "%02d" $((10#$MONTH + 1)))
fi

RANGE_START="${YEAR}-${MONTH}-01"
RANGE_END="${NEXT_YEAR}-${NEXT_MONTH}-01"

# Only notifications is partitioned
TABLES="notifications"

for TABLE in $TABLES; do
    PARTITION_NAME="${TABLE}_${YEAR}_${MONTH}"
    echo "[$(date)] Creating partition: $PARTITION_NAME (${RANGE_START} to ${RANGE_END})..."

    psql "$DATABASE_URL" -c "
        CREATE TABLE IF NOT EXISTS ${PARTITION_NAME}
        PARTITION OF ${TABLE}
        FOR VALUES FROM ('${RANGE_START}') TO ('${RANGE_END}');
    "

    if [ $? -eq 0 ]; then
        echo "[$(date)] Created: $PARTITION_NAME"
    else
        echo "[$(date)] ERROR: Failed to create $PARTITION_NAME"
    fi
done

echo "[$(date)] Done."
