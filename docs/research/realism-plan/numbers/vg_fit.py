"""R3a check: van Genuchten anchored on the sim's soil constants (crates/ps-soil/src/lib.rs).
(a) spec rule: literature n, solve alpha and theta_r; (b) theta_r = 0, solve n and alpha."""
SOILS = (("loam", .45, .27, .12, .05, 1.56), ("sand", .40, .15, .05, .02, 2.68), ("clay", .50, .40, .22, .10, 1.09))
H_FC, H_WP = 336.6, 15296.0  # cm of water at -33 and -1500 kPa

def bisect(f, lo, hi, it=200, geo=False):
    flo = f(lo)
    for _ in range(it):
        mid = (lo * hi) ** .5 if geo else (lo + hi) / 2
        fm = f(mid)
        if flo * fm <= 0: hi = mid
        else: lo, flo = mid, fm
    return mid

def theta(h, ts, tr, a, n):
    return tr + (ts - tr) * (1 + (a * h) ** n) ** -(1 - 1 / n)

def spec_rule(ts, fc, wp, n):
    m = 1 - 1 / n
    def tr_of(a):
        s1 = (1 + (a * H_FC) ** n) ** -m; s2 = (1 + (a * H_WP) ** n) ** -m
        return ts - (fc - wp) / (s1 - s2), s1
    def g(a):
        tr, s1 = tr_of(a); return tr + (ts - tr) * s1 - fc
    a = bisect(g, 1e-5, 1.0, geo=True)
    return a, tr_of(a)[0]

def zero_tr(ts, fc, wp):
    def a_of(n):
        m = 1 - 1 / n; return ((fc / ts) ** (-1 / m) - 1) ** (1 / n) / H_FC
    n = bisect(lambda n: theta(H_WP, ts, 0, a_of(n), n) - wp, 1.01, 5.0)
    return n, a_of(n)

for name, ts, fc, wp, ad, nlit in SOILS:
    a, tr = spec_rule(ts, fc, wp, nlit)
    n0, a0 = zero_tr(ts, fc, wp)
    m0 = 1 - 1 / n0
    h_ad = ((ad / ts) ** (-1 / m0) - 1) ** (1 / n0) / a0
    print(f"{name}: spec rule n {nlit} -> alpha {a:.4f}/cm theta_r {tr:+.3f} (air-dry {ad});"
          f" theta_r=0 -> n {n0:.3f} alpha {a0:.4f}/cm, psi(air-dry) {-h_ad * 0.0980665 / 1000:.0f} MPa")
