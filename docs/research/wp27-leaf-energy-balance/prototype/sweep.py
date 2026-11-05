import sys; sys.path.insert(0, '.')
from proto import *
d = 0.72*0.08
worst = {2:(0,None),3:(0,None),4:(0,None),5:(0,None)}
worstA = {3:0,4:0}
n=0
for ta in (0.0, 15.0, 30.0, 42.0):
  for rh in (0.1, 0.4, 0.7, 0.95):
    ea = rh*es(ta)
    for cloud in (0.0, 1.0):
      esky = sky_emissivity(ta, ea, cloud)
      for u in (0.05, 0.3, 1.0, 5.0):
        for par in (0.0, 100.0, 800.0, 2000.0):
          for stress in (1.0, 0.4, 0.0):
            for cap in (1.0, 0.1):
              q = q_abs_of(par, esky, ta)
              te, ae, _ = coupled_exact(par, ta, ea, q, u, d, stress, cap)
              n+=1
              for k in worst:
                t, a, g, h = fixed_passes(par, ta, ea, q, u, d, stress, cap, n=k)
                err = abs(t-te)
                if err > worst[k][0]: worst[k] = (err, (ta, rh, cloud, u, par, stress, cap, round(te-ta,2), [round(x-ta,3) for x in h]))
                if k in worstA:
                  worstA[k] = max(worstA[k], abs(a-ae)/max(abs(ae),1.0))
print(n)
for k,v in worst.items(): print(k, v)
print(worstA)
