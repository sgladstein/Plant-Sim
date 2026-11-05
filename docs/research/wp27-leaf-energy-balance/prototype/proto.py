"""Prototype: FvCB+Medlyn (as ps-physio gas.rs) coupled to a leaf energy balance.
Units: per one-sided leaf area. T in C unless *_k. Pressures kPa. Conductances mol m-2 s-1.
"""
import math

R = 8.314; T25 = 298.15; ZK = 273.15
SIGMA = 5.670374e-8; CP = 29.3  # J mol-1 K-1 (moist air ~ 29.2-29.3)
EPS_LEAF = 0.97

def arr(t, ea):
    tk = t + ZK
    return math.exp(ea * (tk - T25) / (T25 * R * tk))

def peaked(t, ea, ed, ds):
    tk = t + ZK
    a = math.exp((tk - T25) * ea / (R * tk * T25))
    return a * (1 + math.exp((ds * T25 - ed) / (R * T25))) / (1 + math.exp((ds * tk - ed) / (R * tk)))

BEAN = dict(vcmax25=89.3, jmax25=159.8, rd25=0.88, q10=1.92, g1=4.0, g0=0.01, alpha=0.3, theta=0.7,
            vt=(58550, 200000, 629.26), jt=(29680, 200000, 631.88))

def es(t):  # kPa, Buck 1981 (as Campbell & Norman 1998 eq 3.8)
    return 0.611 * math.exp(17.502 * t / (t + 240.97))

def slope(t):
    return 17.502 * 240.97 * es(t) / (t + 240.97) ** 2

def lam(t):  # J mol-1
    return (2.501e6 - 2.361e3 * t) * 0.018015

def gas(par, t, vpd_leaf, stress=1.0, cap=1.0, ca=420.0, patm=101.3, p=BEAN):
    gs_ = 42.75 * arr(t, 37830) * patm / 100
    ko = 278.4 * arr(t, 36380)
    km = 404.9 * arr(t, 79430) * (1 + 210 * patm / 100 / ko)
    vc = cap * p['vcmax25'] * peaked(t, *p['vt'])
    jm = cap * p['jmax25'] * peaked(t, *p['jt'])
    rd = cap * p['rd25'] * p['q10'] ** ((t - 25) / 10)
    s = p['alpha'] * par + jm
    j = (s - math.sqrt(max(s * s - 4 * p['alpha'] * p['theta'] * par * jm, 0))) / (2 * p['theta'])
    vpd = max(vpd_leaf, 0.5)
    g0c = p['g0'] / 1.57
    m = stress * (1 + p['g1'] / math.sqrt(vpd)) / ca
    def a_of(v, k, ci):
        return v * (ci - gs_) / (ci + k) - rd
    res = []
    for v, k in ((vc, km), (j / 4, 2 * gs_)):
        if a_of(v, k, ca) <= 0:
            res.append((a_of(v, k, ca), ca)); continue
        lo = max(ca - 1 / m, 0) if m > 0 else 0.0
        hi = ca
        f = lambda ci: a_of(v, k, ci) * (1 - m * (ca - ci)) - g0c * (ca - ci)
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            if f(mid) < 0: lo = mid
            else: hi = mid
        ci = 0.5 * (lo + hi)
        res.append((a_of(v, k, ci), ci))
    a, ci = min(res)
    gsc = max(g0c + m * max(a, 0), g0c)
    return a, gsc * 1.57

def gbh_side(u, d, dT=0.0, tair=25.0, patm=101.3):
    """Heat boundary-layer conductance per side, mol m-2 s-1.
    Forced: Campbell & Norman 1998 eq 7.30, 0.135 sqrt(u/d), d = 0.72 x leaf width.
    Free: Leuning 1995 App. E (plantecophys)."""
    cmolar = patm * 1000 / (R * (tair + ZK))
    forced = 0.135 * math.sqrt(max(u, 0.05) / d)
    gr = 1.6e8 * abs(dT) * d ** 3
    free = 0.5 * 21.5e-6 * gr ** 0.25 / d * cmolar
    return forced + free

def sky_emissivity(ta, ea_kpa, cloud):
    eclear = 1.72 * (ea_kpa / (ta + ZK)) ** (1 / 7)  # Brutsaert 1975 (kPa, K)
    return (1 - 0.84 * cloud) * eclear + 0.84 * cloud  # Unsworth & Monteith 1975 / C&N eq 10.12

