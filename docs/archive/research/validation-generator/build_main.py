import sys, os, math
sys.path.insert(0, os.path.dirname(__file__))
from ronw import s, opt, num, curve
import data_phen, data_traits, data_resp
OUT = sys.argv[1]
SRC = {}
for m in (data_phen, data_traits, data_resp): SRC.update(m.SOURCES)

def w(path, text):
    p = os.path.join(OUT, path); os.makedirs(os.path.dirname(p), exist_ok=True); open(p, 'w').write(text)

# ---------- sources.ron
lines = ["// Every source cited by data/validation/ (schema: docs/validation-data.md).",
         "// Values are cited as facts with attribution; no source file is copied.",
         "Sources(", "  schema: 1,", "  sources: {"]
for k in sorted(SRC):
    d = SRC[k]
    lines.append(f'''    {s(k)}: Source(
      citation: {s(d["citation"])},
      doi: {opt(d.get("doi"))},
      url: {s(d["url"])},
      licence: {s(d["licence"])},
      cultivar: {s(d["cultivar"])},
      conditions: {s(d["conditions"])},
    ),''')
lines += ["  },", ")", ""]
w("sources.ron", "\n".join(lines))

def expect(e):
    k = e[0]
    if k == "Range": return f"Range(min: {num(e[1])}, max: {num(e[2])})"
    if k == "Approx": return f"Approx(value: {num(e[1])}, rel_tol: {num(e[2])})"
    if k == "RelChange": return f"RelChange(min: {num(e[1])}, max: {num(e[2])})"
    if k == "Cited": return f"Cited(rel_change: {num(e[1])})"
    if k == "FallsMoreThan": return f"FallsMoreThan({s(e[1])})"
    return k
def srcs(l):
    for x in l: assert x in SRC, x
    return "[" + ", ".join(s(x) for x in l) + "]"

