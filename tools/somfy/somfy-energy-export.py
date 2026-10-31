#!/usr/bin/env python3
"""Preview a TaHoma state-values CSV, or emit JSONL for `moli-os energy import`.

No database access. The caller explicitly maps each file to a configured meter.
Daily exports contain no timezone or day-boundary convention: never guess either.
"""
import argparse
import csv
from datetime import date, datetime, time, timedelta, timezone
from decimal import Decimal, InvalidOperation
import json
import math
from pathlib import Path
import sys
from zoneinfo import ZoneInfo

HOUR = 3_600_000
MAX_BYTES = 20 * 1024 * 1024
MAX_ROWS = 250_000
STATE = 'core:ElectricEnergyConsumptionState in Wh'


def number(text):
    value = Decimal(text)
    if not value.is_finite() or not math.isfinite(float(value)) or value < 0:
        raise ValueError('expected a finite, nonnegative Wh index')
    return value


def read_export(path):
    """Keep the recent and daily series separate, selecting only `device`."""
    if path.stat().st_size > MAX_BYTES:
        raise ValueError('CSV exceeds 20 MiB')
    recent, daily = [], []
    header = None
    with path.open(encoding='utf-8-sig', newline='') as source:
        rows = csv.reader(source)
        title = next(rows, [])
        state = next(rows, [])
        if len(title) != 1 or state != [STATE]:
            raise ValueError('not an electrical energy export in Wh')
        for line, row in enumerate(rows, 3):
            if line > MAX_ROWS:
                raise ValueError('CSV exceeds row limit')
            if not row or all(not c.strip() for c in row):
                continue
            try:
                if not row[0].strip():
                    if header is not None or row.count('device') != 1:
                        raise ValueError('invalid daily header')
                    header = row
                elif header is None:
                    if len(row) != 2:
                        raise ValueError('expected timestamp,index')
                    stamp = datetime.strptime(row[0], '%Y-%m-%d %H:%M:%S')
                    recent.append((stamp, number(row[1])))
                else:
                    if len(row) != len(header):
                        raise ValueError('daily column count does not match header')
                    stamp = datetime.strptime(row[0], '%Y-%m-%d').date()
                    daily.append((stamp, number(row[header.index('device')])))
            except (ValueError, InvalidOperation) as error:
                raise ValueError(f'line {line}: {error}') from error
    return {'recent': recent, 'daily': daily}


def utc_ms(local, zone):
    """Reject ambiguous/nonexistent wall times instead of guessing a DST fold."""
    possibilities = set()
    for fold in (0, 1):
        aware = local.replace(tzinfo=zone, fold=fold)
        utc = aware.astimezone(timezone.utc)
        if utc.astimezone(zone).replace(tzinfo=None) == local:
            possibilities.add(int(utc.timestamp()) * 1000)
    if len(possibilities) != 1:
        raise ValueError(f'ambiguous or nonexistent local timestamp: {local}')
    stamp = possibilities.pop()
    if stamp < 0:
        raise ValueError('timestamps before epoch are unsupported')
    return stamp


def convert(samples, zone, anchor=None):
    """Difference cumulative indexes; hourly slices are time-proportional estimates."""
    readings = {}
    duplicates = 0
    for stamp, value in samples:
        if isinstance(stamp, date) and not isinstance(stamp, datetime):
            if anchor not in ('start', 'end'):
                raise ValueError('--daily-anchor start|end is required for daily dates')
            stamp = datetime.combine(stamp, time())
            if anchor == 'end':
                stamp += timedelta(days=1)
        ts = utc_ms(stamp, zone)
        if ts in readings:
            if readings[ts] != value:
                raise ValueError('conflicting indexes at the same timestamp')
            duplicates += 1
        readings[ts] = value
    hours = {}
    skipped = []
    report = {'duplicates': duplicates, 'decreases': 0, 'gaps': 0, 'implausible': 0}
    ordered = sorted(readings.items())
    for (start, a), (end, b) in zip(ordered, ordered[1:]):
        delta = (b - a) / 1000
        if delta < 0:
            report['decreases'] += 1
            skipped.append((start, end))
            continue  # A correction/reset needs review, never a guessed rollover.
        if end - start > 48 * HOUR:
            report['gaps'] += 1
            skipped.append((start, end))
            continue
        if delta > Decimal(500) * (end - start) / HOUR:
            report['implausible'] += 1
            skipped.append((start, end))
            continue
        current = start
        while current < end:
            hour = current // HOUR * HOUR
            until = min(end, hour + HOUR)
            hours[hour] = hours.get(hour, Decimal(0)) + delta * (until - current) / (end - start)
            current = until
    # First/last partial hours cannot safely be combined with another export.
    # Store.import independently rejects hours overlapping live recording.
    if ordered:
        first, last = ordered[0][0], ordered[-1][0]
        hours = {h: k for h, k in hours.items() if h >= first and h + HOUR <= last
                 and not any(h < b and h + HOUR > a for a, b in skipped)}
    report['estimated_hours'] = len(hours)
    return hours, report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('files', nargs='+', metavar='METER=CSV')
    parser.add_argument('--series', choices=['daily', 'recent'], default='daily')
    parser.add_argument('--timezone', help='Explicit IANA zone of the export, not the Mac zone')
    parser.add_argument('--daily-anchor', choices=['start', 'end'], help='Index at start/end of its dated day; confirm with the source')
    parser.add_argument('--emit', action='store_true', help='Emit JSONL only; never writes to Moli')
    args = parser.parse_args(argv)
    if args.emit and (not args.timezone or (args.series == 'daily' and not args.daily_anchor)):
        parser.error('--emit requires --timezone and, for daily data, --daily-anchor')
    try:
        zone = ZoneInfo(args.timezone) if args.timezone else None
        output = {}
        for mapping in args.files:
            meter, separator, filename = mapping.partition('=')
            if not separator or not meter.strip() or len(meter) > 128:
                raise ValueError('expected METER=CSV with a configured meter ID')
            series = read_export(Path(filename))
            report = {key: len(value) for key, value in series.items()}
            samples = series[args.series]
            report.update(first=str(min((s for s, _ in samples), default='')), last=str(max((s for s, _ in samples), default='')))
            ready = zone is not None and (args.series == 'recent' or args.daily_anchor is not None)
            if ready:
                hours, counts = convert(samples, zone, args.daily_anchor)
                report.update(counts)
                for hour, value in hours.items():
                    key = (meter, hour)
                    if key in output and output[key] != value:
                        raise ValueError('conflicting overlapping hours; do not combine series or partial exports')
                    output[key] = value
            else:
                report['conversion'] = 'timezone/day-boundary confirmation required'
            print(json.dumps({'meter': meter, **report}, ensure_ascii=False), file=sys.stderr)
        # Validate every file before emitting anything. No partially valid import stream.
        if args.emit:
            for (meter, hour), kwh in sorted(output.items()):
                print(json.dumps({'meter': meter, 'hour': hour, 'kwh': float(kwh)}))
        return 0
    except (OSError, ValueError, InvalidOperation, csv.Error, OverflowError, KeyError) as error:
        print(f'CSV: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
