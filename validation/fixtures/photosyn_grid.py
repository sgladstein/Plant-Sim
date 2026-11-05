"""Shared test grid and bean parameters for the gas-exchange fixture (design plan §8.1).

Both generators read this file: photosyn_plantecophys.R (the reference, through
plantecophys::Photosyn) and photosyn_reference.py (an independent implementation of
the same published equations, used when R is unavailable).
"""

PAR = [50, 100, 200, 400, 800, 1200, 1600, 2000]          # absorbed PAR, umol m-2 s-1
TLEAF = [10, 15, 20, 25, 30, 35, 40]                      # degC
VPD = [0.5, 1.0, 1.5, 2.0, 3.0, 4.0]                      # kPa
CA = [280, 420, 800]                                      # ppm

# Bush bean, data/species/bush_bean.ron (values copied; the Rust test reads the RON).
PARAMS = dict(
    Vcmax=89.3, Jmax=159.8, Rd0=0.88, Q10=1.92, TrefR=25.0, g1=4.0, g0=0.01,
    alpha=0.3, theta=0.7, Patm=100.0,
    EaV=58550.0, EdVC=200000.0, delsC=629.26,
    EaJ=29680.0, EdVJ=200000.0, delsJ=631.88,
)

COLUMNS = ["par", "tleaf", "vpd", "ca", "a_net", "gs", "e", "ci"]
