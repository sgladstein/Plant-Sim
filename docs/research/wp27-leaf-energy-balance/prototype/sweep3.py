import sys; sys.path.insert(0, '.')
from proto import *
from sweep2 import teb
d = 0.72*0.08
LO, HI = -20.0, 25.0
def illinois(par, ta, ea, q, u, stress, cap, n):
    f = lambda t: teb(t, par, ta, ea, q, u, stress, cap) - t
    a = ta; fa = f(a); evals = 1
    if fa == 0: return a
    b = a + fa; fb = f(b); evals += 1
    side = 0
    # extend until bracketed (secant steps clamped to the global bracket)
    while fa*fb > 0 and evals < n:
        s = b - fb*(b-a)/(fb-fa) if fb != fa else b + (fb)
        s = min(max(s, ta+LO), ta+HI)
        a, fa = b, fb
        b = s; fb = f(b); evals += 1
    if fa*fb > 0: return b
    while evals < n:
        c = (a*fb - b*fa)/(fb-fa)
        fc = f(c); evals += 1
        if fc*fb < 0:
            a, fa = b, fb
            b, fb = c, fc
            side = 0
        else:
            b, fb = c, fc
            fa *= 0.5   # Illinois
        if abs(b-a) < 1e-3: break
    return b if abs(fb) <= abs(fa) else a
if __name__ == '__main__':
  worst = {k:(0,None) for k in (3,4,5,6,8)}
  stats = {k:[] for k in worst}
  for ta in (0.0, 15.0, 30.0, 38.0, 42.0, 46.0):
    for rh in (0.1, 0.4, 0.7, 0.95):
      ea = rh*es(ta)
      for cloud in (0.0, 1.0):
        esky = sky_emissivity(ta, ea, cloud)
        for u in (0.05, 0.3, 1.0, 5.0):
          for par in (0.0, 100.0, 800.0, 2000.0):
            for stress in (1.0, 0.4, 0.0):
                q = q_abs_of(par, esky, ta)
                te, ae, _ = coupled_exact(par, ta, ea, q, u, d, stress, 1.0)
                for k in worst:
                  t = illinois(par, ta, ea, q, u, stress, 1.0, k)
                  err = abs(t-te); stats[k].append(err)
                  if err > worst[k][0]: worst[k] = (err, (ta, rh, cloud, u, par, stress, round(te-ta,2)))
  for k,v in worst.items():
    s = sorted(stats[k]); print(k, 'max', round(v[0],4), v[1], 'p99', round(s[int(0.99*len(s))],4), 'p95', round(s[int(0.95*len(s))],4))