# ---------- targets.ron
# (id, variable, at, expect, unit, basis, sources, gate, note)
B10 = "mean-method degree-days from sowing, base 10 C"
T = [
 # --- M0 gates (design plan section 7 exit list)
 ("flowering_day", "days_since_sowing", "Stage(Flowering)", ("Range", 28, 57), "d", None,
  ["ventura1991", "dssat_ccpa9001", "dssat_ufga8501", "cavalcante2020", "pinzon2024"], "M0",
  "First open flower. Warm tropical field 28.7 d (Ventura), Palmira 29 d and Gainesville 32 d (DSSAT anthesis dates), Jaboticabal type I about 42 d (Cavalcante, start of R6, days read from figure labels), cool winter greenhouse 54-57 d (Pinzón, start of R6). A temperate summer should sit in the middle."),
 ("maturity_day", "days_since_sowing", "Stage(Mature)", ("Range", 54, 93), "d", None,
  ["ventura1991", "dssat_ccpa9001", "dssat_ufga8501", "cavalcante2020", "pinzon2024", "dssat_ccpo9002", "baldinotti2026"], "M0",
  "Physiological maturity. 54.2 d (Ventura, warm), 60 d (Redkloud, Palmira), 65 d (Seafarer, Gainesville), 73-75 d (Pinzón; Cavalcante type I), 77 d (Redkloud, cool Popayan), 65-93 d across super-early to long-cycle cultivars (Baldinotti, cited by the plan as Conceição 2026)."),
 ("maturity_thermal_time_b10", "thermal_time_cd_b10", "Stage(Mature)", ("Range", 700.0, 950.0), "Cd", B10,
  ["cavalcante2020"], "M0",
  "Type I cultivars reach R9 at 826.8 Cd above 10 C in 75 d (Fig. 3 data labels). Range is that value -15%/+15%. This replaces the plan's '~800 Cd' from Pinzón 2024, whose base (8.3 C) is now known: see the Info target below."),
 ("total_dry_mass_at_maturity", "total_dry_mass_g", "Stage(Mature)", ("Range", 12.0, 35.0), "g", None,
  ["kumar2025", "pinzon2024", "mohammed2018"], "M0",
  "Whole plant including roots and shed organs. Kumar 2025 (plan quote, unverified) 21.8 g at harvest; Pinzón 2024 one pot plant in a winter greenhouse about 13.6 g (root 2.4 + shoot 3.1 + pods 8.3 at R8, period 1); Mohammed 2018 17.6 g at 43 days after emergence (field, per-plant basis unclear). A lone pot plant in temperate full sun is expected near the upper half."),
 ("pods_at_maturity", "pods", "Final", ("Range", 6.0, 20.0), "pods", None,
  ["kumar2025", "kazai2019", "vargas2021"], "M0",
  "Pods per plant. Kumar 2025 11.8 (plan quote, unverified); Kazai 2019 well-watered field 8.7 (Table 1); Vargas 2021 control 8.2 (Table 1). Field plants at stand density carry fewer (DSSAT Redkloud 4.9, Seafarer 4.5 per plant) than a lone pot plant."),
 ("max_height", "height_m", "Max", ("Range", 0.25, 0.50), "m", None,
  ["kumar2025", "dssat_ufga0701", "lopes1987", "kazai2019"], "M0",
  "Kumar 2025 35 cm (unverified); snap bean Bronco canopy 0.33-0.39 m from 50 d on (DSSAT UFGA0701); Negrito 897 39.2 cm in full sun (Lopes 1987); Kazai 2019 51.5 cm (well watered, Table 1)."),
 ("peak_leaf_area", "leaf_area_m2", "Max", ("Range", 0.07, 0.35), "m2", None,
  ["kumar2025", "dssat_ccpa9001", "dssat_ufga8501", "clavijo2019", "maldonado1993"], "M0",
  "Per plant. Kumar 2025 0.12 m2 (unverified); field plants at 25-52 plants/m2 peak at 0.07-0.11 m2 (DSSAT Seafarer, Redkloud); widely spaced plants (5 plants/m2) exceed 0.3 m2 (Clavijo 2019, text); ICA-Cerinza reaches 0.29 m2 at 78 d after emergence in a cool climate (Maldonado 1993, Table 1). A lone pot plant sits between the dense and the open values."),
 # --- Info: phenology on each paper's own base
 ("emergence_day", "days_since_sowing", "Stage(Emerged)", ("Range", 4.0, 17.0), "d", None,
  ["ventura1991", "cavalcante2020", "pinzon2024"], "Info",
  "4.5 d warm field (Ventura); about 10 d (Cavalcante, figure labels); 11 and 17 d in the greenhouse (Pinzón, V1)."),
 ("emergence_thermal_time_b10", "thermal_time_cd_b10", "Stage(Emerged)", ("Approx", 92.5, 0.25), "Cd", B10, ["cavalcante2020"], "Info", "Type I, V1 (emergence), Fig. 3."),
 ("flowering_thermal_time_b10", "thermal_time_cd_b10", "Stage(Flowering)", ("Approx", 430.3, 0.15), "Cd", B10, ["cavalcante2020"], "Info", "Type I, reaching R6 (flowering): cumulative of the Fig. 3 stage sums."),
 ("pod_set_thermal_time_b10", "thermal_time_cd_b10", "Stage(PodSet)", ("Approx", 486.7, 0.15), "Cd", B10, ["cavalcante2020"], "Info", "Type I, reaching R7 (pod formation)."),
 ("seed_fill_thermal_time_b10", "thermal_time_cd_b10", "Stage(SeedFill)", ("Approx", 569.4, 0.15), "Cd", B10, ["cavalcante2020"], "Info", "Type I, reaching R8 (pod filling)."),
 ("emergence_thermal_time_b8_3", "thermal_time_cd_b8.3", "Stage(Emerged)", ("Range", 131.0, 166.0), "Cd", "mean-method degree-days from sowing, base 8.3 C", ["pinzon2024"], "Info", "V1 (emergence) at 145.8 and 150.7 Cd in the two periods; range is the two -10%/+10%."),
 ("flowering_thermal_time_b8_3", "thermal_time_cd_b8.3", "Stage(Flowering)", ("Range", 544.0, 680.0), "Cd", "mean-method degree-days from sowing, base 8.3 C", ["pinzon2024"], "Info", "Reaching R6 (flowering) at 604.4 and 618.2 Cd (cumulative of the stage table); range -10%/+10%. Note this is well above the species file's flowering sum."),
 ("pod_set_thermal_time_b8_3", "thermal_time_cd_b8.3", "Stage(PodSet)", ("Range", 608.0, 757.0), "Cd", "mean-method degree-days from sowing, base 8.3 C", ["pinzon2024"], "Info", "Reaching R7 at 675.1 and 688.6 Cd."),
 ("maturity_thermal_time_b8_3", "thermal_time_cd_b8.3", "Stage(Mature)", ("Range", 723.0, 889.0), "Cd", "mean-method degree-days from sowing, base 8.3 C", ["pinzon2024"], "Info", "Whole cycle 803.5 and 808.6 Cd (the plan's '~800 Cd'); range -10%/+10%."),
 ("emergence_thermal_time_b7", "thermal_time_cd_b7", "Stage(Emerged)", ("Range", 57.6, 110.4), "Cd", "mean-method degree-days from sowing, base 7 C", ["dapaah1999"], "Info", "Sowing to emergence across six sowings, Table 2."),
 ("flowering_thermal_time_b7", "thermal_time_cd_b7", "Stage(Flowering)", ("Range", 338.0, 440.0), "Cd", "mean-method degree-days from sowing, base 7 C", ["dapaah1999"], "Info", "S-E plus E-F per sowing from Table 2 (lowest 347, highest 424); range widened slightly. Mean 76 + 306 = 382."),
 ("pod_set_thermal_time_b7", "thermal_time_cd_b7", "Stage(PodSet)", ("Range", 410.0, 520.0), "Cd", "mean-method degree-days from sowing, base 7 C", ["dapaah1999"], "Info", "Adds F-podding (58-110 Cd); mean 461."),
 ("maturity_thermal_time_b7", "thermal_time_cd_b7", "Stage(Mature)", ("Range", 760.0, 950.0), "Cd", "mean-method degree-days from sowing, base 7 C", ["dapaah1999"], "Info", "Adds podding to physiological maturity (369-452 Cd); mean 76 + 306 + 79 + 392 = 853. Othello is a pinto, likely type III."),
 ("first_trifoliate_thermal_time_b4_4", "thermal_time_cd_b4.4", "Stage(FirstTrifoliate)", ("Range", 263.0, 372.0), "Cd", "single-sine degree-days from seeding, 4.44 C lower and 32.2 C upper threshold, converted from F-days by x 5/9", ["croptime2020"], "Info",
  "526-608 F-days across Sahara, 5630 and Provider = 292-338 Cd, widened 10%. The harness sums the mean method without the upper cutoff; the two agree within a few percent in a temperate summer."),
 ("flowering_thermal_time_b4_4", "thermal_time_cd_b4.4", "Stage(Flowering)", ("Range", 547.0, 763.0), "Cd", "single-sine degree-days from seeding, 4.44 C lower and 32.2 C upper threshold, converted from F-days by x 5/9", ["croptime2020"], "Info",
  "First open flowers at 1094-1248 F-days = 608-693 Cd, widened 10%. Temperate snap bush beans in Oregon: the closest climate match to the default scenario."),
 # --- Info: architecture and yield components
 ("main_nodes_final", "main_nodes", "Final", ("Range", 5.0, 10.0), "nodes", "DSSAT L#SD counts leaves on the main stem including the unifoliate node", ["dssat_ufga8501", "dssat_ccpa9001"], "Info", "Type I main stems end at 6.0-9.8 leaves: Redkloud 6, Isabella 6.3, Manitou 7.3, Seafarer 9.3."),
 ("seeds_per_pod", "seeds_per_pod", "Final", ("Range", 2.0, 5.5), "-", None, ["kumar2025", "kazai2019", "vargas2021", "cropgro_drybean_spe"], "Info", "Kumar 3.7 (unverified); Kazai 2.2; Vargas 3.26; DSSAT SDPDV 3.0-5.5 across cultivars."),
 ("mean_seed_mass", "mean_seed_mass_g", "Final", ("Range", 0.16, 0.60), "g", None, ["dssat_ufga8501", "dssat_ccpa9001", "dssat_ccpo9002", "vargas2021", "kazai2019"], "Info", "Seafarer 0.16, Kazai 0.27, Vargas 0.34, Redkloud 0.41-0.56 g per seed."),
 ("seed_mass_per_plant", "seed_mass_g", "Final", ("Range", 5.0, 15.0), "g", None, ["kumar2025", "vargas2021", "pinzon2024"], "Info", "Kumar 8.8 g (unverified); Vargas 9.4 g (control, Table 1); Pinzón pods 8.4 g dry at R8 (pod walls included)."),
 ("harvest_index_tops", "harvest_index_tops", "Stage(Mature)", ("Range", 0.40, 0.60), "g/g", "DSSAT HIAD: seed over living above-ground mass", ["dssat_ufga8501", "dssat_ccpa9001", "dssat_ccpo9002"], "Info", "Measured HIAD at maturity: Isabella 0.48, Manitou 0.46, Seafarer 0.55, Redkloud 0.51-0.58."),
 ("sla_early", "sla_cm2_per_g", "Day(30)", ("Range", 200.0, 400.0), "cm2/g", None, ["desalew2021", "mohammed2018", "dssat_ccpa9001", "dssat_ufga8501"], "Info", "199-320 cm2/g at 20-33 d after emergence (Desalew, Table 2); 284-341 at 29-43 d (Mohammed, Table 1); DSSAT field SLAD 280-434 before flowering. The plan's '~240 highest genotype' source (Gebeyehu 2019) could not be found."),
 ("nar_early", "nar_g_m2_day", "Day(40)", ("Range", 2.0, 6.0), "g/m2/d", None, ["mohammed2018"], "Info", "Mohammed 2018: 29.5-43.6 mg/dm2/d = 2.9-4.4 g/m2/d between 29 and 43 d after emergence (Table 3). The plan's 'up to 41' matches this paper, not Merga 2021."),
 ("leaves_at_harvest", "leaves", "Max", ("Range", 12.0, 30.0), "leaves", None, ["kumar2025"], "Info", "Kumar 2025: 20 leaves (unverified)."),
 ("branches_at_harvest", "branches", "Final", ("Range", 3.0, 8.0), "branches", None, ["kumar2025"], "Info", "Kumar 2025: 5.5 branches (unverified)."),
]
lines = ["// Whole-plant range targets for the bush bean (design plan section 8.2), read from the",
         "// single_pot scenario. Schema: docs/validation-data.md.",
         "ValidationTargets(", "  schema: 1,", '  species: "bush_bean",', "  targets: ["]
