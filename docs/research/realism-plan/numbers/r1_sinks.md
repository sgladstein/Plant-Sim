# R1 carbon sinks: data for the §6 targets and the [K] parameters

Spec: `docs/research/realism-plan/r1-carbon-sinks.md` on refs/pull/107/head (commit 03b1cbc), §3.5, §4.4, §5, §6.
Tags: [F] I read the number in the source (an abstract counts); [K, to verify] from memory, not re-read; [I] my inference or arithmetic.
Date: 2026-11-05. PMC was captcha-blocked. Some numbers came from SciSpace `extract_data` reading of full text; they are marked [F] only where the tool quoted the paper.

## Takeaway (bands for R1 §6 and values for the parameters)

| Target / parameter | Spec value | Recommendation | Basis |
|---|---|---|---|
| `stem_share_peak` (stem/(stem+leaf) at R6) | 0.35–0.55 | **Keep the band, widen the top to 0.60.** Field stem+petiole share: 0.59–0.66 (UFGA8501), 0.43–0.48 (Lopes et al. 1983, pots, 3 light levels). If petioles are 20–30% of leaf (blade+petiole) mass, moving them to the leaf gives 0.51–0.57 for UFGA8501 and about 0.33–0.40 for Lopes. | [F] data; [I] conversion; petiole share [K] |
| `stem_decline_fill` (R6 peak to maturity) | −10 to −35% | **Keep.** UFGA8501 −33 to −36% includes petioles, which fall with the leaves, so the true stem decline is smaller. Bean stem free sugars fall from about 7% to 1.6% DM from end of vegetative growth to maturity (Peña-Valdivia 1991). Middle-stem starch fell 42% in 7 days of pod fill (Waters et al. 1980). | [F]; band [I] |
| `basal_stem_diameter` (hypocotyl at R6) | 5–10 mm | **Narrow to 4.5–9 mm** and record density. At maturity in the field: hypocotyl 4.0–5.4 mm 1 cm below the cotyledon node in dense Brazilian plots (about 24 plants m⁻²); basal stem 7.6–8.0 mm just above the soil in North Dakota rows 0.76 m apart. A breeder threshold for upright Type II is 5.6 mm. A single, uncrowded pot plant should be at the top of the band. | [F] |
| `tap_root_diameter` (5 cm below collar, R6) | 3–8 mm | **Lower to 2–6 mm** and treat it as Info until better data exist. Shovelomics (Burridge et al. 2016) measured it 2 cm below the hypocotyl at 35–45 DAP; their figure caption says "a range up to 3 mm may be expected" in the other trials. That wording is ambiguous: it may mean values up to 3 mm or a 3 mm spread. The OpenSimRoot rates (0.003 cm d⁻¹ × 4, then × 2 after day 40) integrate to roughly 7–8 mm by day 60 if the rate is radial: check whether OSR's rate is radial or diametric. | [F] Burridge; [I] integration |
| `h_L` (pipe area per leaf area) | start 3e-4 m² m⁻² | **Keep 3e-4; plausible range 1.5e-4–4e-4.** Basal stem area / leaf area per plant: 4.66e-5 m² (7.7 mm) / about 0.15 m² ≈ 3.1e-4; 1.96e-5 m² (5 mm) / 0.067 m² (UFGA8501: LAI 3.5 at 51.7 plants m⁻²) ≈ 2.9e-4; 1.8e-5 m² (4.8 mm) / about 0.125 m² ≈ 1.5e-4. Woody-branch Huber values are about 1e-4 (sapwood only), so whole-stem herb values above that are consistent. | [I] from [F] diameters and LAI |
| `g_max` (radial cambial rate cap) | 0.1 mm d⁻¹ | **Keep 0.1 mm d⁻¹ as the cap.** The mean radial rate a bean hypocotyl needs is about 0.03–0.05 mm d⁻¹ (from about 3 mm at emergence to 5–8 mm by R6–R8 over 50–70 d). Sunflower reaches about 20 mm stem diameter at 1 m height by maturity (about 0.15 mm d⁻¹ radial, including primary growth). A cap of 2–3× the bean mean leaves carbon in control. | [I]; emergence diameter [K]; sunflower [F] |
| `storage_capacity` (s_max) | bean 0.25, wheat 0.35 | **Bean 0.25 is defensible; 0.15–0.30 for a field calibration.** Greenhouse Puebla 152 stems held 30–68% starch DM (Waters et al. 1980). These are extreme values, probably from thin stem sections; they show a high ceiling. Field soluble sugars are about 7% DM at peak (Peña-Valdivia 1991). **Wheat: 0.35 is the top of the range, 0.20–0.30 is typical.** WSC 25–38% DM at peak, 10–14 d after anthesis (Davidson & Chevalier 1992); genotype means 11–21% at anthesis (Ruuska et al. 2006). | [F] |
| wheat WSC decline in fill | – | Peak 10–14 DAA, then down to <5% DM at maturity (Davidson & Chevalier 1992). Mobilized per main stem about 0.35–0.9 g (peduncle 70–244 mg, penultimate internode 95–227 mg, lower internodes 175–450 mg; Ehdaie et al. 2006). Contribution to grain: about 10–20% (irrigated) up to 40–70% under drought (reviews); 34–50% potential in hayed-off crops (van Herwaarden et al. 1998). | [F] |
| `exudation_fraction` (e, per unit root maintenance respiration) | 0.67 | **Use 0.67 only as an upper value. Consider 0.25–0.5.** OSR's own note gives exudation ≈ 2/3 of root respiration. Independent data are lower: legume soluble exudate (root-wash ¹⁴C) is 2.7% of net assimilated ¹⁴C (Warembourg et al. 2003); rhizodeposition is about 5–10% of assimilated C (Nguyen 2003; Yan et al. 2026). Bean root respiration is about 20% of daytime net assimilation at adequate P and 40% at low P (Nielsen et al. 2001). Together: e ≈ 0.1–0.5. | [F] numbers; ratio [I] |
| `phototropism_max_per_day` (θ_max) | 10° | **10° is conservative; 20–30° d⁻¹ is defensible for seedlings and elongating apices.** Green sunflower hypocotyls reach 25° maximum about 100 min after unilateral blue light (about 15–20° h⁻¹ transient). Etiolated Arabidopsis reaches 16° by 80 min. Light-grown potato shoots reach 90° in 75–120 min. Etiolated pea reaches 25° in 2 h. These are short lab bursts under strong one-sided light; under a canopy the gradient is weak. No bean stem kinetics found. | [F] lab; daily cap [I] |
| `phototropism_weight` (k_p) | 0.3 | No data; keep as a calibration knob. | [I] |
| bean leaf tracking (aside) | – | The bean laminar pulvinus bends phototropically: under adaxial 500 µmol m⁻² s⁻¹ the leaf elevation rose about 50° overall, with an initial constant rate reported as about 1.1–1.2° h⁻¹ over the first 30 min (Koller et al. 2001). This is a leaf-angle process, separate from stem phototropism. | [F] via SciSpace extract; the low rate against the 50° total is odd, so check it |

