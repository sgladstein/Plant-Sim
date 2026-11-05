# WP28/WP32 research: bean linkage distances, colour-gene positions, seed-colour rule table

Tags: [F] I read the source this session (full text through a fetch extract, abstract or metadata); [K, to verify] from memory; [I] inference for game design.
This builds on docs/archive/research/notes-m2-m3/bean_genetics.md §1 and §3, and on m2-m3-bean-science.md, "Bean genetics for the breeding bench".

## Takeaway

- **fin–Ppd: both on Pv01, close, but no cM value read.** Kwak et al. 2008 put fin and Ppd on B1 (= Pv01) but give no cM value in the text; the value is only in Fig. 1. Campa et al. (BIC) say Ppd is "close to" fin, again with no number. PHYA3, the Ppd candidate, sits at 47.64 Mb on Pv01 [F]. fin (PvTFL1y) sits on the lower arm of Pv01 at about 76.9 cM on the González 2016 consensus map [F]. **Game value: 8 cM (range 5–15)** [I].
- **P–seed size on Pv07.** No paper puts a seed-weight QTL itself at a cM distance from P. The classic proxy is phaseolin (Phs). Phs is linked to P at 9 cM on D7 in two maps (Nodari 1992; Koinange 1996, as cited by Park et al. 2000) [F]. Vallejos & Chase 1991 give about 15.5 cM, or about 32 cM from their gene order [F; the two values conflict]. **Game value: seed-size QTL 10 cM from P** [I].
- **Other well-sourced pairs:**
  - B–I: "very tightly linked", Pv02 end [F]
  - Rk–B: 28 cM [F]
  - Fin–"Z": 1 cM (Bassett 1997c) [F]. This one conflicts: Z (PvTT2) is now mapped to Pv03 [F].
  - Bip–J: linked on Pv10 [F]
  - V–Rf-1: 11 cM; V–Am: 9 cM [F]
  - C–Gy: 2.7 cM, inside the "complex C" on Pv08 [F]
- **Chromosome positions confirmed from primary sources:** P Pv07, C Pv08, J Pv10, G Pv04, B Pv02, V Pv06, T Pv09, Z Pv03, Bip Pv10, Asp Pv07, fin Pv01, Ppd Pv01, I Pv02. Rk sits on Pv02 by linkage to B.
- **Gene list errors.** Bassett's 2008 gene list (Porch update) has two transcription errors. It puts C on B10 and Fin on B6. McClean et al. 2002, the very source it cites, gives C on B8 and fin on B1. Use the primary sources.
- **Seed-colour rule table.** It can be built as a gated pipeline:
  1. P gate (white)
  2. T gate (partly coloured patterns, using Z/Bip/J/Fib)
  3. C complex (pattern and the R red allele)
  4. Base hue from J × G × B × V × Rk
  5. Asp shine

  Five genotype → colour pairs are [F] from Bassett and Beninger (below), plus the partly coloured pattern table. The market-class genotypes for pinto, cranberry, great northern and navy are mostly [K, to verify].

## 1. Linkage table

Chromosome names: Freyre et al. 1998 linkage group Bn equals chromosome Pv0n under the modern numbering. B1 is Pv01, B7 is Pv07, and so on [K, to verify; standard convention, consistent with every pairing seen this session].