ids = set()
for (id, var, at, e, unit, basis, so, gate, note) in T:
    assert id not in ids, id; ids.add(id)
    lines.append(f'''    Target(
      id: {s(id)},
      scenario: "single_pot",
      variable: {s(var)},
      at: {at},
      expect: {expect(e)},
      unit: {s(unit)},
      basis: {opt(basis)},
      sources: {srcs(so)},
      gate: {gate},
      note: {s(note)},
    ),''')
lines += ["  ],", ")", ""]
w("targets.ron", "\n".join(lines))

# ---------- responses.ron
D, H, SH, C = "response/drought_from_flowering", "response/heat_at_flowering", "response/shade_50", "response/control"
R = [
 # drought
 ("drought_pods", D, "pods", "Final", ("RelChange", -0.90, -0.30), ["kazai2019", "gomes2000"], "M0", "Emergent",
  "M0 exit: pod count down at least 30%. Kazai 2019 -53% (Table 1; cultivars -8% to -95%), Gomes 2000 about -60% (Carioca, text)."),
 ("drought_pods_cited", D, "pods", "Final", ("Cited", -0.53), ["kazai2019"], "Info", "Emergent", "Mean of four cultivars, 8.7 to 4.1 pods per plant."),
 ("drought_pods_fall_more_than_seed_size", D, "pods", "Final", ("FallsMoreThan", "mean_seed_mass_g"), ["kazai2019", "gomes2000", "rezene2012"], "M0", "Emergent",
  "M0 exit: seed size down less than pod count. Pods -53% vs 100-seed weight -18% (Kazai); pods -60% vs seed weight unchanged (Gomes)."),
 ("drought_seed_size", D, "mean_seed_mass_g", "Final", ("Cited", -0.18), ["kazai2019", "rezene2012"], "Info", "Emergent", "100-seed weight -18% (Kazai, Table 1), -19% (Rezene, abstract); Gomes found no significant change."),
 ("drought_seeds_per_pod", D, "seeds_per_pod", "Final", ("Cited", -0.10), ["kazai2019", "rezene2012"], "Info", "Emergent", "-10% (Kazai), -15% (Rezene)."),
 ("drought_seed_yield", D, "seed_mass_g", "Final", ("Cited", -0.58), ["kazai2019", "rezene2012", "gomes2000"], "Info", "Emergent", "-58% (Kazai), -59% (Rezene), about -60% (Gomes, Carioca)."),
 ("drought_root_shoot", D, "root_shoot_ratio", "Final", ("Increase",), ["acostadiaz2007", "sofi2018"], "M0", "PartlyEncoded", "M0 exit: root to shoot ratio higher."),
 ("drought_root_shoot_cited", D, "root_shoot_ratio", "Final", ("Cited", 0.42), ["acostadiaz2007", "sofi2018"], "Info", "PartlyEncoded", "+42% (Acosta-Díaz, shoot -26%, root -44%), +70% (Sofi)."),
 ("drought_shoot_mass", D, "shoot_mass_g", "Final", ("Cited", -0.26), ["acostadiaz2007", "gomes2000"], "Info", "Emergent", "Shoot dry weight -26% (Acosta-Díaz); reduced in all cultivars (Gomes)."),
 ("drought_leaf_area", D, "leaf_area_m2", "Max", ("Decrease",), ["gomes2000"], "Info", "Emergent", "LAI reduced in all cultivars (Gomes, figure only). No measured percentage found; the plan's 'leaf area falls more than other variables' is not yet backed by a number."),
 # heat
 ("heat_pods", H, "pods", "Final", ("Decrease",), ["sharda2026", "silva2020"], "M0", "Encoded",
  "M0 exit: pod set falls. Filled pods fall more in sensitive accessions (Sharda, figure); pods -26% at 37/26 C (Silva). Hot nights in the field can raise pod count while the pods stay empty (Vargas 2021, +54%), so the seed checks below matter more."),
 ("heat_seeds", H, "seeds", "Final", ("Decrease",), ["vargas2021", "sharda2026"], "M0", "Encoded", "Seeds per plant -13% (hot nights) and -20% (hot days) (Vargas, Table 1)."),
 ("heat_seed_mass", H, "seed_mass_g", "Final", ("RelChange", -0.71, -0.50), ["sharda2026"], "Info", "Encoded",
  "Total seed weight -50% (tolerant) to -71% (sensitive) after 7 days at 32/22 C over flowering to podding, the closest regime to the scenario's 32/20 C for 6 days."),
 ("heat_seed_size", H, "mean_seed_mass_g", "Final", ("RelChange", -0.34, -0.08), ["sharda2026"], "Info", "Emergent", "Single seed weight -8% (tolerant) to -34% (sensitive)."),
 ("heat_seeds_per_pod", H, "seeds_per_pod", "Final", ("Cited", -0.27), ["vargas2021"], "Info", "Emergent", "-41% with hot nights, -27% with hot days (Table 1); the scenario raises day and night temperature."),
 # shade
 ("shade_total_mass", SH, "total_dry_mass_g", "Final", ("RelChange", -0.50, -0.30), ["cossu2021"], "M0", "Emergent",
  "M0 exit: total mass 30-50% lower. Total dry mass -33% to -52% under 45% less PAR (Cossu 2021, Fig. 5A)."),
 ("shade_internode", SH, "mean_internode_cm", "Final", ("RelChange", 0.20, 3.0), ["lopes1987", "hanyu1996"], "M0", "Encoded",
  "M0 exit: internodes at least 20% longer. Height +99% at 62% light with leaf number unchanged (Lopes 1987), so internodes about doubled; the bush type lengthens through internodes only (Hanyu 1996)."),
 ("shade_sla", SH, "sla_cm2_per_g", "Day(40)", ("RelChange", 0.10, 0.40), ["cossu2021"], "M0", "Encoded", "M0 exit: specific leaf area higher. Up to +32% (cycle 1) and +15% (cycle 3), Cossu 2021 Fig. 5D."),
 ("shade_height", SH, "height_m", "Max", ("Cited", 0.99), ["lopes1987"], "Info", "Encoded", "39.2 to 78.0 cm at 62% light; a stronger shade gives more (+159% at 30%)."),
 ("shade_main_nodes", SH, "main_nodes", "Final", ("RelChange", -0.15, 0.15), ["hanyu1996", "lopes1987"], "Info", "Encoded", "Node and leaf number unchanged in the bush type."),
 ("shade_pods", SH, "pods", "Final", ("Cited", -0.35), ["cossu2021"], "Info", "Emergent", "Pod number -35% (Fig. 6B)."),
 ("shade_root_mass", SH, "root_mass_g", "Max", ("Cited", -0.30), ["lopes1987"], "Info", "Emergent", "Maximum root dry weight -30% at 62% light, -50% at 30%."),
 # M2 density (scenario not yet written)
 ("density_leaf_area_per_plant", "m2/density_35", "leaf_area_m2", "Max", ("Cited", -0.70), ["clavijo2019"], "Later", "Emergent",
  "Baseline m2/density_5. Leaf area per plant above 0.3 m2 at 5 plants/m2 and below 0.1 m2 at 35 (text)."),
]
lines = ["// Response targets (design plan section 8.3): each scenario against response/control.",
         "// rel_change = scenario / baseline - 1. Schema: docs/validation-data.md.",
         "ValidationResponses(", "  schema: 1,", '  species: "bush_bean",', "  responses: ["]
