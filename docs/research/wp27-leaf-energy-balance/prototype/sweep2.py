import sys; sys.path.insert(0, '.')
from proto import *
d = 0.72*0.08
def teb(t, par, ta, ea, q, u, stress, cap, patm=101.3):
    _, gs = gas(par, t, es(t)-ea, stress, cap)
    gbh = gbh_side(u, d, t-ta, ta); gv = gv_of(gs, gbh)
    tn = closed_form(ta, ea, q, gbh, gv, patm)
    return newton(tn, ta, ea, q, gbh, gv, patm, 1)
def secant_fixed(par, ta, ea, q, u, stress, cap, n):
    """n energy-balance evaluations. g(T)=Teb(T)-T. Start T0=Ta, T1=Teb(Ta). Safeguarded secant
    inside a bracket once one is found (Illinois-free simple version)."""
    t0 = ta; g0 = teb(t0, par, ta, ea, q, u, stress, cap) - t0
    t1 = t0 + g0
    lo, hi = (None, None)
    for _ in range(n-1):
        g1 = teb(t1, par, ta, ea, q, u, stress, cap) - t1
        # maintain bracket
        for (t,g) in ((t0,g0),(t1,g1)):
            if g > 0: lo = t if lo is None or t > lo else lo
            else: hi = t if hi is None or t < hi else hi
        if g1 == g0: break
        tn = t1 - g1*(t1-t0)/(g1-g0)
        if lo is not None and hi is not None and lo < hi and not (lo < tn < hi):
            tn = 0.5*(lo+hi)
        tn = max(ta-20, min(ta+25, tn))
        t0, g0, t1 = t1, g1, tn
    return t1
def roots(par, ta, ea, q, u, stress, cap):
    ts = [ta-20 + 0.25*i for i in range(181)]
    gs_ = [teb(t, par, ta, ea, q, u, stress, cap)-t for t in ts]
    return sum(1 for i in range(len(ts)-1) if gs_[i]*gs_[i+1] < 0)
if __name__ == "__main__":
  pass
worst = {}
multi = 0; n = 0
for ta in (0.0, 15.0, 30.0, 38.0, 42.0, 46.0):
  for rh in (0.1, 0.4, 0.7, 0.95):
    ea = rh*es(ta)
    for cloud in (0.0, 1.0):
      esky = sky_emissivity(ta, ea, cloud)
      for u in (0.05, 0.3, 1.0, 5.0):
        for par in (0.0, 100.0, 800.0, 2000.0):
          for stress in (1.0, 0.4, 0.0):
            for cap in (1.0,):
              q = q_abs_of(par, esky, ta)
              te, ae, _ = coupled_exact(par, ta, ea, q, u, d, stress, cap)
              n += 1
              r = roots(par, ta, ea, q, u, stress, cap)
              if r > 1: multi += 1; print('multiple roots', r, ta, rh, cloud, u, par, stress)
              for k in worst:
                t = secant_fixed(par, ta, ea, q, u, stress, cap, k)
                err = abs(t-te)
                if err > worst[k][0]: worst[k] = (err, (ta, rh, cloud, u, par, stress, round(te-ta,2)))
print(n, 'multi', multi)
for k,v in worst.items(): print(k, v)
