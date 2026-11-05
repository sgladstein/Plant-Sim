# Validation

Scripts that generate reference fixtures and (from WP11) the validation report.

```sh
python -m venv .venv && . .venv/bin/activate
pip install -r validation/requirements.txt
python validation/fixtures/solar_pvlib.py   # WP2: sun position reference (pvlib SPA)
```

Fixtures are committed. Regenerate them only by running these scripts, and say why in the PR.