| Locus pair | Chromosome | Distance | Source | Tag |
|---|---|---|---|---|
| fin – Ppd | Pv01 (B1) | Same linkage group, "close"; no cM in text (Kwak Fig. 1 has Kosambi distances, not read) | Kwak, Velasco & Gepts 2008, J. Hered. 99:283–291, doi:10.1093/jhered/esn005 [K, DOI to verify]; https://scispace.com/pdf/mapping-homologous-sequences-for-determinacy-and-photoperiod-2xvbkclp2e.pdf | [F] |
| fin – Ppd | Pv01 | Ppd "close to" Fin/fin; no number | Campa, Pérez, Giraldez & Ferreira, BIC Annu. Rep. 48:36–37 (2005) [year/volume from the file name, K]; https://www.unioviedo.es/esr/rgiraldez/privado/articulos/BIC48-36.pdf | [F] |
| fin – Ppd, physical | Pv01 | PHYA3 (Phvul.001G221100) at 47.64 Mb. PvTFL1y (Phvul.001G189200) is about 32 gene models proximal, roughly 44–45 Mb [K, to verify], so about 3 Mb apart | González et al. 2021, Front. Plant Sci. 11:599462, doi:10.3389/fpls.2020.599462 (PHYA3 position) | [F] PHYA3; [K] fin Mb |
| fin – Ppd, game value | Pv01 | **8 cM (5–15)**: about 3 Mb × 2–3 cM/Mb (Pv01 mean ≈ 147 cM / 52 Mb ≈ 2.8 cM/Mb; Blair 2018 genome mean 2.13 cM/Mb) | Derived | [I] |
| fin – seed protein Sp9/Sp10 | Pv01 | 4.0 cM (r = 0.039, LOD 18.3) | Campa et al. BIC 48:36 | [F] |
| fin – PvTFL1y | Pv01 | Co-segregate (0 cM) | Kwak et al. 2008; Repinski, Kwak & Gepts 2012, TAG 124:1539–1547, doi:10.1007/s00122-012-1808-8 | [F] |
| fin – Z | "B6" in the gene list (an error; fin is on B1) | 1 cM (Bassett 1997c) | BIC gene list, Porch 2008 update, https://www.bic.uprm.edu/wp-content/uploads/2018/04/BeanGenesList.pdf | [F] text; the conflict is noted in Gaps |
| fin – sellatus WD40 candidate (Phvul.001G192200) | Pv01 | 245 kb from fin | Parker et al. 2023 bioRxiv 10.1101/2023.09.23.559151 (published as Parker et al. 2024 Plant J 119:2765, doi:10.1111/tpj.16947) | [F] |
| fin – No | Pv01 | 31 cM | BIC gene list 2008 | [F] |
| fin – flowering/pod QTL (FT-1, PGT-1, PST-1) | Pv01 | FIN flanked by BMD045 (73.2 cM) and BMC224 (84.9 cM); FIN at 76.9 cM on the consensus map | González et al. 2016, Front. Plant Sci. 7:1940, doi:10.3389/fpls.2016.01940 | [F] |
| Ppd – marker P51600 | Pv01 | about 3% recombination | Gu, Zhu, Wallace, Singh & Weeden 1998, Euphytica 102:125–132 [pages K], doi:10.1023/A:1018340514388 | [F] abstract |
| Hr – marker B303600 | ? | about 13 cM; Hr has no effect in ppd/ppd | Gu et al. 1998 | [F] abstract |
| P – Phs (phaseolin, a seed-weight proxy) | Pv07 (D7/B7) | **9 cM** in two maps (Nodari 1992; Koinange 1996) | Park et al. 2000, J. Amer. Soc. Hort. Sci. 125:466–475, https://journals.ashs.org/downloadpdf/view/journals/jashs/125/4/article-p466.pdf | [F] (secondary citation) |
| P – Phs | Pv07 | 15.5 cM (as cited by Park 2000). The original gene order reads "Pha-8-(Spe/Spg)-24-P-9-(Spa/Spb)", which gives about 32 cM | Vallejos & Chase 1991, TAG 82:353–357, doi:10.1007/BF02190622 | [F] abstract; the two values conflict |
| P – seed weight | Pv07 | Association confirmed by GWAS; no cM given. Classic linkage: Sax 1923 | McClean et al. 2018, New Phytol. 219:1112–1123, doi:10.1111/nph.15259 (via PubMed); Sax 1923 Genetics 8:552 [K] | [F] association |
| P position | Pv07 | "Nearly terminal" on B7. PvTT8/P candidate Phvul.007G171333, QTL interval 28.3–30.0 Mb (v2.1) | Gene list 2008 (Erdmann 2002; Vallejos 1992); Parker et al. 2023/2024 | [F] |
| Phs – Bct – Asp | Pv07 | Order Phs – Bct – Asp (Bct between them); no cM | Gene list 2008 citing Miklas et al. 2000 | [F] |
| Seed thickness QTL ST-7 | Pv07 | 27.8–31.9 cM | González 2016 | [F] |
| B – I (BCMV) | Pv02 end | "Very tightly linked" (Kyle & Dickson 1988); I is nearly terminal | Gene list 2008; McClean et al. 2002 put B at the end of B2, consistent with I | [F] |
| Rk – B | Pv02 | 28 cM (Prakken 1972b) | Gene list 2008 | [F] |
| Ane – B | Pv02 | Linked (Lamprecht 1964), no cM | Gene list 2008 | [F] |
| St – PvIND | Pv02 | Co-map | Di Vittori et al. 2020 bioRxiv (prior notes) | [F] (prior session) |
| St – I / B | Pv02 | St on the same chromosome; distance not read | Prior notes | [I]: do not link them tightly in the game; use about 40 cM |
| Bip – J | Pv10 | "Linked"; J marker OL4S500 is 27.7 cM from the bipana STS | McClean et al. 2002, J. Hered. 93:148–152, doi:10.1093/jhered/93.2.148; https://webpages.charlotte.edu/~jweller2/pages/BINF8350f2011/BINF8350_Readings/McCleanBassett_MappingSeedCoatPhas_JHered2002.pdf | [F] |
| Bip – bip^ana | Pv10 | 0.7 cM between STS markers (alleles of one gene) | McClean 2002 | [F] |
| C – Gy | Pv08 | 2.7 cM (Gy may be part of the complex C) | McClean 2002; gene list | [F] |
| C – R (red) – Prp | Pv08 | Within the "complex C locus" (M, Pr, Acc, C, R, Cst; Prakken 1974). In the game, treat as one multi-allelic locus | Gene list 2008 | [F] |
| V – Rf-1 / V – Am | Pv06 | 11 cM / 9 cM | Gene list 2008 | [F] |
| T – RFLP D1096-2 | Pv09 | 9.0 cM | McClean 2002 | [F] |
| Co-1 – Ur-9 | Pv01 | "near" | Gene list 2008 | [F] |
| Co-2 – Ur-3 | Pv11 | linked | Gene list 2008 | [F] |

## 2. Locus position table

