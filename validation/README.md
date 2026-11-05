# Validation

Scripts that generate reference fixtures and (from WP11) the validation report.

```sh
python -m venv .venv && . .venv/bin/activate
pip install -r validation/requirements.txt
python validation/fixtures/solar_pvlib.py   # WP2: sun position reference (pvlib SPA)
```

Fixtures are committed. Regenerate them only by running these scripts, and say why in the PR.

## The validation report (WP11)

```sh
cargo run --release -p ps-cli -- validate        # runs the scenarios, writes out/validation/
python3 validation/report.py out/validation      # plots and report.html
```

`ps-cli validate` reads the reference data in `data/validation/` (schema in
docs/validation-data.md). It runs every scenario the data names, plus `single_pot` and
the four response scenarios, and evaluates each target and response. It writes:

- `report.md` and `results.csv`, one row per check.
- `curves/<id>.csv`, each reference curve next to the sim's values.
- `runs/<scenario>/`, the CSVs of each run, including the organ table at day 45.

It exits non-zero when an M0-gated check misses and is not listed in
`validation/known_misses.ron`. CI runs it in the `validation` job and uploads the report
as the `validation-report` artifact. The job summary shows the Markdown report.

`report.py` draws three kinds of figure: the response scenarios over time, a 3D view of
the organ graph at day 45, and each reference curve against the sim. It only plots;
the verdicts come from ps-cli.
