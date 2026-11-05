import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from ronw import curve
from dssat import blocks
OUT = sys.argv[1]
b = blocks()
def find(prefix): return next(v for k, v in b.items() if k.startswith(prefix))
exps = [
  ("dssat_ccpa9001_redkloud", "dssat_ccpa9001", find("2. CCPA9001"),
   "CIAT Palmira, Colombia, 1990; Redkloud (Andean Type I, dark red kidney); field, 25 plants/m2, rows 60 cm, irrigated; planted 1990-03-02", 25.0),
  ("dssat_ccpo9002_redkloud", "dssat_ccpo9002", find("3. CCPO9002"),
   "CIAT Popayan, Colombia (cooler, higher altitude), 1990; Redkloud (Andean Type I); field, 25 plants/m2, rows 50 cm; planted 1990-04-25", 25.0),
  ("dssat_ufga8501_seafarer_irrigated", "dssat_ufga8501", find("Trt 5 Seafarer"),
   "Gainesville FL, 1985; Seafarer (Mesoamerican Type I, navy); field, sandy soil, 51.7 plants/m2, rows 38 cm, irrigated (treatment 5, 100 mm); planted 1985-07-01", 51.7),
  ("dssat_ufga8501_seafarer_rainfed", "dssat_ufga8501", find("Trt 6 Seafarer"),
   "Gainesville FL, 1985; Seafarer (Mesoamerican Type I); as treatment 5 but only 20 mm establishment irrigation (treatment 6); 561 mm rain, so a mild drought", 51.7),
  ("dssat_ufga8501_isabella_irrigated", "dssat_ufga8501", find("Trt 1 Isabella"),
   "Gainesville FL, 1985; Isabella (Andean Type I, light red kidney); field, 51.7 plants/m2, irrigated (treatment 1)", 51.7),
  ("dssat_ufga8501_manitou_irrigated", "dssat_ufga8501", find("Trt 3 Manitou"),
   "Gainesville FL, 1985; Manitou (Andean Type I); field, 51.7 plants/m2, irrigated (treatment 3)", 51.7),
  ("dssat_ufga0701_bronco", "dssat_ufga0701", find("Bonus: GreenBean"),
   "Gainesville FL, spring 2007; snap bean Bronco (Habit 1 determinate), high N, optimal irrigation, grown to dry seed (treatment 5); field, 34 plants/m2, rows 61 cm; planted 2007-03-15", 34.0),
]
FIELD = ("Field crop at stand density, so per-plant values are smaller than for a lone pot plant; compare shape and timing, not absolute size. "
         "Per-plant values = kg/ha / 10 / plants per m2.")
vars_ = [  # (column, sim variable, unit, scale, what)
  ("LW_gpl", "leaf_mass_g", "g/plant", 1.0, "LWAD (green leaf, abscised leaves excluded)"),
  ("SW_gpl", "stem_mass_g", "g/plant", 1.0, "SWAD (stem and petioles)"),
  ("PW_gpl", "fruit_mass_g", "g/plant", 1.0, "PWAD (pod walls plus seed)"),
  ("GW_gpl", "seed_mass_g", "g/plant", 1.0, "GWAD (seed)"),
  ("CW_gpl", "shoot_mass_g", "g/plant", 1.0, "CWAD (all above-ground living dry mass)"),
  ("LA_cm2pl", "leaf_area_m2", "m2/plant", 1e-4, "LAID / plants per m2"),
  ("L#SD", "main_nodes", "nodes", 1.0, "L#SD (leaf number on the main stem; DSSAT counts the unifoliate node)"),
  ("CHTD", "height_m", "m", 1.0, "CHTD (canopy height)"),
]
n = 0
for name, src, rows, cond, ppop in exps:
    d = os.path.join(OUT, name); os.makedirs(d, exist_ok=True)
    for col, var, unit, k, what in vars_:
        if col not in rows[0]: continue
        pts = []
        for r in rows:
            v = r.get(col)
            if v in (None, '', 'NA', '-99'): continue
            v = float(v)
            if v == -99: continue
            pts.append((float(r['DAP']), round(v * k, 6), None))
        if not pts: continue
        open(os.path.join(d, var + '.ron'), 'w').write(curve(
            f"{name}_{var}", src, cond, "days_since_sowing", "d", var, unit,
            "Days after planting; planting date taken as sowing day 0. " + FIELD,
            f"Measured time series, DSSAT {src.split('_')[1].upper()} .BNT/.GBT file, column {what}; read from the file",
            pts, comment=f"Measured {var} against days after sowing; DSSAT Drybean experiment {src.split('_')[1].upper()}."))
        n += 1
print(n, "curves")