## Findings by item

### 1. Hypocotyl / basal stem diameter
- Anjos et al. 2018, Crop Breed. Appl. Biotechnol. 18(4):417–425, doi:10.1590/1984-70332018v18n4a61 [F]. Hypocotyl diameter measured with a digital caliper 1 cm below the cotyledon node at physiological maturity (C0 F2 plants). The C1 F4 measurement stage is not stated. Coimbra, MG, Brazil; dry season; rows 0.5 m apart at 12 seeds m⁻¹. Mean of 40 populations 4.76 mm; population range 4.31–5.40 mm; controls 4.48 mm (A525 5.20, Pérola 4.04). Parents are Type II to III.
- Oliveira et al. 2015, Genet. Mol. Res. 14(3):11515–11523, doi:10.4238/2015.September.28.3 [F abstract]: hypocotyl diameter is a selection proxy for upright architecture. No values in the abstract.
- Rodríguez Rodríguez et al. 2026, Euphytica, doi:10.1007/s10681-026-03780-z [F]. Stem diameter at maturity (80% of pods dry), measured "right above the soil surface" with calipers, 2 plants per plot. Four North Dakota sites, 2020–2022, rainfed; rows 0.76 m apart. Market-class means 7.6–8.0 mm; overall 7.7 mm. Cites a 5.6 mm threshold for Type II direct-harvest architecture: Soltani et al. 2016, Crop Sci. 56:3005–3015, doi:10.2135/cropsci2016.02.0119 [F, as cited].
- Burridge et al. 2016 measured stem diameter at soil level with calipers, but the text I read reports no values [F].
- V3 and R1 values: none found. My estimate, inferred and not sourced: about 3–4 mm at V1–V3 for medium-seeded beans, close to the hypocotyl's primary diameter [K, to verify].
- Climbing (Type IV) types: no diameter data found. Costa et al. 1983 (PAB 18(2):137–146, doi:10.1590/S1678-3921.pab1983.v18.15170) compared bush and climbing cultivars but reports partitioning only [F abstract].