| Locus | Phenotype | Chromosome | Position / gene model | Source | Tag |
|---|---|---|---|---|---|
| fin (PvTFL1y) | determinate (fin fin) vs indeterminate | Pv01 | Phvul.001G189200; about 76.9 cM on the consensus map | Repinski 2012; Kwak 2008; González 2016 | [F] |
| Ppd (PHYA3) | photoperiod sensitivity (dominant) | Pv01 | Phvul.001G221100, 47.64 Mb | González 2021; Kwak 2008 (B1) | [F] |
| Hr | strength of the photoperiod response | ? | 13 cM to B303600 | Gu 1998 | [F]; chromosome unknown |
| P (PvTT8, bHLH) | colour gate; pp = white seed and flower | Pv07 | Phvul.007G171333; 28.3–30.0 Mb QTL; near terminal on B7 | McClean 2018; Parker 2024; gene list | [F] |
| Asp | shiny (Asp) vs matte (asp asp) | Pv07 | Phs–Bct–Asp order | Gene list (Miklas 2000); Beninger 2000 | [F] |
| Phs (phaseolin) | seed-size proxy | Pv07 | 9 cM from P | Park 2000 | [F] |
| C complex (PvMYB113 cluster) | pattern (mottle, stripe), R red | Pv08 | — | McClean 2002; Parker 2023; Park 2000 (LG 8) | [F] |
| J (R2R3-MYB, Phvul.010G130600) | colour expression, shine, pattern margin | Pv10 | — | McClean 2002; Parker 2023 | [F] |
| Bip (PvMYC1, bHLH) | partly coloured pattern | Pv10 | Phvul.010G098500, 35.6–36.9 Mb | McClean 2002; Parker 2024 | [F] |
| G | yellow-brown intensifier | Pv04 | — | McClean 2002 | [F] |
| B | greenish-brown modifier | Pv02 | end of chromosome, next to I | McClean 2002; gene list | [F] |
| Rk | recessive red (kidney) | Pv02 (by 28 cM linkage to B) | — | Gene list (Prakken 1972b) | [F] (inferred chromosome) |
| V (F3′5′H, Phvul.006G018800) | flower purple (V) / pink (v^lae) / white (v); violet to black seed | Pv06 | — | McClean 2002; Parker 2023 | [F] |
| T (PvTTG1, WD40) | full colour (T) vs partly coloured (tt); tt also gives white flowers | Pv09 | Phvul.009G044700, 8.4–9.2 Mb (v2.1); 10.1–11.5 Mb on the 5-593 assembly | McClean 2002; Parker 2024; McClean et al. 2023 bioRxiv 10.1101/2023.09.28.560026 | [F] |
| Z (PvTT2, R2R3-MYB) | partly coloured pattern | Pv03 | Phvul.003G132100; 33.7–35.1 Mb (5-593) | Parker 2024; McClean 2023 bioRxiv; McClean 2002 (B3) | [F] |
| D | allele of Z (not a separate locus) | Pv03 | — | Gene list (Bassett 1999b); McClean 2002 | [F] |
| Fib | fibula arcs (dominant) | ? | — | Bassett & McClean review | [F] that the locus exists; position unknown |
| Gy | greenish yellow (recessive) | Pv08 | 2.7 cM from C | McClean 2002 | [F] |
| I | BCMV resistance (dominant) | Pv02 | near terminal | Gene list; Blair 2018 | [F] |
| St | pod string | Pv02 | co-maps with PvIND | Di Vittori 2020 (prior notes) | [F] |
| Co-1 / Co-2 / Co-3 / Co-4 / Co-6 | anthracnose resistance | Pv01 / Pv11 / Pv04 / Pv08 / Pv07 | — | Gene list 2008 | [F] |
| Co-4² / Co-5 | anthracnose | not stated in the list; Co-4² is an allele at Co-4 (Pv08); Co-5 on Pv07 | — | [K, to verify] | [K] |
| y (yellow wax pod; acts with Arg) | pod colour | not stated | — | Gene list | [F] that no position is given |
| Flower colour | = V (plus T and P, which also whiten flowers) | Pv06 | — | McClean 2002 | [F] |
| Growth habit | = fin (plus modifiers) | Pv01 | — | — | [F] |

**Note on the St entry.** The gene list's St entry reads "St stringless pod; st gives a complete string (Prakken 1934)" [F]. That is the opposite of the prior notes, which have St dominant = stringy. Treat the direction as uncertain: [K, to verify]. Modern usage generally gives St (stringless) as dominant.

## 3. Seed-colour rule table

### Gene roles

