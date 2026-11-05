# Multi-plant competition validation for M2: bean density, bean-weed, bean intercrops, FSPM comparators

Tags: [F] = read this session (abstract or full text fetched); [K, to verify] = prior knowledge, not re-read this session; [I] = my inference.
Search budget note: Consensus was rate-limited for several queries; some items rest on abstracts only (said so where true).

## 1. Bean density response (LAI plateau, per-plant plasticity, yield-density form)

### Takeaway
The M2 exit target is well supported by one paper. Clavijo Michelangeli et al. 2019 tested six densities (5 to 35 plants/m²) and three growth habits. LAI rose from about 1.5 to about 2.5 to 2.8 and was flat from 20 plants/m² upward. Leaf area per plant fell from more than 0.3 m² to less than 0.1 m². Leaf N per unit leaf area did **not** change with density. What plateaued was leaf N per unit ground area, so the wording of the §8.3 row should be fixed. I found no fitted Shinozaki-Kira or Holliday a and b parameters for Phaseolus. They will have to be fitted from digitised data.

### Cited Findings
- [F] Clavijo Michelangeli JA, Ricaurte J, Sinclair TR, Rao IM, Beebe SE (2019). Influence of plant density and growth habit of common bean on leaf area development and N accumulation. *J. Crop Improvement*, doi:10.1080/15427528.2019.1644694. Open full text: [CORE PDF](https://core.ac.uk/download/224976763.pdf); [CIAT record](https://alliancebioversityciat.org/publications-data/influence-plant-density-and-growth-habit-common-bean-leaf-area-development-and-n). Details:
  - Sites. Palmira, Colombia (999 m, sown 13 Oct 2013, Mollisol, 4 reps, mean Tmax 31.1 °C). Darién (1570 m, sown 29 May 2014, Inceptisol, 3 reps, mean Tmax 25.5 °C).
  - Densities: 5, 10, 15, 20, 25 and 35 plants/m².
  - Six genotypes, two per habit:
    - Type I (determinate bush): CAL 96, ICA Quimbaya.
    - Type II (indeterminate upright): DOR 364, SER 118.
    - Type III (indeterminate prostrate): G 21212, Carioca.
  - Leaf area per plant: >0.3 m² at 5 plants/m² and <0.1 m² at the highest density. It falls about linearly at high density. Types II and III carried more leaf area per plant than Type I at low density.
  - LAI: about 1.5 at low density, about 2.5 to 2.8 at high density, and about constant at ≥20 plants/m². The paper gives no fitted LAI-versus-density equation in the text, only model-selection results (Table 3). The curve must be digitised from the figures.
  - Leaf N per unit leaf area: about 2.0 g N/m² at Darién and about 1.3 g N/m² at Palmira, stable across densities.
  - Leaf N per ground area: stayed above about 3 g N/m² at all densities at Darién. At Palmira it was about 2.11 g N/m² lower on average, and the low-density plots fell below 3 g N/m².
  - Leaf dry mass per ground area at high density (Types II and III): about 60 to 70 g/m² at Palmira and about 100 to 110 g/m² at Darién.
  - Conclusion: "Plant spacing of 20 plants m⁻² or more was sufficient to result in equal leaf area and N accumulation."
  - The paper gives no numeric yields. Fig. 6 plots yield against leaf N per ground area.
- [F] Kouam et al. 2020 compared 16, 36, 49 and 121 plants/m² (square spacings of 30, 20, 15 and 10 cm). Pods per plant, seeds per plant, biological yield and seed yield responded to density. Pod length, height, seeds per pod and 100-seed weight did not. The highest seed and biological yield came at 16 plants/m². — [Kouam 2020, Not. Bot. Horti Agrobot.](https://consensus.app/papers/details/7168c81a0d5456edabac731cd347e2f8/?utm_source=claude_desktop) (abstract only)
- [F] Musana et al. 2020 (Rwanda) compared 20, 25, 30 and 35 plants/m². Yield and biomass were highest at 25 plants/m² and yield lowest at 35. Pods per plant and 100-grain weight both fell with density. — [Musana 2020, AJFAND](https://consensus.app/papers/details/9cc6986123f55d04aaa0db96c512512d/?utm_source=claude_desktop) (abstract)
- [F] Masa et al. 2017 (Ethiopia) varied inter-row spacing (30 to 50 cm) and intra-row spacing (7 to 13 cm). Wider intra-row spacing lowered LAI, height, biomass and yield, and raised leaf area per plant, pods per plant, seeds per pod and 100-seed weight. The highest LAI was 3.41, and grain yield reached 2264 kg/ha. — [Masa 2017](https://consensus.app/papers/details/160079809c9b50fe85c6cab0968c81d0/?utm_source=claude_desktop) (abstract)
- [F] Njoka 2005 (Kenya): seed weight, pods per plant and seeds per pod fell as density rose, and plant mortality rose with density. — [Njoka 2005](https://consensus.app/papers/details/42c5b4bf152c55459886ee898f9a8f48/?utm_source=claude_desktop) (abstract)
- [F] Merga 2020 (Jimma) tested rows 30 to 60 cm. Grain yield followed a parabola over plant density for the determinate varieties, with the optimum at 40 cm rows. 100-seed weight rose with row spacing. — [Merga 2020, Heliyon](https://consensus.app/papers/details/4fc03a00ce325475a9f08a388a288659/?utm_source=claude_desktop) (abstract)
- [F] Bakure et al. 2025 compared intra-row spacings of 5, 10 and 15 cm. Spacing affected leaf area, branches per plant, shoot dry mass, pods per plant and grain yield. It did **not** affect LAI. The best yield was 2495 kg/ha at 5 cm. — [Bakure 2025, Ann. Appl. Biol.](https://consensus.app/papers/details/be01d1073c9f5cd181db61f51b0a357d/?utm_source=claude_desktop) (abstract). A flat LAI across a high-density range is consistent with a plateau [I].
- [K, to verify] Yield-density forms:
  - Shinozaki & Kira 1956 reciprocal yield law, 1/w = a + b·ρ. Total biomass per area Y = ρ/(a + bρ) approaches the asymptote 1/b. Here a⁻¹ is the mass of an isolated plant and b⁻¹ is the maximum yield per area.
  - Holliday 1960 generalisation, 1/w = a + bρ^θ, which can give a parabolic grain-yield curve.
  - Willey & Heath 1969, Adv. Agron. 21: 281–321, reviewed these forms. Total biomass is usually asymptotic, while seed yield is often parabolic. The bean data above (yield peaks at 16 to 25 plants/m², then falls) fit the parabolic case for seed yield [I].
- [F] Biomass at high density in the 2012 Iran trial: weed-free dry bean reached a peak of 1238.8 g/m² aboveground. — [Ghamari 2013](https://consensus.app/papers/details/2182bfe1a08e53e2a40abbe050158568/?utm_source=claude_desktop)

### Inferences
- [I] A suggested M2 validation test is a density sweep on a 3 × 3 m plot (5, 10, 15, 20, 25, 30, and the ~33 plants/m² that 30 plants on 0.9 m² give).
  - Pass if LAI at ≥20 plants/m² lies within ±10 to 15% of its mean.
  - Pass if LAI at 5 plants/m² is about 50 to 65% of the plateau (about 1.5 against 2.5 to 2.8).
  - Pass if leaf area per plant falls by a factor of 3 or more from 5 to 35 plants/m².
  - Pass if leaf N per leaf area stays about constant with density.
- [I] Under the §8.3 factor-of-two rule, "plateau at 20 plants/m²" passes for any onset between 10 and 40. A tighter test should fit 1/LA_plant = a + b·ρ to the sim output and compare it with a fit to the digitised Clavijo Fig. 2 or 3.
- [I] The §8.3 row says "leaf area and leaf nitrogen per area plateau". It should read "leaf area per ground area and leaf N per ground area plateau at ≥20 plants/m²; leaf N per leaf area is density-invariant".
- [I] The direction of per-plant plasticity is consistent across studies. As density rises, pods per plant, branches per plant and leaf area per plant fall strongly. Seeds per pod and seed mass change little or fall slightly. Height rises slightly.
- [I] Note on plot size: 30 plants on a 1 × 1 m plot is 30 plants/m², already on the plateau. To test the rising part of the curve the sim needs a 2 × 2 or 3 × 3 m plot at 5 to 10 plants/m². Edge effects will be large in a 1 × 1 m plot unless the light grid is periodic or the plot has a guard row.

### Gaps
- No published Shinozaki-Kira or Holliday a and b values for Phaseolus vulgaris were found this session. Digitising Clavijo 2019 (six densities, six genotypes, two sites) is the best route.
- The Clavijo paper reports no numeric yield values, so yield per area against density must come from figures or from other trials (Kouam, Musana).
- No pole-bean (Type IV) density data were found. Clavijo covers Types I to III only.

## 2. Size hierarchy and symmetric versus asymmetric competition

### Takeaway
Theory is clear, but bean-specific numbers were not found. Light is the pre-emptable resource that drives size-asymmetric competition. Nutrient competition is usually about size-symmetric, and water is largely unstudied. Plasticity reduces asymmetry. Size inequality (Gini or CV) rises with density under asymmetric competition.

### Cited Findings
- [F] Schwinning S, Weiner J (1998). Mechanisms determining the degree of size asymmetry in competition among plants. *Oecologia* 113: 447–455. [doi:10.1007/s004420050397](https://link.springer.com/doi/10.1007/s004420050397). Key points:
  - Asymmetry needs a pre-emptable resource, and light is the main case because it is directional.
  - Competition for mineral nutrients is often size-symmetric, with resources divided roughly in proportion to size.
  - Almost nothing is known for water.
  - Morphological and physiological plasticity reduces asymmetry.
  - The paper proposes an allometric framework.
- [F] Metsaranta & Lieffers 2008, *Ann. Bot.* 101: 561–571 (jack pine). Size Gini was nearly always below 0.5. Density had a significant positive effect on size Gini (P < 0.01) at all four sites. The Gini coefficient runs from 0 to 1. The Lorenz asymmetry coefficient S > 1 means a few large plants drive the inequality. — [PMC2710195](https://pmc.ncbi.nlm.nih.gov/articles/PMC2710195)
- [K, to verify] Weiner J (1990). Asymmetric competition in plant populations. *TREE* 5: 360–364.
- [K, to verify] Weiner J, Thomas SC (1986). Size variability and competition in plant populations. *Oikos* 47: 211–222. Most studies found size inequality rising with density or over time in crowded stands, but it did not rise in uncrowded stands.
- [K, to verify] Two papers found this session via search results only (titles seen, not read):
  - Weiner, Stoll et al. 2001, "The effects of density, spatial pattern, and competitive symmetry on size variation in simulated plant populations", *Am. Nat.* [catalogue](https://boris.unibe.ch/81052). Size variation rose with density only when competition was asymmetric, and was highest in random rather than uniform patterns.
  - "Effects of density and extinction coefficient on size variability in plant populations" [AGRIS](https://agris.fao.org/search/es/records/65df68a84c5aef494fe28229). Its title suggests that a higher canopy extinction coefficient raises size variability.
- [F] Zhang et al. 2014, *Ann. Bot.* (three sisters, DNA-based root ID): maize foraged shallower, while common bean explored the vertical profile more evenly. Lateral root branching density was higher for all species in polyculture than in monoculture. This is evidence of below-ground niche complementarity rather than strict segregation. — [Zhang 2014](https://consensus.app/papers/details/f918491bb56950cea5e9ac99a6be2454/?utm_source=claude_desktop)

### Inferences
- [I] Suggested emergent checks for the sim:
  - Shoot-mass Gini in a 30-plant bean plot should rise over time and with density. Example: low density, Gini of about 0.1 to 0.2. Crowded, Gini of about 0.3 to 0.5. These are typical ranges for even-aged herbaceous monocultures [K, to verify], not bean-specific.
  - Switching the light model off (with only roots competing) should flatten the rise in Gini. That would show the merged light grid produces asymmetry.
  - Root uptake from shared 5 cm cells should split in proportion to root length in a cell, which is size-symmetric. Do not use a pre-emption rule.
- [I] Uniform grid sowing will give lower inequality than random sowing. Tests should fix the pattern and seed.

### Gaps
- No per-plant mass distribution (Gini or CV) for Phaseolus stands was found. Candidates to search later are the Weiner lab's datasets and soybean or field-bean density trials.
- No bean-specific study of root overlap or segregation between conspecific neighbours was found.

## 3. Bean-weed competition (critical period, Cousens model)

### Takeaway
The critical period for weed control in dry bean is consistent across studies. It starts at about the V2 to second-trifoliolate stage (2 to 3 weeks after planting) and ends at about first flower (5 to 6 weeks after planting). Season-long weed interference costs 31 to 94% of yield in North America (mean 71%). The Cousens rectangular hyperbola is the standard yield-loss model. The bean-specific I and A values were not extracted this session.

### Cited Findings
- [F] Woolley BL, Michaels TE, Hall MR, Swanton CJ (1993). The critical period of weed control in white bean. *Weed Sci.* (Ontario, 1986–87). With a 3% yield-loss threshold, the critical period ran from the second-trifoliolate stage to first flower for all cultivars and years except one. Pods per plant fell with longer interference. The start of the critical period coincided with the start of rapid weed biomass increase. — [Woolley 1993](https://consensus.app/papers/details/a1efbef1f95c5b9aa13a608d3af4e190/?utm_source=claude_desktop)
- [F] Burnside et al. 1998, *Weed Sci.* (Minnesota): the critical period was 3 to 5 or 6 weeks after planting. Weed removal had little effect on stand or 100-seed weight. The main weeds were wild mustard, foxtails, redroot pigweed, common ragweed, wild buckwheat, hairy nightshade and lambsquarters. — [Burnside 1998](https://consensus.app/papers/details/779530a50ebb534abe2559b41f240798/?utm_source=claude_desktop)
- [F] Beiermann et al. 2021, *Weed Technol.* (Nebraska), using a 5% yield-loss threshold:
  - Without a preemergence herbicide, the critical timing of weed removal ran from V1 (16 days after emergence) to R1 (39 days).
  - With a preemergence herbicide it started at R2 (47 days).
  - Pods per plant fell 73% without the preemergence herbicide and 26% with it (2018).
  - [Beiermann 2021](https://consensus.app/papers/details/8fbf2107033d5f24bd68e0f548c1adb5/?utm_source=claude_desktop)
- [F] Odero et al. 2018, *HortScience* (snap bean, Florida). The critical period was 1.2 to 8.4, 1.7 to 6.7 and 2.2 to 5.2 weeks after emergence at 2.5%, 5% and 10% acceptable yield loss respectively. — [Odero 2018](https://consensus.app/papers/details/89f3200461855c4fa2e607a2f2cfff15/?utm_source=claude_desktop)
- [F] Ghamari et al. 2012 (Iran; Chenopodium album, Amaranthus retroflexus):
  - Season-long weeds cut yield by 60%.
  - Pods per plant were 5.01 in the season-long weedy plots.
  - Weed-free plants were 20 cm taller.
  - The critical period was 20 to 68 days after emergence (5% loss) and 23 to 55 days (10% loss).
  - [Ghamari 2012](https://consensus.app/papers/details/dee4ad39daac51bba63f7ae1678d7b57/?utm_source=claude_desktop). Crop biomass was 676.6 g/m² weedy against 1238.8 g/m² weed-free: [Ghamari 2013](https://consensus.app/papers/details/2182bfe1a08e53e2a40abbe050158568/?utm_source=claude_desktop)
- [F] Blackshaw 1991, *Weed Sci.* (hairy nightshade): 2 plants per m of row cut yield by 13%, and 100 per m by 77%. Interference in the first 3 weeks after emergence already cut yield. Up to 9 weeks weed-free was needed. Seed output was more than 45,000 seeds per plant at low density, peaking at more than 300,000 seeds per m of row at about 30 plants per m. — [Blackshaw 1991](https://consensus.app/papers/details/74aac443d23454cba4caeda690aef630/?utm_source=claude_desktop)
- [F] Soltani et al. 2018, *Weed Technol.* (WSSA, 2007–2016). Potential yield loss with no weed control was 31 to 94% by region (Idaho 50, Michigan 31, Montana 36, Nebraska 59, North Dakota 94, South Dakota 31, Wyoming 71, Ontario 56, Manitoba 71%). The North American mean was 71.4%. — [Soltani 2018](https://consensus.app/papers/details/7f2ae89dfc885ae88ff12114a04702eb/?utm_source=claude_desktop)
- [F] Bannayan et al. 2012, *Field Crops Res.* (redroot pigweed with N rate). Yield loss in the semi-erect habit was 8 to 9% at 0 kg N/ha and 30 to 33% at the highest N rate. In the erect habit it was 13 to 17% and 33 to 40%. N fertilizer increases weed damage, and the semi-erect habit tolerates weeds better. — [Bannayan 2012](https://consensus.app/papers/details/fc47c0f015e05debaa82fae2395c3429/?utm_source=claude_desktop)
- [F] Tejada et al. 2025 (arid Peru; two bean densities, 12 and 8 plants per m of row). At the higher density the critical period was 17 days shorter and the period before interference 4 days longer. Yields at maximum weed dry mass were 800 kg/ha at the higher density against 137.6 kg/ha at the lower. Denser crops compete better. — [Tejada 2025](https://consensus.app/papers/details/d352b59f20ab5222a29619940aa78899/?utm_source=claude_desktop)
- [F] Kalsing et al. 2013 (Alexander grass, Urochloa plantaginea, in common bean): a nonlinear hyperbolic model fit yield loss well against weed density, fresh mass or dry mass. The critical density was 0.4 to 0.7% loss-equivalent. — [Kalsing 2013](https://consensus.app/papers/details/8592894dcd6d5b66b9a9a2ad5112a581/?utm_source=claude_desktop). In Biachessi 2022 the 260,000 plants/ha bean density was most competitive against Alexander grass at 0 to 104 plants/m², using the rectangular hyperbola. — [Biachessi 2022](https://consensus.app/papers/details/910ce0b040735cfb963216c19d235e3b/?utm_source=claude_desktop)
- [F] Cousens R (1985). A simple model relating yield loss to weed density. *Ann. Appl. Biol.* The rectangular hyperbola has two parameters and gave the best fit among 18 models across 22 data sets. — [Cousens 1985](https://consensus.app/papers/details/1ad378a3ddbc53329caf98c5146e5b64/?utm_source=claude_desktop)
  - [K, to verify] Its form is YL = I·d / (1 + I·d/A). I is the % loss per weed per m² as d → 0, and A is the asymptotic maximum % loss.
  - The R implementation is `statforbiology::SScousens85` ([rdrr](https://rdrr.io/cran/statforbiology/man/SScousens85.html)).
  - Comparator from soybean (not bean), Bensch 2003: at 8 plants per m of row, maximum loss was 78.7% for Palmer amaranth, 56.2% for waterhemp and 38.0% for redroot pigweed. Pigweed that emerged later (crop cotyledon stage) caused no significant loss. — [Bensch 2003](https://consensus.app/papers/details/b58ed26ebc48539b83279b94e54d2878/?utm_source=claude_desktop)
- [K, to verify] Cousens et al. 1987 extended the model to relative emergence time, I(T) = I·e^(−cT). Knezevic et al. 2002 (*Weed Sci.* 50: 773–786) standardised the critical-period method: log-logistic fits for weedy periods, Gompertz fits for weed-free periods.

### Inferences
- [I] A "weed challenge" scoring target: an uncontrolled grass or broadleaf weed sown with the bean should cut bean yield by 30 to 70%. A weed that emerges about 2 or more weeks after the bean should cause little loss. Weeding between about V2 and R1 (about 15 to 40 days after emergence) should recover more than 90% of weed-free yield.
- [I] The emergent check is the critical-period shape. Run weedy-until-day-X and weed-free-until-day-X series in the sim, fit log-logistic and Gompertz curves, and compare the start and end against the second trifoliolate stage and first flower.
- [I] Pod number should be the yield component most hit by weed shading, with seed mass roughly unchanged. This is the same pattern as the drought target.

### Gaps
- No bean-specific Cousens I and A values were read this session. Candidates are Blackshaw 1991 (nightshade; per-m-of-row densities in its tables), Bannayan 2012, Kalsing 2013, and pigweed-in-bean papers (e.g. Malik et al. 1993). Full texts are needed.

## 4. Intercropping: bean-maize LER and shade on bean

### Takeaway
Maize-bean intercrops reliably over-yield. A milpa synthesis gives LER of about 1.34. A 2026 review reports 1.16 to 2.78. A relay trial in Egypt reached 1.64 to 1.99. In mixtures the bean is the suppressed partner: maize is barely affected, while bean yield falls 3 to 30% (relay) or its LAI falls by up to 80% in an FSPM.

### Cited Findings
- [F] Mota-Cruz et al. 2025, *Agriculture* (milpa review): milpa LER is 1.34, comparable to global intercrops. Few quantitative evaluations exist. — [Mota-Cruz 2025](https://consensus.app/papers/details/ac461c47be9d57d5843201d620fa7c6d/?utm_source=claude_desktop)
- [F] Chiorese et al. 2026, *Front. Sustain. Food Syst.* (narrative review): milpa LER values range from 1.16 to 2.78. — [Chiorese 2026](https://consensus.app/papers/details/e591d6b0a8fd5b6cb0c181a311aa9e8f/?utm_source=claude_desktop)
- [F] El-Mehy et al. 2023, *Front. Sustain. Food Syst.* 7, doi:10.3389/fsufs.2023.1052392 (Egypt, relay maize with bean cv. Nebraska, additive 100:100 design). Results:
  - Densities: bean 166,667 plants/ha, maize 50,000 plants/ha.
  - Mean total LER was 1.64 to 1.68 at 60 days of co-growth, 1.82 to 1.84 at 45 days and 1.93 to 1.95 at 30 days. The maximum was 1.99.
  - Partial bean LER was 0.72, 0.77 and 0.98.
  - Sole bean yielded 1900 and 2005.5 kg/ha. Intercropped bean yielded 1370 to 1952 kg/ha.
  - The paper did not measure PAR.
  - [Frontiers full text](https://frontiersin.org/articles/10.3389/fsufs.2023.1052392/full)
- [F] Ebel et al. 2017, *Terra Latinoamericana* (organic milpa, central Mexico). Maize-bean gave the highest total production: maize 7.9 Mg/ha and bean 1.9 Mg/ha. Per-plant maize yield was 68.3 g, 1.2 times monoculture. Per-plant bean yield was 27.5 g, equal to monoculture. Adding squash reduced per-plant maize and bean yield. — [Ebel 2017](https://consensus.app/papers/details/91a0693908e25f59b569acce8a4891ae/?utm_source=claude_desktop)
- [F] Zhang et al. 2014, *Ann. Bot.*: maize/bean and maize/bean/squash out-yielded monocultures on a land-equivalent basis. The gain came mainly from a complementarity effect, not a selection effect. — [Zhang 2014](https://consensus.app/papers/details/f918491bb56950cea5e9ac99a6be2454/?utm_source=claude_desktop)
- [F] Rutjens RJL, Evers JB, Band LR, Jones MD, Owen MR (2025). A functional–structural plant model for climbing bean: simulating light capture in maize/bean polycultures. *in silico Plants* 7(2): diaf011, doi:10.1093/insilicoplants/diaf011, CC BY 4.0. [PDF](https://edepot.wur.nl/699776). Code: https://github.com/pmxrr3/maizebeanpolyculture_2023 (no licence stated in the article).
  - Platform: GroIMP/XL. Light is traced stochastically with 72 diffuse and 24 direct sources, at a diffuse fraction of 0.8.
  - Bean parameter ranges:
    - Phyllochron 50 to 75 °C·day; base temperature 8 to 14 °C.
    - Leaf angle 63 to 117°; LMA 2.25 to 6.75 mg/cm².
    - Lmax 0.05 to 0.15 m; minimum internode 0.03 m.
    - Twining helix inclination 35 to 75°, with ±10° noise per internode.
    - Pods appear after 35 days.
  - Layout: 0.5 m between rows and 0.2 m within rows, i.e. about 10 plants/m² of each species. A 10 × 10 grid is cloned 10 times in each direction.
  - Results under Mexican conditions:
    - Bean peak LAI was 80% below monoculture, and maize was barely affected.
    - Combined peak LAI reached up to 350% of the maize peak.
    - Total peak biomass rose by up to 60%.
  - Under Dutch conditions bean biomass was 80% lower than under Mexican conditions.
  - Bean sown later than maize did worst at a delay of about 21 to 30 days.
  - The authors state that no quantitative validation was possible for lack of data. Comparisons were qualitative only (Francis 1982; Clark & Francis 1985; Pleasant & Burt 2010; Nurk 2017).
- [K, to verify] Global intercrop meta-analyses report a mean LER of about 1.2 to 1.3: Yu et al. 2015, *Field Crops Res.* 184: 133–144, mean LER 1.22; Martin-Guay et al. 2018, *Sci. Total Environ.*, LER about 1.3.

### Inferences
- [I] A pole bean on maize in M2/M3 should show four things. Maize yield per plant within about 0 to 10% of sole. Bean yield per plant clearly below sole, with a partial LER of about 0.5 to 0.8 when sown together and lower if sown 3 to 4 weeks late. Total LER of about 1.1 to 1.5. Bean leaves with higher specific leaf area and longer internodes under the maize canopy.
- [I] Rutjens 2025 is the closest published architecture to WP22: GroIMP, a twining helix, and a collision check. It is the natural sanity comparator. It lacks validation itself, so "agrees with Rutjens" is a plausibility check, not validation.

### Gaps
- No direct PAR transmission to a bean understory under maize (fraction of above-canopy light) was found this session.
- No bean-grass intercrop LER (relevant to the WP23 grass) was found. Weed-grass data (Alexander grass, foxtail) in §3 are the nearest proxy.

## 5. FSPM competition comparators and light interception (k)

### Takeaway
FSPM mixture studies validate against plot-level LAI, light interception and biomass, and sometimes per-plant architecture. They rarely validate against per-plant mass distributions. The wheat-maize FSPM of Zhu et al. attributes 64% of a 23% gain in mixture light capture to plasticity. A measured bean k was not confirmed this session. Comparable legumes: mungbean k = 0.68 and soybean k = 0.48 (by fAPAR method).

### Cited Findings
- [F] Zhu J (2015), PhD thesis, *Plant Plasticity in Intercropping: Mechanisms and Consequences*, Wageningen ([edepot](https://edepot.wur.nl/324149)). Simulated light capture in wheat-maize relay intercrop with plasticity was 23% above the sole-crop expectation. Of that, 36% came from configuration and 64% from plasticity. Layout: 6 wheat rows at 12.5 cm alternating with 2 maize rows at 75 cm. Densities were about 250 plants/m² wheat and about 10 plants/m² maize, with a 43.75 cm wheat-maize gap. Journal version: Zhu J, van der Werf W, Anten NPR, Vos J, Evers JB (2015), "The contribution of phenotypic plasticity to complementary light capture in plant mixtures", *New Phytol.* 207: 1213–1222, doi:10.1111/nph.13416 [K, to verify volume and pages] ([zendy](https://zendy.io/title/10.1111/nph.13416)).
- [F] Louarn, Barillot et al. 2020, *Ann. Bot.* 126: 671–685 (Virtual Grassland FSPM, legume-grass mixtures). It was earlier assessed (Faverjon et al. 2019) with "a good capacity to predict forage yield", N fixation and the dynamics of species proportions. Barillot et al. 2014 (wheat-pea) found that the taller species dominates light partitioning. — [PMC7489071](https://pmc.ncbi.nlm.nih.gov/articles/PMC7489071)
- [F, title only] Evers JB et al. (2019). Understanding and optimizing species mixtures using functional–structural plant modelling. *J. Exp. Bot.* 70: 2381–2388, doi:10.1093/jxb/ery288 ([zendy](https://zendy.io/title/10.1093/jxb/ery288)). Content not read this session.
- [F] Geetika et al. 2022, *Field Crops Res.* (mungbean, Vigna, canopy density treatments): k was stable at 0.68 and RUE was 1.3 g/MJ. LAI was the key driver of differences in interception. — [Geetika 2022](https://consensus.app/papers/details/977415896132540eb96b454e7cdb5dee/?utm_source=claude_desktop)
- [F] Kukal & Irmak 2020, *Agric. For. Meteorol.* (high-frequency PAR). k was 0.48 for soybean and sorghum and 0.36 for winter wheat. LUE on absorbed PAR was 3.1 g/MJ for soybean and 5.3 for maize. Maximum fAPAR was 88 to 96%. — [Kukal 2020](https://consensus.app/papers/details/56e05e65e73a50478ca93efa6e089159/?utm_source=claude_desktop)
- [F] Tesfaye et al. 2006, *Eur. J. Agron.* (bean, chickpea, cowpea, Ethiopia). Dry matter correlated with intercepted PAR fraction (r = 0.83 to 0.96), and that fraction correlated with LAI (r = 0.78 to 0.98). Water deficit at mid-season cut bean RUE by 39%. Bean k values were not shown in the abstract. — [Tesfaye 2006](https://consensus.app/papers/details/3ae91826820e59eb8eaec98cbfee41fe/?utm_source=claude_desktop)
- [F] Teixeira et al. 2015 (bean cultivars BRS Radiante and Pérola): the cultivars differed in LAI and architecture but not in k. The value was not in the abstract. — [Teixeira 2015](https://consensus.app/papers/details/9f6633338d855c5bba8e0ba9f611346c/?utm_source=claude_desktop)
- [F] Maize density raised LAI by 28 to 151% and lowered k by 20 to 50%: k is not constant across density. — [Duan 2024](https://consensus.app/papers/details/6908131963b45a52a96ede82ca1bb60e/?utm_source=claude_desktop)
- [K, to verify] Values commonly used for bean k are about 0.6 to 0.8 in DSSAT CROPGRO-Drybean, with the drybean species file KCAN about 0.67. Validate this against the CROPGRO BNGRO047.SPE file.

### Inferences
- [I] Validation hierarchy for M2 competition, taken from how FSPMs validate:
  1. Plot fraction of intercepted PAR against LAI should follow 1 − exp(−k·LAI), with k about 0.5 to 0.8. The Beer's-law fixture already exists in §8.1.
  2. LAI against density should plateau (Clavijo).
  3. Per-plant leaf area and pod number should decline with density.
  4. Mixtures: the taller species should take a disproportionate share of light (Barillot), and LER should exceed 1.
  5. Per-plant mass Gini should rise with density, only when light competition is on.
- [I] A fitted canopy k from the sim should fall slightly as density rises (Duan 2024 direction). This is emergent from clumping in a voxel grid.

### Gaps
- No measured bean k was confirmed from a full text this session. Tesfaye 2006 and Teixeira 2015 full texts likely hold values.
- The Evers 2019 review was not read, so its table of FSPM mixture studies and their validation methods was not extracted.

## 6. Open per-plant datasets and pot-size effect

### Takeaway
No open per-plant competition dataset for bean was found. Rutjens 2025 publishes code and spreadsheets (article CC BY 4.0; no code licence stated). Pot size strongly matters: plant biomass rises 43% per doubling of pot volume, and the biomass-to-volume ratio should be kept at or below 1 to 2 g/L. A bean grown to 21.8 g in a 20 cm pot likely exceeds this, so M0/M1 pot runs should not be compared one-for-one with field per-plant targets.

### Cited Findings
- [F] Poorter H, Bühler J, van Dusschoten D, Climent J, Postma JA (2012). Pot size matters: a meta-analysis of the effects of rooting volume on plant growth. *Funct. Plant Biol.* 39: 839–850, doi:10.1071/FP12049. [PDF](https://www.esalq.usp.br/lepse/imgs/conteudo_thumb/Pot-size-matters-a-meta-analysis-of-the-effects-of-rooting.pdf). Results:
  - Scope: 65 studies, 69 species, pots from 5 mL to 1700 L.
  - Doubling pot volume raised biomass by 43% on average (P < 0.001), with no difference between herbaceous and woody plants. Plants in 2 L pots were more than 3 times the size of plants in 0.2 L pots.
  - Effects per doubling of pot volume: RGR rose about 5%. ULR and photosynthesis rose, while SLA and LMF did not change. RMF fell about 4%.
  - Main mechanism: reduced photosynthesis per leaf area, not allocation. Root confinement or sink feedback is proposed.
  - Root distribution in a 1.3 L pot (MRI): about 50% of roots lay within the outer 4 mm, about 20% of the volume.
  - Recommendation: keep biomass to volume below 1 g/L and avoid more than 2 g/L. Effects saturate below 2 g/L, with the inflection between 0.2 and 2 g/L. About 65% of experiments exceed 1 g/L. Field crops during vegetative growth run about 1 to 1.5 g/L.
- [F] Rutjens 2025 code: https://github.com/pmxrr3/maizebeanpolyculture_2023, with spreadsheet data in the supplement ([PDF](https://edepot.wur.nl/699776)).

### Inferences
- [I] A 20 cm pot holds about 4 to 6 L depending on shape. The §8.2 harvest target of 21.8 g per plant gives about 4 to 5 g/L. That is above Poorter's 2 g/L threshold, so a real bean in that pot would likely be pot-limited.
  - If the sim does not model pot limitation, matching field per-plant targets in a 20 cm pot is fine for game purposes. It is not what a real 20 cm pot would give.
  - If the sim has 5 cm soil cells with a hard wall, root crowding at the wall should emerge, as in Poorter's 50% of roots in the outer 4 mm.
  - A pot-volume sweep (for example 2, 4 and 8 L) is a good emergent response test. Biomass should rise by roughly 20 to 45% per doubling once the biomass-to-volume ratio exceeds about 1 to 2 g/L, and should show no effect below that.
- [I] Pot against plot matters for M2. Moving from one plant in a pot to 30 plants in a shared 5 cm soil block changes rooting volume per plant by orders of magnitude. Reported differences between pot and plot should be checked against the density effect, not attributed to competition alone.

### Gaps
- No Dryad, Zenodo or Dataverse dataset with per-plant bean masses from a density or competition trial was found. A targeted search of Dryad and Zenodo for "Phaseolus density individual plant biomass" remains to be done.
- The licence for the Rutjens GitHub repository is unknown (check for a LICENSE file).
