#!/usr/bin/env python3
"""Prepare an isolated, loopback-only demo. Never overwrites an existing house."""
import argparse
import json
from pathlib import Path
import tempfile
import tomllib


def prepare(root: Path, destination: Path) -> Path:
    destination = destination.resolve()
    destination.mkdir(parents=True, exist_ok=True)
    config = destination / 'moli.toml'
    if config.exists() or (destination / 'home.json').exists():
        raise FileExistsError('Configuration exists; use it as-is or choose a new demo directory.')
    text = f'''# Démonstration synthétique : aucun équipement physique.
[server]
listen = "127.0.0.1:8791"
data_dir = {json.dumps(str(destination))}
language = "fr"
timezone = "Europe/Paris"
worker_threads = 2
history_days = 30
demo = true

[[driver]]
id = "demo"
kind = "demo"
[driver.options]
fixture = {json.dumps(str(root / 'demo/personal/devices.json'))}
tick_s = 15

[energy]
[[energy.meter]]
id = "general"
name = "Général (démo)"
point = "demo:general/energy"
power = "demo:general/power"
role = "total"
[[energy.meter]]
id = "chauffage"
name = "Chauffage (démo)"
point = "demo:chauffage/energy"
power = "demo:chauffage/power"
role = "circuit"
'''
    tomllib.loads(text)
    with config.open('x') as output:
        output.write(text)
    with (destination / 'home.json').open('x') as output:
        json.dump({'title': 'Maison · démonstration', 'navigation': ['','pieces','energie','impression','automatismes'],
                   'favorites': {'climate': [], 'lights': [], 'cameras': []}}, output, ensure_ascii=False, indent=2)
    return config


def self_test(root: Path) -> None:
    with tempfile.TemporaryDirectory() as temporary:
        destination = (Path(temporary) / 'data with spaces').resolve()
        config = prepare(root, destination)
        parsed = tomllib.loads(config.read_text())
        assert parsed['server']['listen'] == '127.0.0.1:8791'
        assert parsed['server']['demo'] is True
        assert all(driver['kind'] == 'demo' for driver in parsed['driver'])
        assert parsed['server']['data_dir'] == str(destination)
        before = config.read_bytes()
        try:
            prepare(root, destination)
        except FileExistsError:
            pass
        else:
            raise AssertionError('Existing configurations must not be overwritten')
        assert config.read_bytes() == before


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--directory', type=Path)
    arguments = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    if arguments.self_test:
        self_test(repo)
        print('prepare-personal-dev: OK')
    else:
        print(prepare(repo, arguments.directory or repo / 'data/development'))