- **P** is the ground factor. With pp, the seed is white whatever the other loci are, and the flower is white too [F: McClean 2002; Beninger 2000; McClean 2023 bioRxiv]. With p^gri, the seed is grey-white and the flower nearly white [F: McClean 2002].
- **C and J** are both needed for the colour-modifying genes to express fully. G gives yellow-brown, B greenish-brown, and V violet to black. Rk is a recessive red (kidney) gene. R is a dominant red gene tightly linked to C [F: Beninger et al. 2000, J. Amer. Soc. Hort. Sci. 125:52–58, https://journals.ashs.org/downloadpdf/view/journals/jashs/125/1/article-p52.pdf].
- **j** gives paler, "immature" colours [F: Beninger 2000].
- **asp asp** gives a dull (matte) coat [F: Beninger 2000].
- **V** has a dominance series: V gives purple flowers, v^lae pink, and v white. V seed coats contain anthocyanins; vv seed coats contain flavonols [F: McClean 2002].
- **C** produces patterns (stripes, speckles, mottles) on a light background, usually cartridge buff, overlain by a darker pattern colour [F: McClean 2002].
- **Segregation ratios** [F: McClean 2002]:
  - C: 3 grey brown : 1 cartridge buff
  - G: 3 yellow brown : 1 pale greenish yellow
  - V: 12 black : 3 mineral brown : 1 buffy citrine
- **gy gy** gives greenish-yellow [F: McClean 2002].
- **Partial patterns** need P_ and tt. The coloured area can take any hue, and the uncoloured area is always white. tt is epistatic to V and gives white flowers [F: Bassett & McClean review, https://www.bic.uprm.edu/wp-content/uploads/2018/04/BeanSeedColor.pdf; McClean 2023 bioRxiv].
- **Temperature:** the pigmented area of t patterns rises with temperature. It increased up to 11-fold from the coolest to the warmest sites, and a 4 °C rise added about 21% of the seed area in growth chambers [F: Parker 2024, PubMed abstract].

### Evaluation order for the game [I]

Evaluate in this order. The first matching gate wins.

1. **Gate P.** pp gives a white seed and a white flower. Stop.
2. **Gate T.** tt gives a partly coloured seed (step 6), and the flower is white whatever V is.
3. **Hue** from C/R, J, G, B, V and Rk (steps 4 and 5).
4. **Pattern** from the C-complex allele: self (solid), mottled, striped or speckled. The pattern colour comes from step 3, on a buff background.
5. **Shine** from Asp: Asp_ is glossy, asp asp is matte.
6. **Partial pattern:** the coloured region from step 3, the rest white.

### Hue rules (P_, T_, self-coloured C)

| # | Genotype (dominance as written) | Phenotype | Source | Tag | RGB hint [I] |
|---|---|---|---|---|---|
| H1 | pp, anything else | white (navy, great northern, white kidney) | McClean 2002/2018; Beninger 2000 | [F] | #F4F1E8 |
| H2 | p^gri p^gri | grey-white | McClean 2002 | [F] | #DAD6CC |
| H3 | P C J G B V Asp | black, glossy (black turtle type) | Beninger 2000 ("black beans P C J B V") | [F] | #1A1416 |
| H4 | P C J G B V asp | matte black (black turtle soup is typically matte) | Beninger 2000 | [F] genotype; [K] the market-class link | #262123 |
| H5 | P C J G b V Asp | dark brown violet | Beninger 2000 | [F] | #3F2630 |
| H6 | P [C r] J G B v | dark brown | Hosfield chemistry review, https://webpages.charlotte.edu/~jweller2/pages/BINF8350f2011/BINF8350_Readings/Hosfield_ChemistrySeedCoatColor.pdf | [F] | #4A3020 |
| H7 | P [C r] J G b v | yellow brown | Hosfield; Beninger 2000 | [F] | #A5773A |
| H8 | P C J g B V / v (G population) | gg = buffy citrine or chamois; GG = mineral brown or yellow brown | McClean 2002 | [F] | buffy citrine #B9A35A; chamois #D9C08A; mineral brown #6E4E37 |
| H9 | V population, 12 : 3 : 1 | black : mineral brown : buffy citrine | McClean 2002 | [F] ratios. [I] reading: two genes with dominant epistasis, e.g. V_ = black, vv B_ = mineral brown, vv bb = buffy citrine | as above |
| H10 | P [cu r] J g B v rk^d | dark red kidney (garnet red) | Hosfield, citing Bassett 1998c; Beninger 2000 (Montcalm) | [F] | #6B1A1F |
| H11 | P C J g B v rk rk | light red kidney | [K, to verify] | [K] | #A4413C |
| H12 | P [C R] J … (dominant R) | red (small red / Mexican red) | R from Beninger 2000 [F]; the class link is [K] | [F/K] | #8E2A23 |
| H13 | P [C r] d j G b v^lae | Prim / Manteca: light yellow | Beninger 2000; Hosfield | [F] | #E8D58A |
| H14 | P … j (otherwise as H7) | pale yellow, "immature" colour (Canario/Mayocoba-like) | Beninger 2000 (j paler) | [F] rule; [K] class | #E9D36A |
| H15 | P C J G B v gy gy | strong greenish-yellow | McClean 2002 (Wagenaar) | [F] | #C9C35A |
| H16 | P C J G B V Rk Gy T Z Bip Asp | line 5-593: black, the standard parent | McClean 2002; Beninger 2000 | [F] | #1A1416 |

### Pattern rules (P_, T_): C-complex alleles

| # | Genotype | Phenotype | Tag | RGB hint [I] |
|---|---|---|---|---|
| C1 | C (self) | solid colour (hue from the table above) | [F] McClean 2002 | — |
| C2 | c c | cartridge-buff background without the dark pattern. The C population gave 3 grey brown : 1 cartridge buff | [F] | #E3CFA4 |
| C3 | C^st (striped) [K] or c^u / [C M] mottled allele (pinto mottle) [K] | buff background with the pattern colour in stripes or mottles | [K, to verify] | pinto: buff #D8B98C with mottle #7A4A2B |
| C4 | Cc heterozygotes | change the pattern (incomplete dominance at C) | [F] Plestenjak 2025 (prior notes) | — |

### Partial-pattern rules (P_, tt), from the Bassett & McClean review [F]

| Genotype at t | Pattern |
|---|---|
| t Z Bip fib | self-coloured (full; "expansa" without Fib) |
| t Z Bip Fib | expansa with fibula arcs |
| t Z/z Bip | ambigua (unstable) |
| t z Bip | virgarcus (dorsal colour) |
| t z^sel/z Bip | piebald |
| t z^sel Bip | sellatus |
| t Z bip^ana | anasazi |
| t z bip^ana | anabip |
| t z bip | bipunctata (two spots by the hilum) |
| t Z j | marginata |
| t z l / l^ers | weak virgarcus (arcus) |
| t z j | white |
| t^cf z l^ers + an unidentified gene | two points (t^cf keeps the flower coloured) |

Allelic series: Bip > bip^ana > bip, and Z > z^sel > z [F: Parker 2024 / bioRxiv].

The McClean 2023 bioRxiv states it plainly [F]: "P t Z Bip J fib express full seed color, P t z Bip J fib express dorsal seed color (virgarcus), and P t z bip J express color in two regions neighboring the hilum."

Game coverage fraction of the coloured area [I]:

| Pattern | Coloured fraction |
|---|---|
| self | 1.0 |
| expansa | 0.85 |
| marginata | 0.5 |
| virgarcus | 0.35 |
| sellatus | 0.3 |
| anasazi | 0.25 |
| bipunctata | 0.1 |
| white | 0 |

Optionally scale the fraction by temperature at seed fill, about +5% of the area per °C [I, from Parker 2024's 4 °C → +21%].

### Named market classes

| Class | Genotype (key loci) | Tag | RGB hint [I] |
|---|---|---|---|
| Black turtle | P T C J G B V asp (matte) or Asp (glossy) | [F] (Beninger: black = P C J B V); the matte class link is [K] | #1E1A1C |
| Navy (pea bean) | pp; small seed. The seed-size QTL is linked to P, which fits navy being small and white | [F] pp = white; [K] that navy specifically is p | #F4F1E8 |
| Great northern | pp; medium-large, flatter seed | [K, to verify] | #F2EEE2 |
| White kidney / cannellini | pp | [K] | #F3EFE6 |
| Pinto | P T, mottled C-complex allele, a brown hue (G B v); buff ground with brown mottles | [K, to verify]: Bassett symbol likely [C M r] or similar | ground #D8B98C, mottle #7A4A2B |
| Cranberry / borlotti | P T, striped/mottled C-complex allele, red hue (R or rk) on a cream ground | [K, to verify] | ground #E8D9C0, pattern #9B2C3A |
| Dark red kidney | P [cu r] J g B v rk^d | [F] | #6B1A1F |
| Light red kidney | P … rk | [K] | #A4413C |
| Small red | P [C R] | [K] | #8E2A23 |
| Pink | P [C R] with diluting modifiers (g, b?) | [K, to verify] | #C9837A |
| Yellow (Canario, Mayocoba, Manteca/Prim) | P [C r] d j G b v^lae (Prim) | [F] for Prim; [K] for other yellows | #E8D58A |
| Orca / Calypso (partly coloured, black and white) | P tt with Z/Bip variants | [F] Orca is studied in Parker 2024; [K] for its exact genotype | black #1E1A1C on #F4F1E8 |
| Anasazi | P t Z bip^ana | [F] | red-maroon #7E2A2A on white |

## 4. Inferences

- **[I] Proposed WP28 positions** (chromosome, cM) for named loci, from the sources above. The absolute cM values are game placements, not published map positions, except where a distance between two loci is [F].

  | Locus | Chromosome | cM | Notes |
  | --- | --- | --- | --- |
  | Co-1 | Pv01 | 2 | |
  | fin | Pv01 | 77 | |
  | Ppd | Pv01 | 85 | 8 cM from fin |
  | Z-near-fin | — | — | Do not model; use Z on Pv03 |
  | B | Pv02 | 2 | |
  | I | Pv02 | 1 | 1 cM from B ("very tight") |
  | Rk | Pv02 | 30 | 28 cM from B |
  | St | Pv02 | ~60 | [I] position |
  | Z | Pv03 | 60 | |
  | G | Pv04 | 50 | |
  | Co-3 | Pv04 | 5 | |
  | PvMYB26 (shattering) | Pv05 | 50 | |
  | V | Pv06 | 10 | |
  | seed-size QTL | Pv07 | 87 | |
  | Phs | Pv07 | 91 | 9 cM from P |
  | Asp | Pv07 | 70 | Order Phs–Bct–Asp |
  | P | Pv07 | 100 | Near terminal |
  | C complex | Pv08 | 40 | Gy at 2.7 cM from it if used |
  | Co-4 | Pv08 | 90 | |
  | T | Pv09 | 20 | |
  | Bip | Pv10 | 40 | |
  | J | Pv10 | 67 | 27 cM from Bip |
  | Co-2 | Pv11 | 95 | |

- **[I] Linkage stories to teach.**
  - Ppd and fin come from different parents (repulsion): about 8% recombinants, so a bush, day-neutral F2 plant is about 0.08 × 0.08 / 4 ≈ 0.2% in the worst case. A good "need a big F2 or backcross" puzzle.
  - In white (pp) navy × coloured large-seeded crosses, white F2 seeds are smaller on average via Phs/QTL linkage at about 10 cM.
  - B–I: selecting for BCMV resistance drags the B colour allele.
- **[I] Model the C complex as one multi-allelic locus** with alleles {C (self), c (buff, no pattern), C^mot (mottle), C^st (stripe), C^R (self red)}. Its internal sub-loci are too tight to recombine in a game (Prakken).
- **[I] The colour layer is a pure function** of about 12 loci: P, T, C, J, G, B, V, Rk, Z, Bip, Asp and Fib. Use a lookup with an "unknown combination" fallback that mixes hue additively (G adds yellow, B adds green-brown, V adds violet-black, rk adds red). Determinism is trivial because there is no RNG. Temperature modulation of the partial pattern is optional.
- **[I] Mutation tiers.** Colour loci are the natural "named tier" at 1e-5. A p mutation on a coloured line gives white seeds 3:1 in the selfed progeny: a nice discovery event.

## 5. Gaps

- **fin–Ppd cM was not read.** It is in Kwak et al. 2008 Fig. 1 (Kosambi). Gu et al. 1998 could give it indirectly: P51600 is about 3% from ppd. The Mb coordinate of Phvul.001G189200 was not confirmed; NCBI/Phytozome were captcha-blocked. The 8 cM value is [I].
- **P–seed-weight QTL distance.** Only the P–Phs proxy is available, and the two values conflict: 9 cM (two maps, cited by Park 2000) against 15.5 / about 32 cM (Vallejos & Chase 1991). The original Sax 1923 and Koinange 1996 papers were not read.
- **Z position conflict.** Bassett's "Fin 1 cM from Z" conflicts with Z = PvTT2 on Pv03 (McClean 2002 B3; Parker 2024). Parker proposes a sellatus-related WD40 candidate 245 kb from fin on Pv01, which may explain the old linkage. Unresolved.
- **Gene list transcription errors.** The Porch 2008 gene list puts C on B10 and Fin on B6. McClean 2002 and Park 2000 put C on LG 8 and fin on B1. I treated the gene list as an error on these two lines.
- **Allele symbols still [K, to verify].** Market-class genotypes for pinto, cranberry, pink, small red, light red kidney, great northern and navy were not read. Bassett has papers on each, e.g. "The seedcoat color genotype of 'Prim' and the Manteca and Coscorron market classes" (AGRIS record https://agris.fao.org/search/en/records/65de32990f3e94b9e5cbc5ad) and the BIC "complex C region [?R]" note. The C-complex allele symbols for mottling and striping (C^st, c^u, [C M]) are memory only.
- **St direction conflict.** The gene list says "St stringless; st complete string"; the prior notes say the opposite. Resolve before shipping.
- **Not found:** chromosome of y (wax pod), Hr, Fib or Ppd–Hr; positions of Co-4² and Co-5.
- **RGB hints are all [I].** They are approximations of the Munsell/ISCC-NBS names Bassett uses (cartridge buff, buffy citrine, chamois, mineral brown, garnet). Check against photos.
- **Sources blocked:** curl to these hosts was blocked by the proxy (403); content came through WebFetch extracts. The BIC site bic.css.msu.edu was not reached directly; I used the bic.uprm.edu mirrors. Schmutz et al. 2014 and McConnell et al. 2010 were not read.

## Sources

- McClean PE, Lee RK, Otto C, Gepts P, Bassett MJ. 2002. Molecular and phenotypic mapping of genes controlling seed coat pattern and color in common bean. J Hered 93:148–152. doi:10.1093/jhered/93.2.148. https://webpages.charlotte.edu/~jweller2/pages/BINF8350f2011/BINF8350_Readings/McCleanBassett_MappingSeedCoatPhas_JHered2002.pdf [F]
- Bassett MJ, McClean P. A brief review of the genetics of partly colored seed coats in common bean. BIC. https://www.bic.uprm.edu/wp-content/uploads/2018/04/BeanSeedColor.pdf [F]
- List of Genes – Phaseolus vulgaris L. (Yarnell 1965; Dickson 1982; Bassett 1989+; Porch 2008 update). https://www.bic.uprm.edu/wp-content/uploads/2018/04/BeanGenesList.pdf [F]
- Beninger CW, Hosfield GL, Bassett MJ, Owens S. 2000. Chemical and morphological expression of the B and Asp seedcoat genes in Phaseolus vulgaris. J Amer Soc Hort Sci 125:52–58. https://journals.ashs.org/downloadpdf/view/journals/jashs/125/1/article-p52.pdf [F]
- Hosfield GL. Seed coat color in Phaseolus vulgaris L.: its chemistry and associated health related benefits. https://webpages.charlotte.edu/~jweller2/pages/BINF8350f2011/BINF8350_Readings/Hosfield_ChemistrySeedCoatColor.pdf [F]
- Parker T et al. 2024. Seed color patterns in domesticated common bean are regulated by MYB-bHLH-WD40 transcription factors and temperature. Plant J 119:2765–2781. doi:10.1111/tpj.16947 (via PubMed); preprint https://www.biorxiv.org/content/10.1101/2023.09.23.559151.full.pdf [F]
- McClean PE et al. 2023. T and Z, partial seed coat patterning genes in common bean… bioRxiv doi:10.1101/2023.09.28.560026 [F]. The journal version (J Exp Bot?) is [K, to verify].
- McClean PE et al. 2018. White seed color in common bean results from convergent evolution in the P gene. New Phytol 219:1112–1123. doi:10.1111/nph.15259 (via PubMed) [F]
- Kwak M, Velasco D, Gepts P. 2008. Mapping homologous sequences for determinacy and photoperiod sensitivity in common bean. J Hered 99:283–291. https://scispace.com/pdf/mapping-homologous-sequences-for-determinacy-and-photoperiod-2xvbkclp2e.pdf [F]
- Repinski SL, Kwak M, Gepts P. 2012. TAG 124:1539–1547. doi:10.1007/s00122-012-1808-8 [F]
- González AM et al. 2016. Front Plant Sci 7:1940. doi:10.3389/fpls.2016.01940 [F]
- González AM et al. 2021. Front Plant Sci 11:599462. doi:10.3389/fpls.2020.599462 [F]
- Gu WK, Zhu JQ, Wallace DH, Singh SP, Weeden NF. 1998. Analysis of genes controlling photoperiod sensitivity in common bean using DNA markers. Euphytica 102. doi:10.1023/A:1018340514388. https://agris.fao.org/search/en/records/67235df1b605bda15e0ab333 [F abstract]
- Campa A, Pérez E, Giraldez R, Ferreira JJ. Linkage relationship between a common bean seed protein and Fin/fin. BIC 48:36–37. https://www.unioviedo.es/esr/rgiraldez/privado/articulos/BIC48-36.pdf [F]
- Vallejos CE, Chase CD. 1991. Extended map for the phaseolin linkage group. TAG 82:353–357. doi:10.1007/BF02190622 [F abstract]
- Park SO et al. 2000. Mapping of QTL for seed size and shape traits in common bean. J Amer Soc Hort Sci 125:466–475. https://journals.ashs.org/downloadpdf/view/journals/jashs/125/4/article-p466.pdf [F]

## 6. Addendum (WP28 step 2): complete named-loci table

### Epistasis sources

| Paper | Status | Note |
|---|---|---|
| Parker et al. 2024, "Seed color patterns in domesticated common bean are regulated by MYB-bHLH-WD40 transcription factors and temperature", Plant J 119(6):2765–2781, doi:10.1111/tpj.16947 | [F] PubMed metadata; bioRxiv full text extract | The "Parker 2024" paper on seed-coat colour genetics |
| Bassett MJ 2007, "Genetics of seed coat color and pattern in common bean", Plant Breeding Reviews 28:239–315 | [K, to verify]: not reachable this session | Plestenjak et al. 2025 [F] cite "Bassett (2007)" for P, T and the C-locus patterns, without the full reference in the extracted text |
| "Bassett 2007, a revised linkage map of common bean" | Not found | The 2008 BIC gene list (Bassett/Porch) is Bassett's map-relevant output that I read |

The rule tables in §3 come from the Bassett-authored primary papers I read:
- McClean & Bassett 2002
- Beninger, Hosfield & Bassett 2000
- the Bassett & McClean partly coloured review
- Hosfield, citing Bassett 1998c and 1999

### Chromosome lengths for converting Mb to cM

**Physical lengths, G19833 v2.1 (Mb)** [K, to verify; Phytozome]:

| Chr | Pv01 | Pv02 | Pv03 | Pv04 | Pv05 | Pv06 | Pv07 | Pv08 | Pv09 | Pv10 | Pv11 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Mb | 52.2 | 49.0 | 52.3 | 45.8 | 40.8 | 32.0 | 51.7 | 63.0 | 38.3 | 44.6 | 53.6 |

**Genetic lengths:**
- Blair 2018 [F]: mean 99.8 cM, Pv01 147 cM, Pv09 46.5 cM, 2.13 cM/Mb on average, with recombination suppressed in the pericentromeres.
- The 2009 BIC core map extract gave roughly [F, fetch extract]: B1 107, B2 175, B3 132, B4 95, B5 113, B6 105, B9 89 and B10 100 cM. The B7, B8 and B11 values were ambiguous.

**Recommended conversion [I]:** do not convert Mb to cM linearly. Most crossovers fall in the distal arms. Assign game cM by hand, in this order:
1. Use the published cM between loci where one exists.
2. Otherwise, inside a distal arm use about 3 cM/Mb.
3. Across a pericentromere use about 0.3 cM/Mb.

### Named loci

In the "Game cM" column, a value is [I] unless a pairwise distance is cited. Phenotypes and dominance are [F] where the source is marked [F].

| # | Locus | Chr | Physical (G19833 v2) | Game cM [I] | Phenotype | Dominance / alleles | Source | Tag |
|---|---|---|---|---|---|---|---|---|
| 1 | fin (PvTFL1y, Phvul.001G189200) | Pv01 | about 44–45 Mb [K] | 77 (consensus FIN 76.9 cM [F]) | Habit: indeterminate (pole) vs determinate (bush) | Fin dominant; fin fin determinate | Repinski 2012; Kwak 2008; González 2016 | [F] |
| 2 | Ppd (PHYA3, Phvul.001G221100) | Pv01 | 47.64 Mb [F] | 85 (8 cM from fin [I]) | Photoperiod sensitivity (delayed flowering under long days) | Ppd sensitive dominant; ppd ppd day-neutral | González 2021; Gu 1998; Kwak 2008 | [F] |
| 3 | Hr | ? | — | separate chromosome [I] | Strength of the photoperiod response | Acts only in Ppd_ (epistatic) | Gu 1998 | [F] effect; position unknown |
| 4 | Co-1 (alleles Co-1², Co-1³, Co-1⁴; Co-x coincides) | Pv01 | distal end [K, about 50 Mb] | 140 | Anthracnose resistance | Dominant | Anthracnose review (scispace 3u180twgd2) | [F] chr; [K] position |
| 5 | B | Pv02 | distal end, next to I | 2 | Greenish-brown modifier of seed colour | B_ vs bb (bb gives yellow-brown, dark brown violet) | McClean 2002; Beninger 2000; gene list | [F] |
| 6 | I | Pv02 | marker at 48.91 Mb [F] | 1 ("very tightly linked" to B) | BCMV resistance (hypersensitive) | Dominant | Soler-Garzón et al. 2021, Front. Plant Sci. 12:769247; gene list | [F] |
| 7 | Rk | Pv02 | — | 30 (28 cM from B [F]) | Recessive red (kidney) seed | rk rk red; rk^d dark red | Gene list (Prakken 1972b); Beninger 2000 | [F] |
| 8 | St (pod string) | Pv02 | peak SNPs 44.03–44.25 Mb; PvIND Phvul.002G271000 nearby [F] | 15 (about 4.7 Mb from I, distal arm) | Pod string | Direction conflicts: gene list says "St stringless; st complete string" [F]; earlier notes say the opposite. Resolve before shipping | bioRxiv 10.1101/2022.02.23.481584; Di Vittori 2020; gene list | [F] position; [K] dominance |
| 9 | y (yellow wax pod) | Pv02 | 122 kb region (Mb not given) | 50 [I] | Yellow vs green pod | y recessive gives a yellow (wax) pod, with Arg | Myers et al. 2019 ASHS abstract (https://ashs.confex.com/ashs/2019/webprogramarchives/Paper31089.html); gene list | [F] |
| 10 | Co-u | Pv02 | — | — | Anthracnose | Provisional | Anthracnose review | [F] chr |
| 11 | Z (PvTT2, Phvul.003G132100) | Pv03 | about 33.7–35.1 Mb (5-593 assembly) [F] | 60 | Partly coloured pattern (with tt) | Z > z^sel > z; D is an allele of Z | Parker 2024; McClean 2023 bioRxiv; McClean 2002 | [F] |
| 12 | bc-1 | Pv03 | marker at 4.20 Mb [F] | 10 | BCMV resistance (strain-specific) | Recessive | Soler-Garzón 2021 | [F] |
| 13 | ms-2 (PvTKPR2, Phvul.003G032100) | Pv03 (from the gene ID) | early Pv03 [I] | 8 | Genic male sterility (a breeding tool: no selfing) | Loss of function; recessive [I] | Xu et al. 2023, TAG 136:56, doi:10.1007/s00122-023-04255-8 | [F] gene; [I] chr and dominance |
| 14 | G | Pv04 | — | 50 | Yellow-brown intensifier | G_ vs gg (gg buffy citrine or chamois) | McClean 2002 | [F] |
| 15 | Co-3 (Co-9 cluster; Ur-5) | Pv04 | — | 5 | Anthracnose resistance | Dominant | Anthracnose review; gene list | [F] chr |
| 16 | PvMYB26 (pod shattering, Phvul.005G157600) | Pv05 | — | 70 | Dehiscent vs indehiscent pod | Indehiscence recessive (3:1) | Di Vittori 2020 (earlier notes) | [F] |
| 17 | bc-u / bc-u^d, bc-4 | Pv05 | 36.11 Mb (bc-u^d marker); bc-4 36.23 Mb [F] | 95 | BCMV, needed for bc-1/bc-2 to act | Recessive | Soler-Garzón 2021 | [F] |
| 18 | V (F3′5′H, Phvul.006G018800) | Pv06 | near the start of Pv06 (gene rank about 9% along) [I] | 10 | Flower purple / pink / white; violet-black seed | V > v^lae > v | McClean 2002; Parker 2023 | [F] |
| 19 | bc-3 (PveIF4E) | Pv06 | 27.20 Mb [F] | 90 | Broad BCMV/BCMNV resistance | Recessive | Soler-Garzón 2021 | [F] |
| 20 | Phs (phaseolin) | Pv07 | proximal [K] | 91 (9 cM from P) | Seed protein; proxy for a seed-size QTL | Codominant | Park 2000; Vallejos & Chase 1991 | [F] linkage |
| 21 | SW7 (seed-size QTL linked to P) | Pv07 | — | 90 | 100-seed weight (additive) | Additive | McClean 2018 (GWAS P–seed weight); Sax 1923 [K] | [F] association; [I] position |
| 22 | Co-6 | Pv07 | between P and Phs [F] | 95 | Anthracnose | Dominant | Anthracnose review | [F] |
| 23 | Asp | Pv07 | order Phs–Bct–Asp [F] | 75 [I]: puts Asp on the far side of Phs from P. That is unverified; the order Phs–Bct–Asp gives no side relative to P | Seed-coat shine | Asp glossy dominant; asp asp matte | Gene list (Miklas 2000); Beninger 2000 | [F] |
| 24 | P (PvTT8, Phvul.007G171333) | Pv07 | 28.3–30.0 Mb QTL [F]; genetically "nearly terminal" | 100 | Colour gate: pp gives white seed and flower | P dominant; p^gri grey-white | McClean 2018; Parker 2024; gene list | [F] |
| 25 | C complex (PvMYB113 cluster; R, M, Pr, Acc, Cst; Prp) | Pv08 | — | 40 | Pattern (mottle, stripe), R red, Prp | Multi-allelic; Cc changes the pattern | McClean 2002; Park 2000; Plestenjak 2025; gene list | [F] |
| 26 | Gy | Pv08 | — | 43 (2.7 cM from C) | Greenish-yellow seed | gy gy recessive | McClean 2002 | [F] |
| 27 | Co-4 (Co-4², Co-4³) | Pv08 | short arm (FISH) | 5 | Anthracnose (Co-4² is broad) | Dominant | Anthracnose review (Melotto 2004) | [F] |
| 28 | T (PvTTG1, Phvul.009G044700) | Pv09 | 8.4–9.2 Mb QTL [F] | 15 | Full vs partly coloured seed; tt also gives white flowers | T dominant; t^cf keeps the flower coloured | Parker 2024; McClean 2002; Bassett & McClean review | [F] |
| 29 | Bip (PvMYC1, Phvul.010G098500) | Pv10 | 35.6–36.9 Mb [F] | 40 | Partly coloured pattern | Bip > bip^ana > bip | Parker 2024; McClean 2002 | [F] |
| 30 | J (R2R3-MYB, Phvul.010G130600) | Pv10 | distal of Bip (by gene ID) [I] | 67 (marker about 27.7 cM from bipana) | Colour expression, shine, marginata pattern | J_ full; j pale ("immature") colour | McClean 2002; Beninger 2000; Parker 2023 | [F] |
| 31 | Fib | ? | — | separate [I] | Fibula arcs in expansa | Dominant | Bassett & McClean review | [F] |
| 32 | Co-2 | Pv11 | distal [K] | 95 | Anthracnose | Dominant | Anthracnose review | [F] chr |
| 33 | Ur-3 (Ur-11 cluster) | Pv11 | with Co-2 [F]; about 46–47 Mb [K] | 93 | Rust resistance | Dominant | Anthracnose review | [F] chr |
| 34 | bc-2 | Pv11 | 9.27 Mb [F] | 20 | BCMV | Recessive | Soler-Garzón 2021 | [F] |

### Requested symbols not found

- **Ts and fd:** these were placeholder symbols in the research brief, not real loci. Drop them.
- **Ms:** the documented example is ms-2 (row 13), a recessive mutant. There are older ms-1, ms-3 and ms-8 symbols [K].

### Budget check

**[I] Budget.** These 34 rows (plus the anonymous QTL) exceed 40 if all are used. A suggested core of 24 named loci:

| Group | Loci |
|---|---|
| Habit and phenology | fin, Ppd |
| Seed colour and shine | P, T, C, J, G, B, V, Rk, Z, Bip, Asp |
| Pod traits | St, y, MYB26 |
| Disease resistance | I, bc-3, Co-1, Co-4, Co-2 |
| Seed size | SW7 |
| Breeding tool | ms-2 |
| Optional | Gy or Fib |

That leaves 6–16 anonymous QTL slots.

### Linked blocks this layout creates

| Chromosome | Block | Spacing |
|---|---|---|
| Pv01 | fin–Ppd | 8 cM |
| Pv02 | I–B–St–Rk | 1, 13, 15 cM |
| Pv07 | P–Co-6–Phs/SW7 | 5–10 cM |
| Pv08 | C–Gy | 2.7 cM |
| Pv10 | Bip–J | 27 cM |
| Pv11 | Co-2–Ur-3 | 2 cM |

Every one is either published [F] or close to physical spacing [I].

### Extra sources for §6

- Soler-Garzón A, McClean PE, Miklas PN 2021. Front. Plant Sci. 12:769247. doi:10.3389/fpls.2021.769247 [F]
- "A comprehensive review of the major genes conditioning resistance to anthracnose in common bean", https://scispace.com/pdf/a-comprehensive-review-of-the-major-genes-conditioning-3u180twgd2.pdf [F]. Authors and year are not shown in the extract; probably Kelly & Vallejo 2004, HortScience 39:1196 [K].
- St GWAS bioRxiv 10.1101/2022.02.23.481584 [F]. Authors are not shown in the extract.
- Myers JR et al. 2019, ASHS abstract on the wax pod gene y [F]
- Xu K et al. 2023, TAG 136:56, doi:10.1007/s00122-023-04255-8 [F]
- Plestenjak E et al. 2025, Front. Plant Sci. 15:1523745, doi:10.3389/fpls.2024.1523745 [F]
- BIC Bean Core Map 2009, https://www.bic.uprm.edu/wp-content/uploads/2018/04/Bean_Core_Map_2009.pdf [F]. Lengths only; the gene positions were unreadable in the text extract.