### 2. Tap and basal root diameter
- Burridge, Jochua, Bucksch & Lynch 2016, Field Crops Res. 192:21–32, doi:10.1016/j.fcr.2016.04.008 [F]. Bean tap root diameter measured with calipers 2 cm below the base of the hypocotyl; excavated 35–45 DAP (flowering to pod elongation), Rock Springs PA. Figure 7 caption: the 2014 tap root diameter "was generally smaller than observed in other trials where a range up to 3 mm may be expected." No table of values. Hypocotyl-root diameter was not measured for bean; the 1.5 mm hypocotyl-root threshold is for cowpea.
- Strock, De La Riva & Lynch 2018, Plant Physiol. 176(1):691–703, doi:10.1104/pp.17.01583 [F abstract]. Reduced-secondary-growth genotypes had 39% (greenhouse) and 43% (field) smaller root cross-sectional area under low P, with 27% and 32% more root length and 68% and 80% more shoot mass. No absolute values in the abstract.
- OpenSimRoot bean (archive roots note): primary diameters are tap 0.9 mm, basal and hypocotyl-borne 0.64–0.72 mm. The secondary rate is described in §3.3 of the spec [F, archive].

### 3. Petiole share and stem share
- UFGA8501 (DSSAT; Gainesville FL 1985, 51.7 plants m⁻², 4 cultivars). SWAD includes petioles; LWAD is leaf blade. Treatment 1 at 85238 (about R6): stem 1658, leaf 865 kg ha⁻¹, giving a share of 0.66; at maturity stem 1108, giving −33% [F file]. Peak LAI 3.25–4.38.
- Lopes et al. 1983, Rev. Ceres 30(172):451–462 [F, via WebFetch of ojs.ceres.ufv.br]. Bean in pots at 100/62/30% light. Maximum leaf dry mass 6.6/5.6/4.6 g; maximum stem+petiole 4.9/4.6/4.3 g, so the stem+petiole share of the maxima is 0.43/0.45/0.48 [I arithmetic]. The stem maximum comes about 20 d after the leaf maximum.
- Petiole share of (blade+petiole) for bean: no source found. Most bean and soybean growth studies lump petioles with stems (Gómez & Frankow-Lindberg 2005; Kumudini et al. 2001; Santos et al. 2016). From memory, 20–30% for soybean at R5–R6 [K, to verify]. Scott et al. 1983 (J. Plant Nutr., Bragg soybean) weighed petioles separately: petioles peaked in September with the blades, and stems peaked in August [F extract]; the values are in their tables, which I did not read.

### 4. Bean stem and root NSC
- Waters, Breen, Mack & Graham 1980, J. Amer. Soc. Hort. Sci. 105(3):424–427 [F, Table 2 read]. Greenhouse, cv. Puebla 152; flowering 48 DAP, early pod fill 63, mid pod fill 70. Starch in mg g⁻¹ DM:
  - Lower stem 204 → 306 → 403.
  - Middle stem 683 → 628 → 364.
  - Upper stem 71 → 476 → 315.
  - Root 36 → 50 → 33.
  
  Soluble sugars were 40–160 mg g⁻¹ in stem and 42–86 in root. Middle-stem starch fell 42% in the 7 days before mid pod fill, equal to about 45% of the pod dry matter gained in that period (as extracted). Values above 600 mg g⁻¹ are implausibly high for whole stems; treat them as a ceiling, not typical.
