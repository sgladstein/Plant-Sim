# M3 research: bean genetics, breeding numbers, weed competition

Written 2026-11-05 by the research thread for M3 (WP28 genome, WP29 pollination, WP30 bench,
WP32 challenges). It fills the M3 gaps listed in `docs/handoff/research.md` ("Research the next
phase needs"). Tags: **[F]** source read (abstract counts when the number is in it), **[F2]**
read in a citing paper, **[K, to verify]** from knowledge, **[I]** inference.

Notes:
- `notes/genetics_map_colour.md`: linkage, locus positions, seed-colour rules, and the
  34-locus table in §6.
- `notes/breeding_quantitative.md`: heritabilities, genotypic variation, crossing, heterosis
  and mutation.
- `notes/weed_competition.md`: Cousens parameters, the critical period, a draft weed species
  and intercrop LER.

## 1. Genome map (WP28)

- **34 named loci on Pv01 to Pv11** (notes §6). For each: chromosome ([F] for 32), physical
  position on G19833 v2 where published, a game cM value, phenotype, dominance and source.
  The notes suggest a core set of 24 loci, which leaves 6 to 16 slots for anonymous QTL.
- **Published distances:**

  | Pair | Distance |
  | --- | --- |
  | Rk–B | 28 cM |
  | P–Phs (stand-in for the seed-size QTL) | 9 cM; other sources give 15.5 or about 32 |
  | C–Gy | 2.7 cM |
  | B–I | "very tightly linked" |
  | V–Rf-1 | 11 cM |
  | V–Am | 9 cM |
  | Bip–J (markers) | 27.7 cM |

  All other cM values are placements [I]. That includes fin–Ppd at 8 cM: Ppd (PHYA3) is at
  47.64 Mb [F], fin at about 44 to 45 Mb [K], and Kwak 2008's figure 1 was unreadable.
- **Mb to cM is not linear.** Crossovers concentrate in the distal arms. Use about 3 cM/Mb in
  distal arms and about 0.3 across pericentromeres [I]. The genome-wide mean is 2.13 cM/Mb,
  and chromosomes run about 47 to 147 cM (Blair 2018) [F].
- **Corrections to round 2:** the 2008 BIC gene list puts C on B10 and Fin on B6. Its own
  source, McClean 2002, has C on Pv08 and fin on Pv01. Use Pv08 and Pv01.
- **Open questions:**
  - Pod string (St): the sources disagree on which allele is dominant.
  - Z: mapped to Pv03 (PvTT2), which conflicts with an old 1 cM Fin–Z linkage.
  - Hr and Fib: no position found.

## 2. Seed-colour rule table (WP28)

Evaluate as a pipeline (notes §3) [F for the gene roles, I for the game order]:
1. **P gate:** pp gives white seed and white flower.
2. **T gate:** tt gives a partly coloured seed and a white flower. Z, Bip and J set the pattern
   (expansa, virgarcus, sellatus, anasazi, bipunctata, marginata). The coloured area grows by
   about 21% of the seed coat per +4 °C during seed fill [F], a nice tie to the sim's weather.
3. **Hue** from C, J, G, B, V and Rk, for example:
   - Black: P C J G B V.
   - Dark brown: P [C r] J G B v.
   - Yellow-brown: P [C r] J G b v.
   - Dark red kidney: P [cu r] J g B v rk^d.
   - Manteca yellow: P [C r] d j G b v^lae.
4. **Pattern** (mottle, stripe) from the C complex alleles.
5. **Shine** from Asp: asp asp is matte.

Sources: Parker et al. 2024 (Plant J 119:2765, doi:10.1111/tpj.16947) [F] and Bassett's primary
papers [F]. Bassett 2007 (Plant Breeding Reviews 28) was not reachable. The genotypes for
pinto, cranberry, pink, great northern and navy are [K]. The colour hints are approximations.

## 3. Breeding numbers (WP28 to WP30)

| Quantity | Value | Source |
| --- | --- | --- |
| h² (narrow sense), 100-seed weight | 0.57 to 0.86 | White 1994; Nienhuis & Singh 1988 [F] |
| h², days to flowering / maturity | 0.29 to 0.75 / 0.26 to 0.63 | Cerna & Beaver 1990; Msolla 2007 [F] |
| h², seeds per pod / pods per plant | about 0.57 / 0.2 to 0.5 | [F] |
| h², seed yield | 0.09 to 0.75, often about 0.2 | [F] |
| h², basal root growth angle | 0.60 to 0.67 (field) | Burridge 2016 [F] |
| Basal root angle range, RILs | −19° to +57° from horizontal | Liao 2004 [F] |
| Germplasm ranges (4,274 accessions) | leaf length 4.5 to 20.7 cm; 100-seed weight 3.5 to 96 g; seeds per pod 2.1 to 9.6; flowering 27 to 104 d | Rana 2015 [F] |
| Vcmax between two cultivars | +12% (gs +44 to 79%) | Egesa 2024 [F] |
| Hand-cross pod set (emasculated) | 17 to 23% greenhouse; 18% field; 70 to 90% with hooking | Carpentieri-Pípolo 2001; van Rheenen 1978; Usher & Redden 1986 [F] |
| Seeds per crossed pod | 2 to 3 | [F] |
| Stigma receptive | from 3 days before opening, best the day before | Inoue & Suzuki 1959 [F] |
| Seed at maximum dry weight / harvest-dry | 35 to 45 / about 58 days after flowering | van de Venter 1996 [F] |
| Green seed sown undried at 30 days | 90% normal seedlings (3% if dried first) | van de Venter 1996 [F] |
| Heterosis, yield, mid-parent | +26 to 124%; better-parent heterosis in 20 of 47 F1s, sharp F2 drop | Singh & Urrea 1995; Gutierrez & Singh 1985 [F] |
| Heterosis, seed size | at most 6% | Motto 1978 [F] |
| Spontaneous mutation, morphological loci | below 1e-6 per locus per gamete (barley); none for bean | Kahler 1984 [F] |

Game defaults [I] (per-trait table in the notes):
- `allele_sd = CV_G × default / √(loci)`. For example, vcmax25 with CV_G 0.10 over 2 loci
  gives about 6.3; seed mass uses CV_G 0.20.
- Dominance of about +0.2·allele_sd on leaf area, internode length and flowers per raceme,
  for mild F1 vigour.
- Add heritable traits for basal root angle (about 20° ± 10°, h² 0.6) and flowering thermal
  time (CV_G 0.08).

Crossing in the game [I]:
- Pod-set chance about 0.4 when emasculated.
- About 0.6 of the normal seed number per crossed pod.
- About 50% accidental self when emasculation is skipped.
- Green seed plantable from about 30 days after pollination.

**Founders.** `Species::sample_genotype` draws the two alleles at each locus independently
(`crates/ps-species/src/lib.rs:562`), so every founder is heterozygous and segregates on
selfing. Pure-line founders need one draw used for both alleles [F, code].

**Environmental variance.** Decision 0027's plant-to-plant variation should bring measured h²
down from 1. Check it with a parent–offspring regression against the targets above.

## 4. Weed competition and intercrops (WP32)

**Cousens model** (% loss = I·N / (1 + I·N/A)), bean [F]:

| Weed | I (% per plant) | A (%) |
| --- | --- | --- |
| Palmer amaranth | 209 per plant per m of row | 95 |
| Redroot pigweed (snap bean) | 10 to 32 per plant per m of row | 47 to 63 |
| Wild poinsettia, emerging with the bean / 12 days before it | 2.4 / 5.5 per plant per m² | 48 / 60 |
| Alexander grass | 4.6 to 9.1 per plant per m² | 91 to 155 (fit artefact) |

No bean I or A exists for lambsquarters, barnyardgrass or cocklebur.

**Emergence time matters more than density.** Weeds that emerge at bean V3 cause about half
the loss of weeds that emerge with the crop (common ragweed: 10 to 22% vs 4 to 9%) [F].

**Critical period (5% loss).** It runs from about V1 to V3 (15 to 20 days after emergence) to
about R1 to R2 (39 to 47 days). Uncontrolled weeds cost 31 to 94% of yield, with a North
American mean of 71% [F].

**Mechanism.** Light comes first. Weeds cut pods per m² and LAI; seed mass changes only under
severe competition. Nitrogen fertiliser increases the loss, from 8 to 9% up to 30 to 33%
(Bannayan 2012) [F]. The sim's shared light, N and water should produce this without special
rules.

**Intercrops.**
- Bean–maize LER is 0.87 to 1.99 depending on site; maize dominates.
- The meta-analysis mean is 1.22 across all intercrops (Yu 2015) [F].
- No LER exists for common bean with wheat, oat or barley. Faba bean with wheat gives 1.29 to
  1.65 [F].

**Challenge targets** [I], bean yield against weed-free:

| Challenge | Target |
| --- | --- |
| 4 to 8 tall broadleaf weeds/m², emerging with the bean, never weeded | −45 to −80% |
| 1 weed/m² | −10 to −35% |
| Same weeds emerging at V3 | at most half the loss of the with-crop case |
| Weeded at V2 and kept clean to R1 | ≥ 95% of weed-free |
| Weedy, high N | larger relative loss than weedy, no added N |
| Weedy plot at bean maturity | weed shoot mass 0.4 to 2.5 kg/m² |

### Recommendation: make the first weed a C3 broadleaf

The literature's worst bean weeds are the pigweeds (Amaranthus), and they are C4. The sim has
only the C3 FvCB solver. Options:

1. **C3 first (recommended):** common lambsquarters (*Chenopodium album*) or common ragweed
   (*Ambrosia artemisiifolia*). Both are tall, competitive and C3, so the weed is a species
   file plus a seed bank, with no new physiology. Ragweed has bean emergence-timing data
   (Chikoye 1995).
2. **Pigweed:** needs a C4 photosynthesis model (Collatz 1992 or von Caemmerer 2000) with
   its own validation. That is a WP-sized piece of work.

The draft pigweed parameters are in the notes in case option 2 is chosen later:
- SLA 13.5 m²/kg; RUE 1.74 g/MJ.
- Shoot elongation base 14.4 °C and optimum 27.9 °C.
- Height 0.9 to 2 m.
- 10,000 to 40,000 seeds per plant under competition.

For a grass weed, the WP23 wheat parameters can stand in for a C3 grass.

## 5. Gaps

- fin–Ppd distance: Kwak 2008 figure 1 was unreadable.
- Market-class genotypes beyond the ones read.
- St dominance; the Z position conflict.
- Single-plant h² in pots; germplasm-wide CV for bean Vcmax and SLA.
- Exact emasculation timing in hours.
- F1 to F2 yield decline as a number.
- Bean I and A for lambsquarters and grasses.
- The Cousens 1987 relative-emergence-time equation form is [K].
- No bean LER with a small-grain cereal.
- No field time series of weed height and LAI against bean.