ids = set()
for (id, sc, var, at, e, so, gate, tag, note) in R:
    assert id not in ids, id; ids.add(id)
    base = "m2/density_5" if sc.startswith("m2/") else C
    lines.append(f'''    Response(
      id: {s(id)},
      scenario: {s(sc)},
      baseline: {s(base)},
      variable: {s(var)},
      at: {at},
      expect: {expect(e)},
      sources: {srcs(so)},
      gate: {gate},
      tag: {tag},
      note: {s(note)},
    ),''')
lines += ["  ],", ")", ""]
w("responses.ron", "\n".join(lines))

# ---------- parameters.ron
P = [
 ("vcmax25", ("Range", 49.0, 155.0), "umol/m2/s", "25 C; Pimentel's basis not stated (likely Ci); Wullschleger Ci basis", ["pimentel2011", "wullschleger1993", "mckenziegopsill2020"],
  "Pimentel 89.3 (n = 4, Carioca); common-bean range 49-91 in Wullschleger (as quoted by Pimentel); 76.5 rising to 155.4 during vegetative growth in the field (McKenzie-Gopsill). Herbaceous annual mean 75."),
 ("jmax25", ("Range", 96.0, 166.0), "umol/m2/s", "25 C", ["pimentel2011", "wullschleger1993"], "Pimentel 159.8; Wullschleger bean range 96-165 (as quoted by Pimentel); annual mean 154."),
 ("rd25", ("Approx", 0.88, 0.5), "umol/m2/s", "day respiration fitted from the A/Ci curve, not a dark measurement", ["pimentel2011"], "No measured mature-leaf dark respiration for bean was found."),
 ("specific_leaf_area", ("Range", 250.0, 350.0), "cm2/g", "CROPGRO SLAVR, standard conditions", ["cropgro_drybean_spe"], "SLAVR 305 (Andean Type I), 310 (Meso Type I); 250-350 across cultivars."),
 ("leaf_area_potential", ("Range", 133.0, 180.0), "cm2", "largest full trifoliate (CROPGRO SIZLF)", ["cropgro_drybean_spe"], "SIZLF 133-180 cm2."),
 ("seed_mass", ("Range", 0.22, 0.66), "g", "maximum seed mass (CROPGRO WTPSD)", ["cropgro_drybean_spe"], "Meso Type I 0.23, Andean Type I 0.60."),
 ("seeds_per_pod", ("Range", 3.0, 5.5), "-", "CROPGRO SDPDV", ["cropgro_drybean_spe"], "Andean 3.5, Meso 5.2."),
 ("flowers_per_raceme", ("Range", 3.0, 8.0), "-", None, ["santanna_climbers"], "Axillary racemes of 3-8 flowers, occasionally more."),
 ("pod_set_base", ("Range", 0.20, 0.55), "-", "fraction of flowers that make a mature pod", ["binnie1981"], "45-80% of flowers and young pods are shed across seven cultivars (inferred set fraction)."),
 ("phyllochron", ("Range", 50.0, 70.0), "Cd", "base 8 C as in the species file; CROPGRO TRIFL is per thermal day (22 Cd above 5 C)", ["cropgro_drybean_spe"],
  "TRIFL 0.34 (0.32-0.36) leaves per thermal day = 61-69 Cd per node above 5 C, about 56 above 8 C (inferred conversion)."),
 ("leaf_frost_kill", ("Range", -3.5, -0.8), "C", None, ["cropgro_drybean_spe", "meyer2001", "gubbels1969"], "CROPGRO FREEZ1 -2.22 (leaf damage), FREEZ2 -5.0 (death); seedling LT50 -3.25 to -3.5 (Meyer); no serious injury above -0.8 (Gubbels)."),
 ("phenology.t_base", ("Range", 5.0, 10.0), "C", None, ["cropgro_drybean_spe", "dapaah1999", "pinzon2024", "cavalcante2020"], "Bases in use: 5 (CROPGRO), 7 (Dapaah, from a development-rate regression), 8.3 (Pinzón), 10 (Cavalcante)."),
 ("phenology.flowering", ("Range", 320.0, 600.0), "Cd", "base 8 C from germination, as in the species file", ["pinzon2024", "cavalcante2020", "dapaah1999", "croptime2020"],
  "Approximate conversions of the flowering sums to base 8 (inferred, depends on the temperatures of each trial): Dapaah about 350, CROPTIME about 470, Cavalcante about 515, Pinzón about 600. The default 320 is at the low end."),
 ("phenology.maturity", ("Range", 740.0, 980.0), "Cd", "base 8 C from germination, as in the species file", ["pinzon2024", "cavalcante2020", "dapaah1999"],
  "Approximate base-8 conversions: Dapaah about 770, Pinzón about 790, Cavalcante about 975 (inferred)."),
]
lines = ["// Published values for species-file parameters (data/species/bush_bean.ron), compared",
         "// with the file's defaults. Schema: docs/validation-data.md.",
         "ParameterChecks(", "  schema: 1,", '  species: "bush_bean",', "  checks: ["]
