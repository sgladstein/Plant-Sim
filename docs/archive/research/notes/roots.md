# Root architecture and root water uptake models, with common bean root parameters, compared with design plan v3 (sections 3.6, 3.7, 3.16, 8.5)

Source-quality note: the main primary source here is the actual OpenSimRoot bean parameter set (cultivar Carioca), downloaded from the OpenSimRoot GitLab repository on 2026-11-05 and read as data. Saved copies: /tmp/claude-0/-home-claude-Plant-Sim/b9667701-e1b8-5d1a-aab2-5cb6e2180076/scratchpad/downloads/roots/osr/ (carioca.xml, growthrates.xml, angles.xml, respiration.xml, exudates.xml, rootHairs.xml). The CPlantBox soybean files are in .../downloads/roots/cpb/. The XML comments cite the original papers (Lynch & van Beem 1993, Miller et al. 2003, Walk et al. 2006, Nielsen et al. 1994/2001). Those comments are the parameter authors' notes; I did not re-read the cited papers this session. Items marked "(from prior knowledge, not re-verified this session)" are well-known citations I could not fetch, because PMC and PubMed pages were behind a captcha and Consensus was rate-limited. Treat them as needing confirmation.

Base URL for the OpenSimRoot bean files: https://gitlab.com/rootmodels/OpenSimRoot/-/tree/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca (raw files: https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml, and so on for carioca.xml, angles.xml, respiration.xml, exudates.xml, rootHairs.xml).

## 1. OpenSimRoot / SimRoot bean parameter set: root classes, growth rates, branching, angles, diameters, costs

### Takeaway
OpenSimRoot ships a full bean parameter set, "Bean-Carioca-SimRoot4", ported from the original SimRoot 4 input file and commented. Its root system is built from five kinds of axes plus laterals:
- primary (tap) root
- basal roots in whorls (2 whorls of 4 roots; the original SimRoot file had 8 and 6)
- up to 10 hypocotyl-borne (adventitious) roots
- first-order laterals (normal and "fast-growing" classes)
- fine second-order laterals

Our plan models only tap root plus 1st and 2nd order laterals. It omits basal and hypocotyl-borne roots, which together carry most of a bean plant's root length (Miller et al. 2003, as quoted in the file: 120–170 m of basal plus adventitious root length at 21 days, excluding the primary root system).