- Peña-Valdivia & Ortega-Delgado 1991, J. Sci. Food Agric. 55, doi:10.1002/jsfa.2740550408 [F abstract plus extract]. Cv. Jamapa, greenhouse pots. Roots, stems and branches are temporary carbohydrate stores, and stem starch is hydrolysed for seed growth. Stem and branch free sugars were about 7 (units probably % DM) at 59 DAS and 1.6 at maturity. Root NSC peaked at 0.18 g per plant at 80 DAS. Pod walls held 11–18% starch.
- Remobilisation share to seed yield in bean: not quantified in field studies found. Waters gives the 45% short-window figure above.

### 5. Wheat stem WSC
- Davidson & Chevalier 1992, Crop Sci. 32:186–190, doi:10.2135/cropsci1992.0011183X003200010038x [F abstract]. Spring wheat Edwall and Waverly, Pullman WA. Peak WSC 250–380 mg g⁻¹ DM, 10–14 d after anthesis; below 50 mg g⁻¹ at maturity. Losses from peak were 859–1235 mg (irrigated) and 619–662 mg (non-irrigated); the unit basis is unclear, per stem or per sample.
- Ruuska et al. 2006, Funct. Plant Biol. 33:799–809, doi:10.1071/FP06062 [F]. WSC at anthesis: environment means 108–203 mg g⁻¹, genotype means 112–213 mg g⁻¹; H² = 0.90.
- Ehdaie et al. 2006, Crop Sci. 46, doi:10.2135/cropsci2006.01.0013 [F]. Main-stem WSC mobilized: peduncle 70–244 mg, penultimate internode 95–227 mg, lower internodes 175–450 mg. Lower internodes gave 51% of the stem-mobilized WSC.
- van Herwaarden et al. 1998, Aust. J. Agric. Res. 49:1083–1093, doi:10.1071/A97040 [F]. Stem and spike WSC could contribute 34–50% of yield in severely hayed-off crops.
- Contribution ranges from reviews: 10–20% irrigated, 40–60% under heat or drought (Thakur et al. 2019, Plant Sci. Today); 20–40% non-stressed, up to 70% stressed (Ram et al. 2018, citing Vignjevic 2015 and Rebetzke 2008) [F abstracts].

### 6. Herbaceous radial growth (g_max)
- No direct dendrometer rate for bean, soybean or tomato stems was extracted. LVDT tomato studies (Gallardo et al. 2006, Irrig. Sci.; De Swaef & Steppe 2010, Funct. Plant Biol.) report stem growth rate as an index but give no number in the abstracts.
- Sunflower: stem diameter at 1 m height at maturity 17.3–23.3 mm (Radu & Gurău 2023, Sci. Pap. Ser. A Agron. 66(1)), with flowering at 48–63 DAS [F]. That implies about 0.15 mm d⁻¹ radial on average [I].
- Bean: 3 mm to 5–8 mm over about 60 d gives 0.02–0.04 mm d⁻¹ [I].

### 7. Phototropism
- Yokotani-Tomita et al. 1999, Physiol. Plant. (8-epixanthatin): green sunflower hypocotyls, unilateral blue light. Bending began 20–60 min after onset and reached a maximum of 25° at 100 min [F extract].
- Orbović & Poff 1991, Plant Physiol.: etiolated Arabidopsis, maximum about 16° about 80 min after a blue pulse [F abstract].
- Britz & Carroll 1993, Plant Growth Regul.: etiolated pea, latency 0.25–0.5 h at 80 pmol m⁻² s⁻¹; maximum response about 25° (55° after red-light pretreatment) [F extract].
- Vinterhalter & Vinterhalter 2012, Arch. Biol. Sci.: light-grown potato explants reach 90° in 75–120 min under continuous blue light [F abstract].
- Cosgrove 1985, Photochem. Photobiol.: etiolated cucumber, lag 4.5 h [F abstract].
- Ballaré et al. 1992, Plant Physiol.: de-etiolated cucumber bends away from far-red reflected by neighbours, so phytochrome acts in canopies [F abstract]. This supports driving `v` with the light field, and possibly the R:FR.
- Brennan et al. 1976, Plant Physiol.: intact green mung bean (Vigna, then called Phaseolus aureus) under white unilateral light bends in the epicotyl, and the stem must be lit [F]. The extract did not give angles.
- Bean leaf pulvinus: Koller et al. 2001, Isr. J. Plant Sci. [F extract], see the takeaway table.