for (p, e, unit, basis, so, note) in P:
    lines.append(f'''    ParamCheck(
      parameter: {s(p)},
      expect: {expect(e)},
      unit: {s(unit)},
      basis: {opt(basis)},
      sources: {srcs(so)},
      gate: Info,
      note: {s(note)},
    ),''')
lines += ["  ],", ")", ""]
w("parameters.ron", "\n".join(lines))

# ---------- curves: Pinzón logistic fits evaluated, Maldonado leaf area
fits = {1: {"DM_root": (2.4150, 0.03874, 272.38), "DM_shoots": (3.1459, 0.0099, 347.64), "DM_pods": (8.8839, 0.03952, 734.42)},
        2: {"DM_root": (1.2501, 0.0302, 301.65), "DM_shoots": (1.6972, 0.01373, 285.80), "DM_pods": (5.7600, 0.1117, 786.50)}}
end = {1: 803.5, 2: 808.6}
var = {"DM_root": "root_mass_g", "DM_shoots": "vegetative_shoot_mass_g", "DM_pods": "fruit_mass_g"}
for per, fs in fits.items():
    for k, (a, b, c) in fs.items():
        xs = [x for x in range(100, 801, 50)] + [end[per]]
        pts = [(x, round(a / (1 + math.exp(-b * (x - c))), 4), None) for x in xs]
        tab = 2 if per == 1 else 3
        w(f"curves/pinzon2024/{var[k]}_period{per}.ron", curve(
            f"pinzon2024_{var[k]}_period{per}", "pinzon2024",
            f"Greenhouse pot plant, BRSMG Marte, period {per} ({'April-June' if per == 1 else 'July-September'} 2023)",
            "thermal_time_cd_b8.3", "Cd", var[k], "g/plant",
            "x: mean-method degree-days from sowing, base 8.3 C. Shoot means leaf + stem (the paper does not separate them).",
            f"Evaluated from the published logistic fit Y = a/(1+exp(-b(x-c))) in Table {tab} (a {a}, b {b}, c {c}; R2 0.9), every 50 Cd and at R8; not measured points (the weekly data are in Fig. 4 only)",
            pts, comment=f"Pinzón-Sandoval 2024, {var[k]}, period {per}: points evaluated from the paper's logistic fit."))