### Cited findings: root classes and topology (carioca.xml)
- Root class tree:
  - hypocotyl (rootClassID 96) bears `hypocotylBornRoots` and `basalWhorl1`/`basalWhorl2` (a third whorl is commented out).
  - primaryRoot (100) bears `lateralPrimaryRoot` and `lateralPrimaryRootFastGrowing`.
  - Basal whorls (99) bear `lateralBasalRoots`.
  - Hypocotyl-borne roots (98) bear `lateralHypocotylBornRoots`.
  - First-order laterals (97) bear `finelateral` / `finelateralFastGrowing`.
  - Fine laterals have no branches.
  - [OpenSimRoot carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Basal whorls:
  - Whorl 1 appears at branchingTimeOffset 4.167 d (100 h) with 4 roots per whorl. Whorl 2 appears at 6.25 d (150 h) with 4 roots, 0.5 cm further along the hypocotyl.
  - A comment says the original SimRoot file had 2 whorls of 8 and 6 basal roots, and that this "seems too much for carioca", so it was changed to 2×4.
  - Another comment says all basal primordia are already present in the seed, and lower whorls develop faster at first.
  - [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Hypocotyl-borne roots: interBranchingDistance 0.4 cm, branchingTimeOffset 10 d, maxNumberOfBranches 10, not allowed above ground. In the original SimRoot they branched from the stem; OpenSimRoot attaches them to the hypocotyl/top of the primary root. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Lateral branching on the primary root:
  - Normal laterals: interBranchingDistance 0.1 cm (10 per cm), with an unbranched apical zone (`lengthRootTip`) of 10 cm.
  - Fast-growing laterals: interbranch distance drawn uniformly from 1–5 cm, same 10 cm apical zone.
  - A comment says the original file's value (0.5 cm = 2 per cm) was "too low to explain Lynch and van Beem 1993", and that the 2008 "three sisters" experiment showed 12 laterals per cm on the primary root.
  - [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Laterals on basal roots: interBranchingDistance 0.15 cm (about 7 per cm, "very consistent 7/cm for basal roots in 2008 three sister experiment"). Apical unbranched zone is 8 cm (whorl 1) or 10 cm (whorls 2–3). [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Laterals on hypocotyl-borne roots: interbranch distance 0.4 cm, apical zone 10 cm. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Fine (2nd-order) laterals on 1st-order laterals: interbranch distance 0.5 cm (0.55 cm on fast laterals), apical zone 4 cm. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Tissue density (`specificWeight`) is 0.094 g/cm³ for every root class. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)

### Cited findings: elongation rates (cm/day versus root age) and diameters (growthrates.xml)
All values are from [growthrates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml).

| Root class | Potential elongation rate | Stochastic multiplier | Diameter | Notes in the file |
| --- | --- | --- | --- | --- |
| Primary (tap) root | 2.357 cm/d constant, 0–250 d | none | 0.09 cm | "Lynch and van Beem 1993: 33 cm in 14 days = 2.357"; original file 0.098 cm/h = 2.352 cm/d; "Miller et al 2003 primary root is 0.8–1 mm thick" |
| Basal roots (each whorl) | 0.1 at day 0 → 4 cm/d from day 3 (day 2 for whorl 3) to day 10 → 3 at day 15 → 2.352 from day 25 to 35 → 0 at day 40 | none | 0.068 cm | Original was 0.17 cm/h = 4.08 cm/d, judged "high, maybe hydroponics based"; "CIAT field observations: basals can grow at least 2 m long"; Miller 2003 gives 0.65–0.72 mm |
| Hypocotyl-borne roots | 1 cm/d → 0.8 at day 25 → 0 at day 35 | normal, mean 1, sd 0.1 | 0.064 cm | Miller 2003: 0.64 mm |
| Laterals on primary root | 0.768 cm/d for 6 d, 0 by day 10 | normal, mean 1.0, sd 0.1 | 0.03 cm | Original 0.032 cm/h; Walk et al. 2006: tap laterals grow for 288 h, basal laterals 180 h, adventitious and 2nd-order laterals 72 h; "laterals are about 1/3 mm" (pot observation 2009) |
| Fast-growing laterals on primary root | 0.768 cm/d to day 15, 0 by day 20 | mean 1.6 (1.4–1.8) | 0.03 cm | "very long laterals with a lot of branches; grow nearly as fast as the primary root up to 30–40 cm" (hydroponics) |
| Laterals on basal and hypocotyl-borne roots | 0.768 cm/d to day 4 → 0.2 at day 6 → 0 at day 10 | mean 0.8 | 0.03 cm | |
| Fine laterals | 0.2 cm/d to day 3, 0 by day 5 | mean 0.6 | 0.01 cm | |
| Fast fine laterals | 0.5 → 1 cm/d over days 0–3, 0 at day 4 | mean 0.6 | 0.015 cm | "get 2–3 cm long at least" |
| Hypocotyl | 3 cm/d over days 1–2, ending by day 5 | none | 0.4 cm | |

- Secondary (radial) growth rate is a table rising to 0.003 cm/d by about day 29. It is scaled per class by a factor: 4 for the primary root (dropping to 2 after 40 d), 0.7 for basal roots (dropping to 0.4), 0.6 → 0.2 for hypocotyl-borne roots, 0.1–0.3 for laterals. So the tap root thickens most. [growthrates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml)
- Miller et al. 2003 (quoted in the file): in sand culture at 21 days, beans had 35–76 m of adventitious root length and 50–140 m of basal root length, 120–170 m in total excluding the primary root system. The file's author comments "This seems rather a lot." [growthrates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml)

### Cited findings: angles and tropisms (angles.xml)
All values are from [angles.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/angles.xml). Note: in SimRoot, a basal root's growth angle comes from its gravitropism strength, not its insertion angle.

| Root class | Branching (insertion) angle | Gravitropism | Soil impedance (random deflection) | Other |
| --- | --- | --- | --- | --- |
| Primary root | 0° | 0.011 (v2: uniform −0.015 to −0.005) | 0.01 | 16 xylem poles |
| Basal whorl 1 | 90° | 0.002 (v2: −0.08 to −0.04) | 0.008 | |
| Basal whorl 2 | 90° | 0.001 (v2: −0.04 to −0.02) | 0.008 | |
| Basal whorl 3 | 90° | 0.0005 (v2: −0.02 to −0.01) | 0.008 | |
| Hypocotyl-borne roots | 85° | 0 (v2: −0.002 to 0) | 0.003 | grow nearly horizontal |
| 1st-order laterals | 75° | 0 | 0.02 (v2: ±0.1 to ±0.2) | |
| Fine laterals | 75° | 0 | 0.5 | |

The basal whorl rows encode the shallow-upper-whorl / steep-lower-whorl gradient.

### Cited findings: costs (respiration.xml, exudates.xml, rootHairs.xml, carioca.xml)
- Root respiration: 0.09 g/g/day for the first 2 days of segment age, then 0.04 g/g/day, for all root classes. [respiration.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/respiration.xml)
- Measured values quoted in the respiration.xml comments ([respiration.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/respiration.xml)):
  - Nielsen et al. 2001 (28-day-old bean): 3–16 µmol/g/day.
  - Fan et al. 2003: apical root segments respire about 3× more than older segments (about 0.09 versus 0.03–0.045 g/g/day).
  - Gniazdowska et al. 1998 (bean): 30 nmol/g/s ≈ 0.031 g/g/day, higher in young plants and lower under P deficiency.
  - The file's authors note a large unresolved gap between segment-level and whole-root-system measurements.
- Root exudation: 1.7e-6 g C per cm² of root surface per day for new segments, falling to 1.2e-6 by 10 days of segment age and to 0 at 20 days.
  - Basis: Nielsen et al. 1994 (bean), about 4.7–6.7 nmol C/mm/h near the tip.
  - The comment says this yields about 0.12 g C per g root per day, a little less than respiration (about 2/3 of it). It also says monocot exudation is about 1/5–1/3 of bean's (Groleau-Renaud 1998; Sauer et al. 2006).
  - [exudates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/exudates.xml)
- Root hairs: density 2000 per cm² of root surface on primary and basal roots. Length 0.03 cm, starting at 1–2 d segment age. Diameter 5e-4 cm.
  - Comments: Yan et al. 2004 found 40–80 hairs/mm on bean basal roots and 100 hairs/mm on the primary root. Föhse et al. 1991 gives 50 hairs/mm, 0.2 mm long, and notes bean has the second-lowest hair surface area of the species compared.
  - [rootHairs.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/rootHairs.xml)
- Seed and allocation ([carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)):
  - Seed mass 0.20 g. Comment: "small seeded beans about 0.3 g"; Yan et al. 1995 gives seed size 16–63 g per 100 seeds, Carioca 23.8 g per 100.
  - Seed reserve duration 25 d, reserve release rate 40% per day.
  - Carbon-to-dry-weight ratio 0.45.
  - `carbonAllocation2RootsFactor`, as a fraction of total allocation: 1.0 on day 0, 0.5 on day 1, 0.2 from day 5 onward.
- Nitrogen costs ([carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)):
  - Biological N fixation: 3.95e-5 g C per µmol N = 39.5 g C per mol N ≈ 2.8 g C per g N. This follows CROPGRO's 7.06 g glucose per g N.
  - The comment also quotes Minchin & Witty 2005: total legume cost including nodule construction and maintenance is 3–5 kg C per kg N fixed (7–12 mol CO2 per mol N2).
  - Nitrate uptake: 1.392e-5 g C per µmol N (from Bouma et al. 1996, upper value).
  - relativeRelianceOnBNF = 0.30.

### Cited findings: field and phenotyping context
- Shovelomics of field-grown bean and cowpea:
  - Basal root whorl number and basal root number measured in the lab agree with field values at pod elongation (R² 0.66 and 0.92).
  - Cowpea has no basal root whorls; this is a bean-specific trait.
  - [Burridge et al. 2016, Field Crops Res.](https://consensus.app/papers/details/ea8c70e7d0555e3485ab7c5b68ed3446/?utm_source=claude_desktop)
- Seedling phenotypes of 577 bean genotypes predicted seed yield: basal root number mattered in 22% of 51 environments, adventitious root abundance in 35%, and taproot length in 12%. [Strock et al. 2019, Field Crops Res.](https://consensus.app/papers/details/dd110c5b35865542aeacbde81af76c6e/?utm_source=claude_desktop)
- Seed yield correlates positively with deep roots and root angles ≥ 40° from horizontal across 6 environments. [Kimurto et al. 2023, Field Crops Res.](https://consensus.app/papers/details/8f95f364b56c5f6eb1b0fa2998d6a787/?utm_source=claude_desktop)
- Drought-tolerant, deep-rooted genotypes have more root biomass and higher taproot lateral branching density. P-stress adaptation is linked to shallow root systems. [Camilo et al. 2021, Agronomy](https://consensus.app/papers/details/c71854535a2a5583a70dac1bab46760d/?utm_source=claude_desktop)
- Basal root number correlates positively with adventitious root number. Root growth angle is highly heritable. [Ibram et al. 2025, J. Appl. Genet.](https://consensus.app/papers/details/e32ee535859f52cda8140d4d129e74f5/?utm_source=claude_desktop)
- OpenSimRoot has been used for in-silico soil coring of deep, intermediate and shallow bean root phenotypes. Root length density versus depth is best estimated between rows, 5 cm from the plant base. [Burridge et al. 2020, Plant Phenomics](https://consensus.app/papers/details/fe61944f04f3586aa0cf6664592f2e18/?utm_source=claude_desktop)
- Background on SimRoot's origins, from prior knowledge and not re-verified this session:
  - SimRoot was introduced for common bean in Lynch, Nielsen, Davis & Jablokow 1997, Plant and Soil 188:139–151 (doi:10.1023/A:1004289903980).
  - Basal root gravitropism and shallowness improve P acquisition (Bonser, Lynch & Snapp 1996, New Phytol.; Lynch & Brown 2001, Plant Soil 237:225–237).
  - Walk et al. 2006 (Plant Cell Environ.) modelled trade-offs between basal root shallowness and deep water access.
  - Miller, Ochoa, Nielsen, Beebe & Lynch 2003 (Funct. Plant Biol. 30:973–985) showed hypocotyl-borne (adventitious) roots are a cheap, topsoil-foraging root class that improves P acquisition.
  - OpenSimRoot is described in Postma et al. 2017, New Phytol. 215:1274–1286 (doi:10.1111/nph.14641).

### Inferences
- Specific root length per class, computed from the OpenSimRoot diameters and tissue density 0.094 g/cm³ using SRL = 1 / (ρ·π·r²). This is my calculation.

  | Root class | Diameter | SRL |
  | --- | --- | --- |
  | Primary root | 0.9 mm | about 17 m/g |
  | Basal roots | 0.68 mm | about 29 m/g |
  | Hypocotyl-borne roots | 0.64 mm | about 33 m/g |
  | 1st-order laterals | 0.3 mm | about 150 m/g |
  | Fast fine laterals | 0.15 mm | about 600 m/g |
  | Fine laterals | 0.1 mm | about 1350 m/g |

  Whole-system SRL depends on the length mix. Laterals dominate length, so the whole-system value is probably about 80–300 m/g, and it falls over time as the tap and basal roots thicken (inference). Our single 70 m/g is plausible as a whole-system average for an older plant, but it is too low for fine roots, and §3.16 labels it "fine roots". Recommendation: derive carbon per cm from each class's diameter and a tissue density of about 0.08–0.1 g/cm³, not from one SRL.
- Implied maximum lengths, from integrating the rate tables (my integration; multipliers at their means):

  | Root class | Implied maximum length |
  | --- | --- |
  | Basal roots | about 105–110 cm (growth stops by day 40) |
  | Hypocotyl-borne roots | about 27 cm (stops by day 35) |
  | Laterals on primary root | about 6 cm |
  | Fast laterals | about 20–25 cm |
  | Laterals on basal roots | about 4–5 cm |
  | Fine laterals | about 0.5 cm |
  | Fast fine laterals | about 2 cm |

  The primary root has no cap (2.36 cm/d indefinitely). In the field it reaches 1 m or more within 6 weeks. In a 20 cm pot, it reaches the bottom about 8–9 days after germination.
- In the CRootBox parameter scheme our plan uses, the bean set maps roughly as follows (inference):
  - Tap root: r ≈ 2.4 cm/d, lmax ≥ 100 cm (field), la ≈ 10 cm, lb ≈ 5 cm (SimRoot original), ln ≈ 0.1 cm.
  - Basal roots: 4–16 per plant in 2–4 whorls. Insertion angle about 90° with weak gravitropism graded by whorl, giving growth angles of about 10–60° below horizontal. r ≈ 2.4–4 cm/d, lmax about 1 m.
  - Hypocotyl-borne roots: 0–10 or more per plant, near horizontal, r ≈ 1 cm/d, lmax about 25–30 cm.
  - 1st-order laterals: insertion 75°, r ≈ 0.77 cm/d over 6–10 d, lmax about 5–6 cm (normal) or 20–30 cm (fast), ln ≈ 0.5 cm, la ≈ 4 cm.
  - 2nd-order laterals: lmax about 0.5–2 cm, r ≈ 0.12–0.6 cm/d.
- The 0.04 g/g/day root respiration is roughly in line with the plan's maintenance respiration, but it is not the same quantity: the plan keys maintenance to nitrogen (0.015 g C/g N/day). Exudation is a separate carbon sink of comparable size (about 2/3 of respiration in the bean set). The plan has no exudation term. Recommend a lumped "rhizodeposition" sink of about 5–15% of carbon allocated to roots (inference; the magnitude is uncertain).

### Gaps
- I could not read Lynch et al. 1997 (the SimRoot bean paper) or Miller et al. 2003 directly; values come via OpenSimRoot file comments. The root-to-shoot partitioning table (0.2 of allocation after day 5) is a model input, not a measurement.
- Root dry mass and length time series for bush bean (as opposed to Carioca, a Mesoamerican type) were not found.

## 2. CRootBox/CPlantBox parameter files: is there a Phaseolus set?

### Takeaway
No Phaseolus or bean root parameter file exists in CPlantBox `modelparameter/structural/rootsystem/`. Guessed paths such as `Phaseolus_vulgaris.xml` return 404. The GitHub directory listing was blocked for this session, so I probed file names directly against raw.githubusercontent.com. The closest legume sets are `Glycine_max.xml` and `Glycine_max_Moraes2020.xml` (soybean), which include a basal-root class. The plan's §8.5 item "check CRootBox example parameter sets for a Phaseolus entry" resolves to: use OpenSimRoot Bean/Carioca instead.

### Cited findings
- `Glycine_max_Moraes2020.xml` parameters. In the CPlantBox convention `a` is radius in cm, r is in cm/d, lengths are in cm and theta is in radians.

  | Root type | a | r | lmax | la | lb | ln | theta | Tropism |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | Tap root (subType 1) | 0.2 | 5.5 | 197.35 | 2 | 1 | 0.65 | 0 | gravitropic N=1.5, σ=0.4 |
  | 1st-order lateral (subType 2) | 0.04 | 2 | 40.3 | 3 | 3 | 0.7 | 1.22 (70°) | N=0.1, σ=0.3 |
  | 2nd-order lateral (subType 3) | 0.02 | 2 | 5 | 5 | 0.2 | 0.2 | 70° | N=0, σ=0.5 |
  | Basal root (subType 4) | 0.06 | 3.5 | 49 | 5 | 2 | 3 | 1.5708 (90°) | N=0.5, σ=0.1 |

  - Seed: firstB 3 d, delayB 7 d, maxB 15. The base `Glycine_max.xml` differs only in having no basal roots (maxB 0) and tap σ 0.2.
  - Root lifetime rlt = 1e11, meaning no root death.
  - [CPlantBox Glycine_max_Moraes2020.xml (raw)](https://raw.githubusercontent.com/Plant-Root-Soil-Interactions-Modelling/CPlantBox/master/modelparameter/structural/rootsystem/Glycine_max_Moraes2020.xml); [CPlantBox repo](https://github.com/Plant-Root-Soil-Interactions-Modelling/CPlantBox)
- Other dicot files confirmed to exist (HTTP 200): `Anagallis_femina_Leitner_2010.xml`, `Lupinus_albus_Leitner_2014.xml`, `Heliantus_Pagès_2013.xml` (figshare copy: [Heliantus CRootBox parameter file](https://figshare.com/articles/Heliantus_-_CRootBox_parameter_file/4828948)). [CPlantBox raw files](https://raw.githubusercontent.com/Plant-Root-Soil-Interactions-Modelling/CPlantBox/master/README.md)
- CPlantBox webapp for exploring parameters: [cplantbox.fz-juelich.de](https://cplantbox.fz-juelich.de), linked from the repo README. [README](https://raw.githubusercontent.com/Plant-Root-Soil-Interactions-Modelling/CPlantBox/master/README.md)
- Background on the CRootBox model family, from prior knowledge and not re-verified this session:
  - CRootBox: Schnepf, Leitner, Landl & Lobet 2018, Ann. Bot. 121:1033–1053, doi:10.1093/aob/mcx221. Root types have r, lmax, la, lb, ln, theta, a, tropism type, N and σ, and successor rules. Elongation follows a negative-exponential approach to lmax, l(t) = lmax(1 − e^(−r·t/lmax)).
  - CPlantBox: Zhou et al. 2020, in silico Plants 2:diaa001.
  - RootTyp: Pagès et al. 2004, Plant Soil 258:103–119.
  - ArchiSimple: Pagès et al. 2014, Ecol. Model. 290:76–84. Elongation rate scales with tip diameter, and laterals' diameters come from a parent-to-child ratio. This removes most per-class tables.
  - Pagès et al. 2013/2014 calibrated typologies on many dicots, but I did not confirm a Phaseolus entry in them.

### Inferences
- The plan's CRootBox-style root type list is structurally right, but it needs a "basal" successor rule on the seed (CPlantBox has firstB, delayB, maxB for exactly this) and a "shoot-borne" rule for hypocotyl roots (CPlantBox has nC, firstSB, delaySB, nz). Both exist in CRootBox, so adding them does not break the CRootBox-style design.
- Soybean's tap root (5.5 cm/d) is faster than bean's (2.36 cm/d). Do not copy soybean rates.
- ArchiSimple's diameter-driven scheme fits a game: tip diameter sets elongation rate and carbon cost per cm. Fewer parameters, and carbon limitation shows up naturally as thinner, slower tips (inference).

### Gaps
- Could not list the whole CPlantBox modelparameter folder, so other legume files under unguessed names (for example a Pagès typology bean file) cannot be ruled out. Confirm with `gh` once GitHub access is enabled.
- Did not obtain Pagès et al. typology values for Phaseolus.

## 3. Elongation versus temperature, maximum rooting depth, root:shoot ratio, root lifespan

### Takeaway
Bean tap root potential is about 2.4 cm/d and basal roots about 2.4–4 cm/d (Lynch & van Beem 1993, via OpenSimRoot). Field basal roots can reach 2 m (CIAT observation in the file comments), and deep genotypes have roots at 40° or more from horizontal. In the default 20×20 cm pot every axis reaches a wall or the bottom within about 1–2 weeks. Root lifespan is not parameterised in either OpenSimRoot bean or CPlantBox soybean: roots never die in those sets.

### Cited findings
- Tap root: 33 cm in 14 days = 2.357 cm/d (Lynch & van Beem 1993). Basal roots in field observations "can grow at least 2 m long". [growthrates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml)
- Root growth angles ≥ 40° from horizontal (deep roots) are associated with yield across environments. [Kimurto et al. 2023](https://consensus.app/papers/details/8f95f364b56c5f6eb1b0fa2998d6a787/?utm_source=claude_desktop)
- OpenSimRoot's bean set has no temperature response table in the files I read. Root growth is modulated by nutrient stress tables (impactOn:rootPotentialLongitudinalGrowth) and by carbon. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Rooting depth versus drought adaptation in bean:
  - Guimarães, Brunini & Stone 1996 measured "root water absorption efficiency" in mm³ of water per cm of root per day. It rises with depth.
  - Drought-adapted BAT 477 and Carioca keep higher root length density deep in the profile.
  - Numeric values are in the PDF, which I could not fetch.
  - [Guimarães et al. 1996, Pesq. Agropec. Bras. 31:393–399](https://apct.sede.embrapa.br/pab/article/view/4497)
- CPlantBox soybean files use rlt = 1e11 (immortal roots). [Glycine_max_Moraes2020.xml](https://raw.githubusercontent.com/Plant-Root-Soil-Interactions-Modelling/CPlantBox/master/modelparameter/structural/rootsystem/Glycine_max_Moraes2020.xml)

### Inferences
- Pot geometry: 20 cm depth at 2.36 cm/d means the tap root reaches the bottom about 8–9 days after germination. Basal roots at 4 cm/d and shallow angles reach the 10 cm radius wall in about 3–5 days of their growth.
  - The pot wall/bottom behaviour (deflect along wall) will dominate root appearance in M1. Real potted beans show root circling and a mat at the bottom; the plan's deflection rule reproduces this qualitatively.
  - Consider letting roots that hit the bottom stop (air pruning at drainage holes) as an option.
- Temperature: use the plan's soil-temperature multiplier with bean cardinal temperatures. A suggested response is zero below about 8–10 °C, optimum about 25–30 °C, zero above about 38 °C (inference consistent with §3.11; I found no bean-specific root elongation versus temperature curve this session).
- Root:shoot partitioning: OpenSimRoot gives roots 20% of allocation after day 5. That implies a root:shoot dry-mass ratio of about 0.2–0.25 during vegetative growth, falling during pod fill. The plan's priority scheme (0.6 + 0.4×(1−FTSW)) should be tuned to hit roughly this under no stress (inference).
- Lifespan/turnover: no data found. For a 65–93 day annual, ignoring fine-root death is consistent with both reference models. Optionally kill fine (2nd-order) laterals at about 2–4 weeks of age for visual thinning (inference, unsourced).

### Gaps
- No bean-specific temperature response of root elongation was found.
- Measured maximum rooting depth of determinate bush (snap) bean in the field was not obtained this session. The commonly cited 0.6–1.2 m could not be sourced.
- No root:shoot time series for bush bean was found.
- No bean fine-root lifespan data was found.

## 4. Root water uptake: is "RLD × available water × rate" acceptable? Typical RLD

### Takeaway
The plan's scheme is a macroscopic sink-term model: uptake in each cell is proportional to root length density and limited by relative available water. That is the same family as the Feddes/Gardner/CERES approaches used in crop models. It is acceptable for a game at 2 cm resolution. Its weaknesses:
- the uptake rate depends linearly on available water, with no hydraulic limitation
- it ignores root-to-shoot hydraulic architecture, which matters for Couvreur and Javaux-style compensation
- the "uptake rate" constant needs a physically meaningful value, a maximum per unit root length

Because the plan distributes demand in proportion to potential uptake, it already gives a crude form of compensatory uptake (wet cells supply more).

### Cited findings
- Bean drought studies express uptake per unit root length directly ("root water absorption efficiency", mm³ per cm per day) and use root length density in cm/cm³ as the state variable. [Guimarães et al. 1996](https://apct.sede.embrapa.br/pab/article/view/4497)
- In-silico OpenSimRoot coring of bean computes root length density profiles by depth for shallow, intermediate and deep phenotypes. [Burridge et al. 2020](https://consensus.app/papers/details/fe61944f04f3586aa0cf6664592f2e18/?utm_source=claude_desktop)
- Background models, from prior knowledge and not re-verified this session:
  - Feddes, Kowalik & Zaradny 1978 (Simulation of field water use and crop yield, Pudoc): the sink term S = α(h)·S_max, with S_max distributed by root density. α is a trapezoid in pressure head with thresholds h1–h4. For beans, h3 is commonly about −400 to −1000 cm and h4 (wilting) about −8000 cm.
  - Couvreur, Vanderborght & Javaux 2012 (HESS 16:2957–2971): a macroscopic model derived from root hydraulic architecture. Uptake = (standard uptake fraction × total transpiration) + a compensation term driven by the difference between local and effective soil water potential.
  - de Jong van Lier et al. 2008 (Vadose Zone J. 7:1065–1078): matric flux potential–based uptake. Uptake per layer depends on root length density and root radius through a microscopic geometry.
  - DSSAT/CERES (Ritchie): maximum uptake per unit root length is about 0.03 cm³ of water per cm of root per day, with an exponential dependence on soil water above lower limit.
  - Root length density for crops is typically 0.1–5 cm/cm³. Above about 1 cm/cm³, uptake becomes nearly independent of root length density.
  - R-SWMS (Javaux et al. 2008, VZJ 7:1079–1088) and DuMux-rosi / CPlantBox-dumux (Koch et al. 2018, VZJ) solve Richards' equation coupled to a root xylem network. These are far too expensive for the game.

### Inferences
- Concrete recommendation (inference): define per-cell potential uptake as U_pot = RLD × V_cell × u_max × f(θ_rel). Use u_max ≈ 0.03 cm³ per cm of root per day (CERES-like; check against Guimarães 1996 values). For f(θ_rel), use a Feddes-like curve in relative available water (θ − θ_wp)/(θ_fc − θ_wp): 1 above about 0.4–0.5, falling linearly to 0.
  - With 2 cm cells (8 cm³) and RLD 1 cm/cm³, potential uptake ≈ 0.24 cm³ per cell per day.
  - A pot of about 6,300 cm³ (20×20 cm cylinder, about 785 cells) with RLD averaging 1 cm/cm³ supplies about 190 cm³/day. That is consistent with transpiration of a mid-size bean plant of roughly 100–300 mL/day (inference).
- Add a saturating root length density term, for example U ∝ RLD/(RLD + 0.5 cm/cm³) × θ_rel, so dense root mats do not create unlimited supply. This mimics the de Jong van Lier and CERES ln(RLD) behaviour (inference).
- Using FTSW weighted by root length density for the stress factor, with a 0.4 threshold, is standard practice (Sinclair-type FTSW curves). It is consistent with the uptake function if the same threshold is used.

### Gaps
- Could not extract Guimarães 1996 numbers (efficiency in mm³/cm/day, root length density by depth) or other field root length density profiles for bean.
- Could not verify the Feddes h-thresholds for Phaseolus in a primary source this session.

## 5. Carbon cost of roots: construction cost, SRL, diameter by order

### Takeaway
The bean data (OpenSimRoot) give diameters by order of 0.9 mm (tap), 0.64–0.72 mm (basal and hypocotyl-borne), 0.3 mm (1st-order laterals) and 0.1–0.15 mm (fine laterals), with tissue density 0.094 g/cm³. Fine-root SRL is therefore about 150 m/g for first-order laterals and 600–1350 m/g for fine laterals, far above 70 m/g. For dynamic costs, root respiration is 0.04 g/g/day (0.09 at the tip) and exudation is about 2/3 of respiration.

### Cited findings
- Diameters, tissue density, respiration and exudation as given in section 1. [growthrates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml); [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml); [respiration.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/respiration.xml); [exudates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/exudates.xml)
- Bean roots thicken under water stress in deeper layers (more diameter, surface area and volume), so SRL is plastic. [Riyaz et al. 2024, Discover Plants](https://consensus.app/papers/details/59748a29d45b595c8ccf6ffd34444ab2/?utm_source=claude_desktop)
- Specific root length and root tissue density are routinely measured in bean field studies (for example in work with 6-BAP), but the abstract gives no values. [Gonzales 2025](https://consensus.app/papers/details/03f3d836433355d3a9d868f59ccfd42a/?utm_source=claude_desktop)

### Inferences
- §3.7 calls 70 m/g the SRL of "bean fine roots". That is wrong for fine roots: bean fine laterals are at least about 150 m/g, and 600 or more for 2nd-order laterals. 70 m/g is only defensible as a whole-system average that includes thick tap and basal roots.
  - Fix: carbon per cm = ρ_tissue × π × (d/2)² × 0.45 g C per g dry matter, per root class, with ρ_tissue ≈ 0.094 g/cm³ (OpenSimRoot bean).
  - Example: a 1st-order lateral (0.3 mm) costs about 3.0e-5 g C per cm before growth respiration; the tap root (0.9 mm) costs about 2.7e-4 g C per cm.
- Construction cost 1.25 (growth respiration 20%) is in the normal range for roots, which are roughly 1.2–1.3 g glucose per g dry matter (inference, standard value).
- Secondary thickening of the tap root (OpenSimRoot: up to about 0.012 cm/d radial growth ×4 scaling) is a real carbon sink. The plan lists "secondary thickening 0.4" in priorities, but §3.7 does not say roots thicken. Add tap and basal root thickening for both carbon and rendering (taper).

### Gaps
- No direct measured whole-plant SRL for bush bean was found (search results were not fetchable). Need to confirm with a measured study (for example Butare et al. 2011, Orbi PDF: https://orbi.uliege.be/bitstream/2268/104206/1/N%c2%b0444%20-%20Butare%202011.pdf, not read).

## 6. Nodulation (for M2): carbon cost and timing

### Takeaway
The model sources give a carbon cost of fixation of about 2.8 g C per g N for the fixation process alone (CROPGRO convention), or 3–5 g C per g N including nodule construction and maintenance (Minchin & Witty 2005). The plan's "10–15% of assimilate when active" is within commonly cited ranges. Better: make the cost emergent from N fixed × about 3–5 g C per g N, plus nodule construction.

### Cited findings
- CROPGRO uses 7.06 g glucose per g N fixed (≈ 2.8 g C per g N; 39.5 g C per mol N). That is halfway between soybean values from LaRue & Patterson 1981 and Ryle et al. 1979. [OpenSimRoot carioca.xml comments](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Minchin & Witty 2005, as quoted in the same file: on average, legumes spend 3–5 kg C per kg N fixed (7–12 mol CO2 per mol N2) including nodule construction and maintenance. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- OpenSimRoot bean assumes 30% reliance on biological N fixation. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Hansen et al. 1993 (Planta 189:538–545), on bean cv. OAC Rico and its supernodulating mutant:
  - The mutant had 2× the nodule dry weight and almost 6× the nodule number.
  - Its nodule respiration at day 37 was up to 3× higher.
  - N2 fixation was almost identical in the two, so extra nodule mass costs carbon without adding N.
  - [Hansen et al. 1993](https://link.springer.com/article/10.1007/BF00198217)
- In legumes, H2 evolution from nodules declines at the onset of flowering, before peak N2 fixation, and tracks the carbon exchange of the lowest leaves. [Bethlenfalvay & Phillips 1977, Plant Physiol. 60:419](https://pmc.ncbi.nlm.nih.gov/articles/PMC542628)
- From prior knowledge, not re-verified this session: Kaschuk et al. 2009 (Soil Biol. Biochem. 41:1233–1244) report that rhizobial symbioses consume about 4–16% of photosynthate. Older budgets (Pate and coworkers, for example "Utilization of net photosynthate for nitrogen fixation and protein production in an annual legume", [PubMed 16660179](https://pubmed.ncbi.nlm.nih.gov/16660179/)) report higher shares, up to roughly a quarter to a third of net photosynthate going to nodulated roots. I could not read that abstract.

### Inferences
- Common bean is a relatively poor N fixer among grain legumes. Nodules appear about 1–2 weeks after emergence, activity peaks around flowering to early pod fill, and it falls during seed fill (inference from general bean agronomy; the timing is unsourced here). A fixed 10–15% is a reasonable M2 placeholder. Prefer cost = N_fixed × about 4 g C per g N + nodule growth at root construction cost, with fixation suppressed by high soil mineral N (the plan already has this).

### Gaps
- Primary bean-specific fractions of photosynthate to nodules (for example Hansen et al. 1993 Part II with ¹³C labelling, Planta 189:546–556) were not read.

## 7. Rendering-relevant appearance metrics

### Takeaway
A bean root system looks like:
- a thick (0.8–1 mm) tap root
- a crown of 4–16 basal roots fanning out about 0.5–1.5 cm below the soil surface, in 2–4 whorls with distinct angles (upper whorls shallower)
- up to about 10 or more near-horizontal hypocotyl-borne roots near the surface
- dense, short laterals at about 7–12 per cm, with a 4–10 cm unbranched tip zone

### Cited findings
- Whorl count, basal root number, angles, diameters, lateral densities and apical zones as tabulated in section 1. [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml); [angles.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/angles.xml)
- Shovelomics descriptors visible on field crowns: basal root whorl number, basal root number, basal root growth angle, number of hypocotyl roots, tap root diameter at 5 and 15 cm, secondary branching, nodule score. [Burridge et al. 2016](https://consensus.app/papers/details/ea8c70e7d0555e3485ab7c5b68ed3446/?utm_source=claude_desktop)

### Inferences
- Render roots by diameter class. Fine laterals (0.1 mm) are sub-pixel at plant scale, so draw them as lines or density haze, with root hairs as a soft halo. Draw the tap root with taper from secondary growth.
- Nodules (M2): pink-interiored spheres of about 1–3 mm, clustered on the upper tap root and basal roots (inference, general botany).

### Gaps
- No quantitative crown-image metrics, such as convex hull or width-to-depth ratio over time, were gathered for bush bean.

## Summary of divergences and errors in plan v3

All items below are inferences from the findings above.
1. **Missing basal roots.** This is the defining bean root class: 2–4 whorls, 4–16 roots, insertion about 90° with whorl-graded gravitropism, 2.4–4 cm/d, about 1 m long. Basal root growth angle is the classic heritable "phosphorus efficiency versus deep water" trait, and it fits the plan's "root depth tendency" locus well. Add it as a seed-borne root type (CRootBox already has firstB, delayB, maxB).
2. **Missing hypocotyl-borne (adventitious) roots.** Up to about 10 per plant, horizontal, about 1 cm/d, about 25–30 cm, appearing from about day 10, at 0.64 mm diameter. Miller et al. 2003 show basal plus hypocotyl roots reach 120–170 m per plant by day 21. Add as shoot-borne roots on the hypocotyl.
3. **SRL of 70 m/g labelled as "fine roots" is wrong.** Bean fine roots are about 150–1350 m/g depending on order. Use per-class diameter × tissue density 0.094 g/cm³.
4. **Lateral parameters.** The plan has no numbers yet. Use the OpenSimRoot values: inter-branch distance 0.1–0.15 cm on tap and basal roots (7–12 laterals per cm), apical zone 8–10 cm, lateral rate 0.77 cm/d for 6–10 d, 2nd-order laterals 0.5 cm apart with a 4 cm apical zone.
5. **Uptake "rate" needs a defined unit and saturation.** Recommend u_max ≈ 0.03 cm³ per cm per day with a Feddes-type relative water function, saturating in root length density.
6. **Root exudation/rhizodeposition is absent.** In the bean model it is about 2/3 of root respiration.
7. **No tap/basal root secondary thickening in §3.7**, though priorities list secondary thickening.
8. **Nodule cost of 10–15%** is acceptable as a placeholder. Prefer an emergent cost of about 3–5 g C per g N fixed.
9. **Pot effects.** In the 20 cm pot, the tap root hits the bottom by about day 9. Most of the root system will be wall-deflected, which is acceptable but should be stated in the model card.
10. **§8.5 data gap resolved.** There is no Phaseolus set in CPlantBox; use OpenSimRoot Bean/Carioca (GPLv3 repository: the parameter values are facts, but do not copy the files verbatim into the repo without checking the licence).