def residual(tl, ta, ea, q_abs, gbh, gv, patm):
    """Full nonlinear energy balance residual W m-2 (per one-sided area, both sides emitting)."""
    return (q_abs - 2 * EPS_LEAF * SIGMA * (tl + ZK) ** 4 - 2 * CP * gbh * (tl - ta)
            - lam(ta) * gv * (es(tl) - ea) / patm)

def closed_form(ta, ea, q_abs, gbh, gv, patm):
    """Linearised (Penman-Monteith / C&N 14.6 style) leaf temperature."""
    tk = ta + ZK
    rni = q_abs - 2 * EPS_LEAF * SIGMA * tk ** 4
    gr = 4 * EPS_LEAF * SIGMA * tk ** 3 / CP  # per side
    D = es(ta) - ea
    lv = lam(ta)
    num = rni - lv * gv * D / patm
    den = 2 * CP * (gbh + gr) + lv * gv * slope(ta) / patm
    return ta + num / den

def newton(tl, ta, ea, q_abs, gbh, gv, patm, n=2):
    lv = lam(ta)
    for _ in range(n):
        f = residual(tl, ta, ea, q_abs, gbh, gv, patm)
        df = -8 * EPS_LEAF * SIGMA * (tl + ZK) ** 3 - 2 * CP * gbh - lv * gv * slope(tl) / patm
        tl -= f / df
    return tl

def gv_of(gs, gbh, amphi=True):
    gbw = 1.075 * gbh * (2 if amphi else 1)  # Leuning 1995: gbw = 1.075 gbh
    return gs * gbw / (gs + gbw) if gs > 0 else 0.0

def coupled_exact(par, ta, ea, q_abs, u, d, stress=1.0, cap=1.0, patm=101.3):
    """Reference: bisection on T - T_eb(gs(T)) with exact nonlinear EB each time."""
    def eb_given_t(t):
        _, gs = gas(par, t, es(t) - ea, stress, cap)
        gbh = gbh_side(u, d, t - ta, ta)
        gv = gv_of(gs, gbh)
        lo, hi = ta - 30, ta + 30
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            if residual(mid, ta, ea, q_abs, gbh, gv, patm) > 0: lo = mid
            else: hi = mid
        return 0.5 * (lo + hi)
    lo, hi = ta - 25, ta + 25
    for _ in range(80):
        mid = 0.5 * (lo + hi)
        if eb_given_t(mid) - mid > 0: lo = mid
        else: hi = mid
    t = 0.5 * (lo + hi)
    a, gs = gas(par, t, es(t) - ea, stress, cap)
    return t, a, gs

def fixed_passes(par, ta, ea, q_abs, u, d, stress=1.0, cap=1.0, patm=101.3, n=3, newton_steps=1,
                 relax=1.0):
    """GECROS-like: start at Ta; each pass: gas exchange at T (leaf VPD), closed form + Newton for T."""
    t = ta
    hist = []
    for _ in range(n):
        a, gs = gas(par, t, es(t) - ea, stress, cap)
        gbh = gbh_side(u, d, t - ta, ta)
        gv = gv_of(gs, gbh)
        tn = closed_form(ta, ea, q_abs, gbh, gv, patm)
        tn = newton(tn, ta, ea, q_abs, gbh, gv, patm, newton_steps)
        t = t + relax * (tn - t)
        hist.append(t)
    a, gs = gas(par, t, es(t) - ea, stress, cap)
    return t, a, gs, hist

def q_abs_of(par_abs, eps_sky, ta, sky_view=1.0, nir_ratio=0.25):
    """Absorbed SW + LW per one-sided area.
    SW: absorbed PAR (umol) / 4.57 umol J-1 * (1 + nir_ratio) where nir_ratio = absorbed NIR / absorbed PAR energy.
    LW: upper side sees sky over sky_view and leaves/ground at Ta elsewhere; lower side sees Ta."""
    sw = par_abs / 4.57 * (1 + nir_ratio)
    lt = SIGMA * (ta + ZK) ** 4
    lw = EPS_LEAF * (sky_view * eps_sky * lt + (1 - sky_view) * lt + lt)
    return sw + lw

