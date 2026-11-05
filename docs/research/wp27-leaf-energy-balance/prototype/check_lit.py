import sys; sys.path.insert(0, sys.argv[1])
import proto as P
d = 0.72*0.08
for ta, rh in ((33,40),(35,30),(35,20)):
    ea = rh/100*P.es(ta); D = P.es(ta)-ea; esky = P.sky_emissivity(ta, ea, 0.0)
    for g1 in (4.0, 5.79, 8.0):
        P.BEAN['g1'] = g1
        row=[]
        for u in (0.3, 1.0, 2.5):
            q = P.q_abs_of(1500.0, esky, ta)
            t,a,gs = P.coupled_exact(1500.0, ta, ea, q, u, d, 1.0)
            row.append(f"u{u}: dT {t-ta:+.1f} gs {gs:.2f} A {a:.1f}")
        print(f"Ta {ta} D {D:.1f} g1 {g1}: " + " | ".join(row))
# gs needed: fixed gs sweep at Ta 35 D~3.9 u 2.5
ta=35; ea=0.3*P.es(ta); esky=P.sky_emissivity(ta,ea,0); q=P.q_abs_of(1500.0,esky,ta)
for gs in (0.2,0.4,0.6,0.8,1.0):
    gbh=P.gbh_side(2.5,d,0,ta); gv=P.gv_of(gs,gbh)
    t=P.newton(P.closed_form(ta,ea,q,gbh,gv,101.3),ta,ea,q,gbh,gv,101.3,3)
    print(f"fixed gs {gs}: dT {t-ta:+.1f}")
