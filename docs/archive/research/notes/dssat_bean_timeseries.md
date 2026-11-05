# DSSAT CROPGRO bean measured time series (Phaseolus vulgaris)

## Provenance
- Source: GitHub repo DSSAT/dssat-csm-data, folders `Drybean/` and `GreenBean/`, obtained 2026-11-05 by `git clone --depth 1 --filter=blob:none --sparse https://github.com/DSSAT/dssat-csm-data` (git over HTTPS worked; API/raw not needed). Commit 99c95b340acb9515c2d0a8ab582f5fbbf076f345 (2026-09-30). Weather file `Weather/UFGA8501.WTH` fetched from the same commit.
- Raw copies of all 6 Drybean experiments (.BNX/.BNT/.BNA), the GreenBean UFGA0701 (.GBX/.GBT/.GBA) and UFGA8501.WTH: `scratchpad/research/dssat_raw/`.
- Licence: dssat-csm-data has **no LICENSE file** anywhere in the tree and the README does not state one (so formally "all rights reserved"/unspecified; attribute DSSAT Foundation and the original experimenters, and ask DSSAT before redistributing). The sister code repo DSSAT/dssat-csm-os carries `license.txt` = BSD-3-Clause, Copyright (c) 2021 DSSAT Foundation; it does not explicitly cover the data repo.
- Cultivar habits from BNGRO048.CUL ECO codes: MESDET/ANDDET = determinate (Type I), MESIND/ANDIND = indeterminate.

