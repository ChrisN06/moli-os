"""Synthetic fixtures only; no house exports or databases are read here."""
import contextlib
from datetime import date, datetime
from decimal import Decimal
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from zoneinfo import ZoneInfo

SPEC = importlib.util.spec_from_file_location('exporter', Path(__file__).with_name('somfy-energy-export.py'))
exporter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(exporter)
UTC = ZoneInfo('UTC')

CSV = '''Example energy
core:ElectricEnergyConsumptionState in Wh

2024-01-01 00:00:00,1000
2024-01-01 01:00:00,2000

,device,heatingSystem,electric
2024-01-01,1000,1000,999999
2024-01-02,24000,24000,999999
'''


class ExportTests(unittest.TestCase):
    def test_real_structure_uses_device_not_repeated_total(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'example.csv'
            path.write_text('\ufeff' + CSV)
            data = exporter.read_export(path)
            self.assertEqual([v for _, v in data['daily']], [1000, 24000])
            hours, report = exporter.convert(data['daily'], UTC, 'start')
            self.assertAlmostEqual(float(sum(hours.values())), 23)
            self.assertEqual(report['estimated_hours'], 24)

    def test_dst_days_have_23_or_25_actual_hours(self):
        for start, end, count in [(date(2024, 3, 31), date(2024, 4, 1), 23),
                                  (date(2024, 10, 27), date(2024, 10, 28), 25)]:
            hours, _ = exporter.convert([(start, Decimal(0)), (end, Decimal(count * 1000))], ZoneInfo('Europe/Paris'), 'start')
            self.assertEqual(len(hours), count)
            self.assertTrue(all(value == 1 for value in hours.values()))

    def test_day_boundary_is_explicit_and_ambiguous_wall_times_fail(self):
        with self.assertRaises(ValueError):
            exporter.convert([(date(2024, 1, 1), Decimal(0))], UTC)
        for stamp in [datetime(2024, 10, 27, 2, 30), datetime(2024, 3, 31, 2, 30)]:
            with self.assertRaises(ValueError):
                exporter.utc_ms(stamp, ZoneInfo('Europe/Paris'))
        start, _ = exporter.convert([(date(2024, 1, 1), Decimal(0)), (date(2024, 1, 2), Decimal(24000))], UTC, 'start')
        end, _ = exporter.convert([(date(2024, 1, 1), Decimal(0)), (date(2024, 1, 2), Decimal(24000))], UTC, 'end')
        self.assertEqual(min(end) - min(start), 24 * exporter.HOUR)

    def test_duplicates_conflicts_resets_and_gaps(self):
        def reading(day, amount):
            return date(2024, 1, day), Decimal(amount)
        hours, report = exporter.convert([reading(1, 1000), reading(1, 1000), reading(2, 25000), reading(3, 0), reading(4, 24000), reading(10, 50000)], UTC, 'start')
        self.assertEqual(len(hours), 48)
        self.assertEqual(report['duplicates'], 1)
        self.assertEqual(report['decreases'], 1)
        self.assertEqual(report['gaps'], 1)
        with self.assertRaises(ValueError):
            exporter.convert([reading(1, 1000), reading(1, 2000)], UTC, 'start')

    def test_unsafe_values_and_other_sensor_units_fail(self):
        for value in ['NaN', 'Infinity', '-1', '1e9999']:
            with self.assertRaises(ValueError):
                exporter.number(value)
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'example.csv'
            path.write_text(CSV.replace('core:ElectricEnergyConsumptionState in Wh', 'core:TemperatureState in °C'))
            with self.assertRaises(ValueError):
                exporter.read_export(path)

    def test_partial_hours_and_skipped_intervals_are_not_filled(self):
        def reading(hour, minute, amount):
            return datetime(2024, 1, 1, hour, minute), Decimal(amount)
        hours, report = exporter.convert([reading(0, 30, 1000), reading(1, 0, 2000),
                                         reading(1, 30, 3000), reading(1, 45, 0),
                                         reading(2, 0, 1000), reading(3, 0, 2000),
                                         reading(3, 30, 3000)], UTC)
        self.assertEqual(len(hours), 1)  # Only the complete 02:00–03:00 interval.
        self.assertEqual(report['decreases'], 1)
        self.assertEqual(next(iter(hours.values())), 1)

    def test_conflicting_files_fail_before_any_jsonl_is_emitted(self):
        with tempfile.TemporaryDirectory() as folder:
            first, second = Path(folder) / 'a.csv', Path(folder) / 'b.csv'
            first.write_text(CSV)
            second.write_text(CSV.replace('01:00:00,2000', '01:00:00,3000'))
            stdout, stderr = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                code = exporter.main(['--emit', '--series', 'recent', '--timezone', 'UTC',
                                      f'example={first}', f'example={second}'])
            self.assertEqual(code, 2)
            self.assertEqual(stdout.getvalue(), '')
            self.assertIn('conflicting overlapping hours', stderr.getvalue())

    def test_preview_emits_nothing_and_repeated_files_are_deduplicated(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'example.csv'
            path.write_text(CSV)
            stdout, stderr = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                self.assertEqual(exporter.main([f'example={path}']), 0)
            self.assertEqual(stdout.getvalue(), '')
            self.assertIn('confirmation required', stderr.getvalue())
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                self.assertEqual(exporter.main(['--emit', '--series', 'recent', '--timezone', 'UTC', f'example={path}', f'example={path}']), 0)
            lines = stdout.getvalue().splitlines()
            self.assertEqual(len(lines), 1)
            self.assertEqual(json.loads(lines[0])['kwh'], 1)
            # No import or database-writing command exists in the exporter.
            self.assertEqual([p.name for p in Path(folder).iterdir()], ['example.csv'])


if __name__ == '__main__':
    unittest.main()