### 8. Pipe model / Huber value
- Mencuccini et al. 2019, New Phytol. (terminal woody branches) and Xu et al. 2021 give sapwood:leaf Huber values for woody plants. I did not read their medians this session; woody-branch values of about 1e-4 are from memory [K, to verify].
- No herbaceous whole-stem Huber value found for bean. The whole-stem values derived from diameters and leaf area are in the takeaway, 1.5e-4–3e-4 [I].

### 9. Exudation
- OpenSimRoot exudates.xml comment: exudation ≈ 0.12 g C per g root per day, about 2/3 of respiration; monocots 1/5–1/3 of bean [F, archive roots note].
- Warembourg et al. 2003, Plant Soil 256: 12 Mediterranean species, ¹⁴C pulse. Root-wash (soluble exudate) ¹⁴C as % of net assimilated: legumes 2.7, grasses 1.6, forbs 1.1. Rhizosphere respiration as % of below-ground ¹⁴C: legumes 51, grasses 46 [F extract].
- Nguyen 2003, Agronomie 23: about 17% of net fixed C is lost from roots, 12% as rhizosphere respiration and 5% as soil residues [F extract].
- Yan et al. 2026, Adv. Sci. (746 tracer observations): 17% of assimilated C goes below ground and rhizodeposition is one third of that. Legumes put more into rhizodeposition than into root biomass [F abstract].
- Nielsen, Eshel & Lynch 2001, J. Exp. Bot.: bean root respiration about 20% of daytime net assimilation at medium and high P, about 40% at low P [F extract].
- Minchin & Pate 1973, J. Exp. Bot. (pea): 42% of shoot net C goes to the supporting root and 32% to nodules over 9 d [F abstract].

## Inferences
- Petioles move the stem-share band down by about 0.07–0.10 [I]. Unless the sim reports petioles separately, the band should stay wide.
- `h_L` ≈ 3e-4 reproduces 5–8 mm at field densities; a lone pot plant with 0.2–0.3 m² of leaf would reach 8.5–10.5 mm. So for single_pot, the 5–10 mm upper end is reasonable even though the field data sit at 4–8 mm [I].
- If the OSR root radial rate is a radial rate, the tap root at R6 would be about 6–8 mm, above Burridge's indication. Check the OSR definition before trusting the 3–8 mm band [I].
- e = 0.67 against respiration would put exudation at about 13% of daytime assimilation at adequate P, the high end of the tracer literature (5–10%). A value of 0.3–0.5 fits better [I].
- Wheat storage 0.35 g g⁻¹ can be reached (Davidson & Chevalier, 38%), but typical anthesis values are 11–21%, peaking a little later. 0.25–0.30 is a safer default [I].

## Gaps
- Bean hypocotyl diameter time series (V3, R1, R6) and climbing-type diameters: no data found. A greenhouse growth-analysis paper with stem calipers would close this.
- Absolute bean tap root and basal root diameters: Strock 2018 and Burridge 2016 full tables (Plant Physiol. was robots-blocked).
- Petiole share of leaf mass in bean: none. Scott et al. 1983 (soybean) and Ojima & Fukui 1966 have separated petioles in tables I did not read.
- Field bean stem starch time course; and whether the Waters 1980 mg g⁻¹ values are per section DM (very high).
- A herbaceous dicot dendrometer rate in mm d⁻¹; bean stem phototropic kinetics.
- The Davidson & Chevalier per-stem basis.