## Experiments in the Drybean folder (all of them)
| Exp | Site, year | Cultivars (habit) | Factors | Sampling |
|---|---|---|---|---|
| UFGA8501 | Gainesville FL (Irrigation Park, 29.63N -82.37W), 1985, planted 1985-07-01 (85182) | Isabella (ANDDET), Manitou (ANDDET), **Seafarer (MESDET)**, C-20 (MESIND) | irrigated vs nonirrigated | 6 dates, 32-88 DAP |
| UFGA8601 | Gainesville FL, 1986 | Porrillo Sintetico, BAT 477 (both MESIND) | 5 irrigation regimes | 12 dates; has CHTD |
| CCPA8629 | CIAT Palmira, Colombia, 1986 | Porrillo Sint., BAT 477, BAT 881 (MESIND) | 2 row widths x 2 densities | 32 dates |
| CCPA9001 | CIAT Palmira, 1990, planted 1990-03-02, irrigated (40+50+45 mm) | 16 cvs incl. **Redkloud (ANDDET)**, Canadian Wonder, A 195, A 486, BAT 1393, WAF 9, Jatu Rong (ANDDET), Brasil 2 (MESDET) | cultivar only | ~10 dates, 11-73 DAP |
| CCPO9002 | CIAT Popayan, Colombia, 1990, planted 1990-04-25 | same 16 cvs | cultivar only | ~9 dates |
| IGQU8903 | ICTA site, Guatemala (14.27N, -90W, farmer's field), 1989 | ICTA-Ostua, Rabia de Gato, Turbo III (MESIND) | cultivar | 1-2 dates only |
| GreenBean UFGA0701 | Gainesville FL, spring 2007, planted 2007-03-15 | snap bean Bronco (Habit 1) | N rate x harvest stage | 10 dates; has CHTD, CWID |

No file contains a V-stage column (VSTD); `L#SD` = leaf number on main stem (closest proxy). Height (CHTD, m) exists only in UFGA8601 (indeterminate) and GreenBean UFGA0701.

## Column codes / units
DAP = days after planting. L#SD main-stem leaf no.; LAID m2/m2; P#AD pods/m2; G#AD seeds/m2; LWAD leaf, SWAD stem (+petiole), PWAD pod (shell+seed), GWAD seed, SHAD shell, CWAD total tops (above-ground) dry mass, all kg/ha; HIAD harvest index; SH%D shelling %; SLAD specific leaf area cm2/g; GWGD mg/seed; CHTD canopy height m; CWID canopy width m.
Derived: `*_gpl` = g/plant = kg/ha / 10 / PPOP; `LA_cm2pl` = LAID / PPOP x 1e4 (cm2 leaf per plant). Leaf mass excludes abscised leaves (LWAD -> 0 at maturity). -99 = missing.

## 1. UFGA8501 Seafarer (Meso Type I, navy bean), Gainesville 1985
PPOP 51.7 plants/m2, rows 38 cm, sandy soil (IBBN910016). Trt 5 irrigated: 8 irrigations, 100 mm total (85184-85267). Trt 6 "nonirrigated": only the 2 establishment irrigations (20 mm). Season rain (planting to 85270) = 561 mm, so drought was mild (final yield 1811 vs 1741 kg/ha). BNA: anthesis 214/217 (DOY), maturity 247/250, seed size 0.161/0.171 g.

### Trt 5 Seafarer irrigated
```csv
DATE,DAP,L#SD,LAID,P#AD,SWAD,GWAD,LWAD,CWAD,PWAD,SHAD,G#AD,HIAD,SH%D,SLAD,GWGD,LW_gpl,SW_gpl,PW_gpl,GW_gpl,SH_gpl,CW_gpl,LA_cm2pl
1985-08-02,32,8.00,1.73,0.0,379.5,0.0,527.8,907.3,0.0,0.0,0.0,0.000,0.0,332.0,0.0,1.021,0.734,0.000,0.000,0.000,1.755,334.6
1985-08-14,44,9.83,3.27,6.7,1105,3.7,927.4,2252,219.9,216.3,69.6,0.002,1.6,351.6,6.7,1.794,2.137,0.425,0.007,0.418,4.356,632.5
1985-08-26,56,9.25,3.51,104.9,1543,471.4,1072,3731,1115,643.9,725.2,0.126,37.2,328.3,122.0,2.074,2.985,2.157,0.912,1.245,7.217,678.9
1985-09-06,67,9.33,0.69,198.0,1102,1445,198.5,3315,2015,570.1,1003,0.436,70.6,370.0,145.0,0.384,2.132,3.897,2.795,1.103,6.412,133.5
1985-09-17,78,9.25,0.00,230.5,980.5,1802,21.7,3307,2305,502.6,1121,0.545,78.0,0.0,164.3,0.042,1.897,4.458,3.485,0.972,6.397,0.0
1985-09-27,88,9.25,0.00,231.7,986.5,1811,0.0,3303,2317,505.2,1127,0.548,78.0,0.0,164.3,0.000,1.908,4.482,3.503,0.977,6.389,0.0
```

### Trt 6 Seafarer nonirrigated (mild drought)
```csv
DATE,DAP,L#SD,LAID,P#AD,SWAD,GWAD,LWAD,CWAD,PWAD,SHAD,G#AD,HIAD,SH%D,SLAD,GWGD,LW_gpl,SW_gpl,PW_gpl,GW_gpl,SH_gpl,CW_gpl,LA_cm2pl
1985-08-02,32,8.00,1.31,0.0,278.0,0.0,479.0,756.9,0.0,0.0,0.0,0.000,0.0,276.7,0.0,0.926,0.538,0.000,0.000,0.000,1.464,253.4
1985-08-14,44,9.33,2.98,0.0,888.4,0.0,922.4,1827,16.3,16.3,0.0,0.000,0.0,323.9,0.0,1.784,1.718,0.032,0.000,0.032,3.534,576.4
1985-08-26,56,9.33,3.21,67.1,1386,264.0,979.8,3141,775.0,511.1,688.5,0.084,33.8,333.7,37.7,1.895,2.681,1.499,0.511,0.989,6.075,620.9
1985-09-06,67,8.83,1.89,141.1,1401,870.6,486.5,3313,1425,554.4,688.6,0.263,60.3,395.2,125.2,0.941,2.710,2.756,1.684,1.072,6.408,365.6
1985-09-17,78,9.75,0.00,233.1,1018,1741,0.2,3350,2332,590.3,1021,0.520,74.7,0.0,171.2,0.000,1.969,4.511,3.368,1.142,6.480,0.0
1985-09-27,88,9.73,0.00,233.2,1018,1741,0.0,3350,2332,590.4,1021,0.520,74.7,0.0,171.2,0.000,1.969,4.511,3.368,1.142,6.480,0.0
```

### Trt 1 Isabella irrigated (Andean Type I, light red kidney) - same experiment, for comparison
```csv
DATE,DAP,L#SD,LAID,P#AD,SWAD,GWAD,LWAD,CWAD,PWAD,SHAD,G#AD,HIAD,SH%D,SLAD,GWGD,LW_gpl,SW_gpl,PW_gpl,GW_gpl,SH_gpl,CW_gpl,LA_cm2pl
1985-08-02,32,6.17,1.98,0.0,486.2,0.0,630.4,1117,0.0,0.0,0.0,0.000,0.0,310.5,0.0,1.219,0.940,0.000,0.000,0.000,2.161,383.0
1985-08-14,44,6.92,3.48,35.8,1197,39.5,918.1,2594,478.6,439.1,252.5,0.015,8.2,377.7,15.7,1.776,2.315,0.926,0.076,0.849,5.017,673.1
1985-08-26,56,6.58,3.46,194.9,1658,938.8,864.9,4534,2011,1072,601.5,0.207,46.6,397.0,156.1,1.673,3.207,3.890,1.816,2.074,8.770,669.2
1985-09-06,67,6.58,0.60,229.2,1425,1405,206.2,3948,2317,911.8,558.2,0.356,60.4,349.8,252.7,0.399,2.756,4.482,2.718,1.764,7.636,116.1
1985-09-17,78,6.25,0.00,244.7,1108,1695,0.8,3565,2456,760.8,549.6,0.476,68.9,126.5,307.4,0.002,2.143,4.750,3.279,1.472,6.896,0.0
1985-09-27,88,6.25,0.00,244.7,1108,1696,0.0,3564,2457,760.9,549.7,0.476,68.9,0.0,307.4,0.000,2.143,4.752,3.280,1.472,6.894,0.0
```

### Trt 3 Manitou irrigated (Andean Type I) - same experiment
```csv
DATE,DAP,L#SD,LAID,P#AD,SWAD,GWAD,LWAD,CWAD,PWAD,SHAD,G#AD,HIAD,SH%D,SLAD,GWGD,LW_gpl,SW_gpl,PW_gpl,GW_gpl,SH_gpl,CW_gpl,LA_cm2pl
1985-08-02,32,6.75,1.54,0.0,305.6,0.0,541.0,846.6,0.0,0.0,0.0,0.000,0.0,279.9,0.0,1.046,0.591,0.000,0.000,0.000,1.638,297.9
1985-08-14,44,8.25,4.38,0.0,1192,0.0,1185,2396,19.2,19.2,0.0,0.000,0.0,363.8,0.0,2.292,2.306,0.037,0.000,0.037,4.634,847.2
1985-08-26,56,7.67,3.64,37.2,1601,50.2,1110,3235,523.6,473.4,249.2,0.016,9.4,334.8,18.7,2.147,3.097,1.013,0.097,0.916,6.257,704.1
1985-09-06,67,7.67,2.35,190.0,1658,1107,616.8,4204,1929,822.7,464.3,0.263,56.1,375.8,249.2,1.193,3.207,3.731,2.141,1.591,8.132,454.5
1985-09-17,78,7.08,0.24,239.7,1453,1673,75.3,3945,2417,744.3,452.5,0.424,69.4,304.8,369.8,0.146,2.810,4.675,3.236,1.440,7.631,46.4
1985-09-27,88,7.33,0.01,247.5,1376,1762,2.1,3856,2478,715.7,519.9,0.457,71.2,67.5,341.0,0.004,2.662,4.793,3.408,1.384,7.458,1.9
```

Nonirrigated counterparts: Isabella trt 2, Manitou trt 4, C-20 (indeterminate) trts 7/8 - see raw BNT.

## 2. CCPA9001 Redkloud (Andean Type I, dark red kidney), CIAT Palmira 1990, trt 15
PPOP 25 plants/m2, rows 60 cm, planted 1990-03-02, irrigated 40/50/45 mm on DOY 61/73/89 plus rain; no drought treatment. BNA: ADAT 90 day-of-year (= 29 DAP), MDAT 121 DOY (= 60 DAP; file note says JWJ subtracted 6 d from MDAT to estimate physiological maturity, though measured seed mass still rises to 73 DAP); yield 1732 kg/ha, seed 0.450 g. Note: the BNX @SITE line says "Popayan" but the title, field and weather station (CCPA) are Palmira - looks like a copy error in the file.
```csv
DATE,DAP,L#SD,LAID,P#AD,SWAD,GWAD,LWAD,CWAD,PWAD,SHAD,G#AD,HIAD,SH%D,SLAD,GWGD,LW_gpl,SW_gpl,PW_gpl,GW_gpl,SH_gpl,CW_gpl,LA_cm2pl
1990-03-13,11,2.667,0.183,0.00,20.23,0.00,55.60,75.83,0.00,0.000,0.00,0.000,0,330.2,0,0.222,0.081,0.000,0.000,0.000,0.303,73.2
1990-03-20,18,4.167,0.459,0.00,42.31,0.00,118.2,160.6,0.00,0.000,0.00,0.000,0,387.7,0,0.473,0.169,0.000,0.000,0.000,0.642,183.6
1990-03-27,25,6.167,1.300,0.00,149.2,0.00,302.2,451.4,0.00,0.000,0.00,0.000,0,433.7,0,1.209,0.597,0.000,0.000,0.000,1.806,520.0
1990-04-03,32,6.000,2.463,0.00,420.3,0.00,576.9,997.2,0.00,0.000,0.00,0.000,0,426.0,0,2.308,1.681,0.000,0.000,0.000,3.989,985.2
1990-04-10,39,6.556,2.786,161.6,730.8,2.06,673.9,1546,141.5,139.5,133.0,0.001,1.279,413.3,1.550,2.696,2.923,0.566,0.008,0.558,6.184,1114.4
1990-04-18,47,6.667,2.548,272.8,851.5,152.2,694.9,2495,948.6,796.5,662.3,0.060,15.25,366.3,23.39,2.780,3.406,3.794,0.609,3.186,9.980,1019.2
1990-04-24,53,6.778,2.331,245.8,717.3,602.4,619.7,2880,1543,940.5,564.5,0.211,39.17,378.6,106.7,2.479,2.869,6.172,2.410,3.762,11.520,932.4
1990-04-30,59,6.000,1.485,187.5,697.0,1291,448.6,3177,2023,731.8,427.1,0.406,63.79,329.2,301.9,1.794,2.788,8.092,5.164,2.927,12.708,594.0
1990-05-08,67,6.000,0.495,140.0,671.8,1548,177.4,3027,2144,596.4,379.6,0.512,72.22,280.8,410.4,0.710,2.687,8.576,6.192,2.386,12.108,198.0
1990-05-14,73,6.000,0.358,121.7,578.1,1668,171.2,2956,2202,533.3,405.3,0.563,75.71,208.8,410.9,0.685,2.312,8.808,6.672,2.133,11.824,143.2
```

## 3. CCPO9002 Redkloud, CIAT Popayan (cooler, higher altitude) 1990, trt 15
PPOP 25 plants/m2, rows 50 cm, planted 1990-04-25 (90115), irrigation 3 x 15 mm late season. BNA: ADAT 151 DOY (36 DAP), MDAT 192 DOY (77 DAP), yield 2805 kg/ha, seed 0.556 g.
```csv
DATE,DAP,L#SD,LAID,P#AD,SWAD,GWAD,LWAD,CWAD,PWAD,SHAD,G#AD,HIAD,SH%D,SLAD,GWGD,LW_gpl,SW_gpl,PW_gpl,GW_gpl,SH_gpl,CW_gpl,LA_cm2pl
1990-05-09,14,2.04,0.28,0.000,28.10,0.00,70.54,98.63,0.00,0.00,0.00,0.000,0.0,402.3,0.0,0.282,0.112,0.000,0.000,0.000,0.395,112.0
1990-05-15,20,3.86,0.50,0.000,63.75,0.00,147.0,210.8,0.00,0.00,0.00,0.000,0.0,341.0,0.0,0.588,0.255,0.000,0.000,0.000,0.843,200.0
1990-05-23,28,5.166,0.72,0.000,155.1,0.00,242.4,397.5,0.00,0.00,0.000,0.000,0.0,299.2,0.0,0.970,0.620,0.000,0.000,0.000,1.590,288.0
1990-06-05,41,5.750,1.73,17.59,516.3,0.00,551.2,1069,1.97,1.97,0.000,0.000,0.0,313.3,0.0,2.205,2.065,0.008,0.000,0.008,4.276,692.0
1990-06-21,57,6.333,2.38,310.0,1015,75.71,797.7,2836,1022,946.7,653.5,0.026,7.3,299.0,11.6,3.191,4.060,4.088,0.303,3.787,11.344,952.0
1990-06-28,64,6.000,2.14,222.3,1003,443.5,730.0,3522,1789,1345,626.8,0.135,25.1,291.8,78.8,2.920,4.012,7.156,1.774,5.380,14.088,856.0
1990-07-04,70,6.833,2.09,179.0,854.4,1482,698.4,4205,2647,1165,541.5,0.355,56.1,297.8,276.0,2.794,3.418,10.588,5.928,4.660,16.820,836.0
1990-07-17,83,5.555,0.50,139.7,688.7,2348,161.2,4023,3169,821.1,474.7,0.584,73.8,339.9,492.4,0.645,2.755,12.676,9.392,3.284,16.092,200.0
```

## Bonus: GreenBean UFGA0701 snap bean Bronco (Habit 1 determinate), Gainesville spring 2007, trt 5 (high N, optimal irrigation, grown to dry seed)
PPOP 34 plants/m2, rows 61 cm, planted 2007-03-15 (07074). Includes canopy height CHTD (m). Rows with only NIAD/NLCC are soil-N samples. Trt 4 is identical up to fresh harvest (07136). File note: N-rate data unpublished, "ask Djidonou and Boote".
```csv
DATE,DAP,CHTD,CWID,L#SD,SH%D,GWGD,CWAD,LWAD,SWAD,PWAD,GWAD,SLAD,LAID,P#AD,G#AD,HIAD,HIPD,NUPC,NIAD,NLCC,LW_gpl,SW_gpl,PW_gpl,GW_gpl,CW_gpl,LA_cm2pl
2007-03-27,12,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,137.7,10.1,NA,NA,NA,NA,NA,NA
2007-03-29,14,0.083,0.067,0,0.0,0.0,46,36,10,0,0,190.1,0.06,0,0,0.000,0.000,2.3,-99,-99,0.106,0.029,0.000,0.000,0.135,17.6
2007-04-04,20,0.117,0.141,2,0.0,0.0,161,134,27,0,0,201.0,0.27,0,0,0.000,0.000,9.4,-99,-99,0.394,0.079,0.000,0.000,0.474,79.4
2007-04-11,27,0.158,0.170,3,0.0,0.0,269,196,73,0,0,205.2,0.40,0,0,0.000,0.000,13.6,-99,-99,0.576,0.215,0.000,0.000,0.791,117.6
2007-04-18,34,0.205,0.263,5,0.0,0.0,602,400,202,0,0,181.6,0.72,0,0,0.000,0.000,27.9,-99,-99,1.176,0.594,0.000,0.000,1.771,211.8
2007-04-25,41,0.276,0.312,6,0.0,0.0,1128,697,431,0,0,191.5,1.34,0,0,0.000,0.000,46.8,-99,-99,2.050,1.268,0.000,0.000,3.318,394.1
2007-04-27,43,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,149.8,10.9,NA,NA,NA,NA,NA,NA
2007-05-04,50,0.364,0.396,7,0.0,0.0,1830,1034,718,78,0,192.4,1.93,668,0,0.000,0.042,66.3,-99,-99,3.041,2.112,0.229,0.000,5.382,567.6
2007-05-09,55,0.355,0.421,6,1.8,1.4,2469,1022,847,601,14,212.0,2.16,624,728,0.006,0.243,84.8,-99,-99,3.006,2.491,1.768,0.041,7.262,635.3
2007-05-16,62,0.386,0.472,6,13.5,16.2,3401,974,901,1526,210,224.9,2.19,554,1316,0.062,0.449,111.7,-99,-99,2.865,2.650,4.488,0.618,10.003,644.1
2007-05-23,69,0.358,0.446,7,31.0,45.7,4114,887,852,2375,762,213.1,1.86,482,1614,0.185,0.577,119.6,-99,-99,2.609,2.506,6.985,2.241,12.100,547.1
2007-05-30,76,0.376,0.441,7,63.9,153.7,5181,621,901,3659,2352,217.0,1.32,386,1549,0.454,0.706,146.2,-99,-99,1.826,2.650,10.762,6.918,15.238,388.2
2007-06-04,81,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,-99,176.5,41.7,NA,NA,NA,NA,NA,NA
2007-06-06,83,0.328,0.366,7,68.6,205.5,4459,283,753,3423,2439,253.9,0.69,387,1140,0.547,0.768,121.7,-99,-99,0.832,2.215,10.068,7.174,13.115,202.9
```