mal = [(15, 77.99, 63.15), (22, 118.09, 111.19), (29, 172.52, 166.73), (36, 309.87, 336.82), (43, 449.77, 468.99),
       (50, 706.81, 712.12), (57, 1153.45, 1202.47), (64, 1727.47, 1897.89), (71, 1876.76, 2362.32), (78, 2924.69, 3183.73)]
for i, (cv, hab) in enumerate([("cerinza", "ICA-Cerinza (Type I)"), ("tundama", "ICA-Tundama (Type II)")]):
    w(f"curves/maldonado1993/leaf_area_{cv}.ron", curve(
        f"maldonado1993_leaf_area_{cv}", "maldonado1993",
        f"Field, Mosquera, Colombia, 2640 m, mean 14 C, 0.50 x 0.12 m; {hab}; mean of 12 plants",
        "days_since_emergence", "d", "leaf_area_m2", "m2/plant", "Cool high-altitude climate: slow development, so compare shape more than timing.",
        "Cuadro 1 (total leaf area per plant, cm2, converted to m2)",
        [(d, round(v[i] / 1e4, 6), None) for d, *v in mal],
        comment=f"Maldonado and Corchuelo 1993, leaf area per plant, {hab}."))
print("sources", len(SRC), "targets", len(T), "responses", len(R), "params", len(P))
