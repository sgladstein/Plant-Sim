"""Lower-baseline check: well-watered sunlit bean leaf, Ta 30 C, wind 2.5 m/s, VPD 1-4 kPa.
Compare the fitted dT = a + b*VPD with the field baselines (b = -1.9 to -2.7 C/kPa)."""
import sys; sys.path.insert(0, sys.argv[1] if len(sys.argv) > 1 else '.')
import proto as P
d = 0.72 * 0.08
ta = 30.0
for g1 in (4.0, 5.79, 8.0):
    P.BEAN['g1'] = g1
    xs, ys = [], []
    for vpd in (1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0):
        ea = P.es(ta) - vpd
        q = P.q_abs_of(1500.0, P.sky_emissivity(ta, ea, 0.0), ta)
        t, a, gs = P.coupled_exact(1500.0, ta, ea, q, 2.5, d, 1.0)
        xs.append(vpd); ys.append(t - ta)
    n = len(xs); mx = sum(xs) / n; my = sum(ys) / n
    b = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sum((x - mx) ** 2 for x in xs)
    print(f"g1 {g1}: dT = {my - b * mx:+.2f} {b:+.2f}*VPD   points " + " ".join(f"{y:+.1f}" for y in ys))