if __name__ == '__main__':
    d = 0.72 * 0.08  # bean leaflet ~8 cm wide
    print("== hot still day: Ta 35 C, RH 30 %, clear")
    ta = 35.0; ea = 0.30 * es(ta); esky = sky_emissivity(ta, ea, 0.0)
    print(f"eps_sky={esky:.3f} es={es(ta):.3f} D={es(ta)-ea:.3f} kPa")
    for u in (0.2, 0.5, 1.0, 3.0):
        for stress in (1.0, 0.3, 0.0):
            par = 1500.0
            q = q_abs_of(par, esky, ta)
            te, ae, ge = coupled_exact(par, ta, ea, q, u, d, stress)
            t1, a1, g1, h1 = fixed_passes(par, ta, ea, q, u, d, stress, n=1)
            t3, a3, g3, h3 = fixed_passes(par, ta, ea, q, u, d, stress, n=3)
            t4, a4, g4, h4 = fixed_passes(par, ta, ea, q, u, d, stress, n=4)
            print(f"u={u:4} s={stress:3}: Tl-Ta exact {te-ta:6.2f} A {ae:5.2f} gs {ge:.3f} | 1 pass {t1-ta:6.2f} | 3 passes {t3-ta:6.2f} (A {a3:5.2f}) | 4 {t4-ta:6.2f} hist {[round(x-ta,2) for x in h4]}")
    print("== linearisation error alone (no gas coupling), Ta 35, u 0.2, gs fixed")
    for gs in (0.0, 0.05, 0.2, 0.5):
        q = q_abs_of(1500, esky, ta)
        gbh = gbh_side(0.2, d, 0, ta); gv = gv_of(gs, gbh)
        cf = closed_form(ta, ea, q, gbh, gv, 101.3)
        nt1 = newton(cf, ta, ea, q, gbh, gv, 101.3, 1)
        lo, hi = ta - 30, ta + 40
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            if residual(mid, ta, ea, q, gbh, gv, 101.3) > 0: lo = mid
            else: hi = mid
        print(f"gs={gs}: exact {lo-ta:6.3f} closed {cf-ta:6.3f} +1 newton {nt1-ta:6.4f}")
    print("== clear night: Ta 2 C, RH 90 %, PAR 0, u 0.5 / 0.1")
    ta = 2.0; ea = 0.9 * es(ta); esky = sky_emissivity(ta, ea, 0.0)
    for u in (0.5, 0.1):
        for sv in (1.0, 0.5, 0.0):
            q = q_abs_of(0.0, esky, ta, sky_view=sv)
            te, _, _ = coupled_exact(0.0, ta, ea, q, u, d)
            print(f"eps_sky={esky:.3f} u={u} sky_view={sv}: Tl-Ta {te-ta:6.2f}")
    print("== sensitivity of per-node assumptions at Ta 30 RH 50 u 1, PAR 1500 / 300")
    ta = 30.0; ea = 0.5 * es(ta); esky = sky_emissivity(ta, ea, 0.0)
    for par in (1500.0, 300.0, 50.0):
        for sv in (1.0, 0.5, 0.0):
            for nr in (0.15, 0.25, 0.5):
                q = q_abs_of(par, esky, ta, sky_view=sv, nir_ratio=nr)
                te, ae, ge = coupled_exact(par, ta, ea, q, 1.0, d)
                print(f"PAR {par:6} sky {sv} nir {nr}: dT {te-ta:6.2f} A {ae:6.2f} E(mmol) {1000*gv_of(ge, gbh_side(1.0, d, te-ta, ta))*(es(te)-ea)/101.3:5.2f}")
    print("== A and E vs air-temperature leaf (status quo), Ta 35 RH 30 u 0.5 PAR 1500")
    ta = 35.0; ea = 0.30 * es(ta); esky = sky_emissivity(ta, ea, 0.0)
    a0, g0 = gas(1500, ta, es(ta) - ea)
    e0 = g0 * (es(ta) - ea) / 101.3
    q = q_abs_of(1500, esky, ta)
    te, ae, ge = coupled_exact(1500, ta, ea, q, 0.5, d)
    ee = gv_of(ge, gbh_side(0.5, d, te - ta, ta)) * (es(te) - ea) / 101.3
    print(f"status quo A {a0:.2f} gs {g0:.3f} E {1000*e0:.2f} mmol | EB: Tl {te:.2f} A {ae:.2f} gs {ge:.3f} E {1000*ee:.2f} mmol")
