"""Band-averaged leaf optics from PROSPECT-D weighted by ASTM G173-03 (via pvlib)."""
import numpy as np, prosail
from pvlib.spectrum import get_reference_spectra

ref = get_reference_spectra(standard="ASTM G173-03")
wl_all = ref.index.values.astype(float)
def interp(col): return np.interp(np.arange(400, 2501), wl_all, ref[col].values)
WL = np.arange(400, 2501)
G = interp("global"); D = interp("direct")

def frac():
    w = wl_all; g = ref["global"].values
    tot = np.trapezoid(g, w)
    def part(a, b):
        m = (w >= a) & (w <= b); return np.trapezoid(g[m], w[m]) / tot
    print(f"G173 global total {tot:.1f} W/m2; <400 {part(280,400):.3f}; 400-700 {part(400,700):.3f}; 700-2500 {part(700,2500):.3f}; >2500 {part(2500,4000):.3f}")
    d = ref["direct"].values; td = np.trapezoid(d, w)
    m = (w >= 400) & (w <= 700); print(f"direct: PAR fraction {np.trapezoid(d[m], w[m])/td:.3f}")

def bands(r, t, wgt=G, label="global"):
    out = {}
    for name, a, b in [("PAR", 400, 700), ("NIR", 700, 2500), ("SW400-2500", 400, 2500)]:
        m = (WL >= a) & (WL <= b)
        wE = wgt[m]
        wP = wgt[m] * WL[m]  # photon weighting
        R = np.sum(r[m]*wE)/np.sum(wE); T = np.sum(t[m]*wE)/np.sum(wE)
        out[name] = (R, T, 1-R-T)
        if name == "PAR":
            Rp = np.sum(r[m]*wP)/np.sum(wP); Tp = np.sum(t[m]*wP)/np.sum(wP)
            out["PAR_photon"] = (Rp, Tp, 1-Rp-Tp)
            Rf = r[m].mean(); Tf = t[m].mean(); out["PAR_flat"] = (Rf, Tf, 1-Rf-Tf)
    return out

def run(n, cab, car, cw, cm, cbrown=0.0, ant=0.0):
    w, r, t = prosail.run_prospect(n, cab, car, cbrown, cw, cm, ant=ant, prospect_version="D")
    assert w[0] == 400 and w[-1] == 2500
    return r, t

frac()
cases = {
 # name: N, Cab ug/cm2, Car, Cw cm, Cm g/cm2
 "bean typical (N1.6,Cab45,Cw.016,Cm.0035)": (1.6, 45, 10, 0.016, 0.0035),
 "bean thin/shade (N1.4,Cab35,Cw.012,Cm.0025)": (1.4, 35, 8, 0.012, 0.0025),
 "bean dark (N1.8,Cab60,Cw.018,Cm.0045)": (1.8, 60, 12, 0.018, 0.0045),
 "wheat flag (N1.5,Cab50,Cw.016,Cm.0050)": (1.5, 50, 11, 0.016, 0.0050),
 "wheat lower N (N1.5,Cab30,Cw.014,Cm.0045)": (1.5, 30, 8, 0.014, 0.0045),
 "senescent yellow (N1.6,Cab8,Car6,Cbrown0.3)": (1.6, 8, 6, 0.012, 0.0035),
}
print("\ncase | band | R | T | A   (global-weighted unless noted)")
for k, (n, cab, car, cw, cm) in cases.items():
    cb = 0.3 if "senescent" in k else 0.0
    r, t = run(n, cab, car, cw, cm, cbrown=cb)
    o = bands(r, t); od = bands(r, t, D)
    for b in ["PAR", "PAR_photon", "PAR_flat", "NIR", "SW400-2500"]:
        R, T, A = o[b]; print(f"{k} | {b} | {R:.3f} | {T:.3f} | {A:.3f}")
    R, T, A = od["SW400-2500"]; print(f"{k} | SW400-2500 direct-weighted | {R:.3f} | {T:.3f} | {A:.3f}")

print("\nChl sweep (bean-like N1.6, Car=Cab/4.5 min 2, Cw .016, Cm .0035); PAR photon-weighted absorptance")
MW = 893.5*0.75 + 907.5*0.25  # approx chl a+b mean molar mass g/mol (a:b=3)
xs = []; ys = []
for cab in [2, 5, 10, 15, 20, 25, 30, 40, 50, 60, 70, 80]:
    r, t = run(1.6, cab, max(2, cab/4.5), 0.016, 0.0035)
    A = bands(r, t)["PAR_photon"][2]; Ae = bands(r, t)["PAR"][2]
    umol = cab*1e-6*1e4/MW*1e6  # ug/cm2 -> g/m2 -> mol/m2 -> umol/m2
    xs.append(umol); ys.append(A)
    print(f"Cab {cab:3d} ug/cm2 = {umol:6.0f} umol/m2 : A_PAR(photon) {A:.3f}  A_PAR(energy) {Ae:.3f}  Evans-form chl/(chl+76) {umol/(umol+76):.3f}")
xs = np.array(xs); ys = np.array(ys)
# fit A = amax*chl/(chl+K)
best = None
for amax in np.linspace(0.85, 1.0, 151):
    for K in np.linspace(10, 200, 381):
        e = np.sum((amax*xs/(xs+K) - ys)**2)
        if best is None or e < best[0]: best = (e, amax, K)
print(f"fit A = amax*chl/(chl+K): amax={best[1]:.3f} K={best[2]:.1f} umol/m2, rmse={np.sqrt(best[0]/len(xs)):.4f}")
best = None
for K in np.linspace(10, 200, 1901):
    e = np.sum((xs/(xs+K) - ys)**2)
    if best is None or e < best[0]: best = (e, K)
print(f"fit A = chl/(chl+K): K={best[1]:.1f} umol/m2, rmse={np.sqrt(best[0]/len(xs)):.4f}")
print("\nN sensitivity at Cab 45: ")
for n in [1.2,1.4,1.6,1.8,2.0,2.5]:
    r,t=run(n,45,10,0.016,0.0035); o=bands(r,t)
    print(f"N {n}: A_PAR {o['PAR'][2]:.3f} R_NIR {o['NIR'][0]:.3f} T_NIR {o['NIR'][1]:.3f} A_NIR {o['NIR'][2]:.3f} A_SW {o['SW400-2500'][2]:.3f}")
print("\nCw sensitivity (N1.6,Cab45):")
for cw in [0.008,0.012,0.016,0.020,0.025]:
    r,t=run(1.6,45,10,cw,0.0035); o=bands(r,t)
    print(f"Cw {cw}: A_NIR {o['NIR'][2]:.3f} A_SW {o['SW400-2500'][2]:.3f}")
