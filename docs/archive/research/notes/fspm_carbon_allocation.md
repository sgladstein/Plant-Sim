# FSPMs, carbon allocation and source-sink models, compared with design plan v3 (§3)

Scope: what established functional-structural plant models (FSPMs) do for structure, organ growth and carbon allocation; parameter values worth adopting; where plan v3 §3 (lines 92-293 of /mnt/project-files/design/design-plan-v3-complete.md) departs from established practice, and how risky each departure is.

Provenance key. **[F]** = read in this session (page fetched, or abstract returned by search). **[K]** = standard reference cited from prior knowledge; the DOI/URL is given but the text was not re-read in this session, so a writer should treat specific numbers tagged [K] as "to verify". Anything under "Inferences" is my reasoning, not a source claim.

---

## 1. GreenLab: source-sink formulation, common pool, beta-law sink variation, calibration

### Takeaway
GreenLab puts each growth cycle's biomass into one common pool. It shares the pool among expanding organs in proportion to (organ-type sink strength × beta-law sink variation with organ age), and fits the "hidden" sink parameters to organ-mass data by inverse non-linear least squares. It is well proven for calibration, but by construction it cannot produce local starvation or branch autonomy. Our per-organ reserves are a deliberate departure, and they make the model harder to calibrate.

### Cited Findings
- GreenLab (Yan, Kang, de Reffye, Dingkuhn 2004) runs organogenesis with a dual-scale automaton on growth cycles of constant thermal time. Biomass production is computed from transpiration with constant transpiration efficiency. "The fresh biomass is then distributed among expanding organs according to their relative demand." Demand comes from allometry plus potential-growth kinetics per organ type, and these potential growth rates "are then used as estimates of relative sink strength." Hidden parameters are fitted by non-linear least squares, running the model "in inverted mode". The model reproduced variability of leaf size along the plant, and compensatory growth after pruning, as an effect of internal competition. [F] — [Yan et al. 2004, Ann Bot 93:591-602](https://consensus.app/papers/details/b29026a2d78b595288c3c0d065ff83dd/?utm_source=claude_desktop)
- Wheat calibration: organ masses from four harvests were fitted simultaneously. The fit estimated "the sink strengths of organs of each type, age and tiller order, and two empirical parameters linked to biomass production." [F] — [Kang et al. 2007, Ann Bot](https://consensus.app/papers/details/7f560f7dff5a5ab6baa61391eefe83b1/?utm_source=claude_desktop)
- Maize "multi-fitting": fitting against several growth stages gave better parameter accuracy than a single fit at maturity, and it "permitted extracting generic organ expansion kinetics from the static observations." Predictions of inter-seasonal variability in grain-fill partitioning were weaker, which the authors put down to cob sink size and terminal leaf senescence. [F] — [Guo et al. 2006, Ann Bot 97](https://consensus.app/papers/details/e4f7f8f8497f50d79a4622048baf6769/?utm_source=claude_desktop)
- Across plant population densities in maize, the sink-variation (beta) parameters and most relative sink strengths "varied little", while the biomass-production parameter and internode sink strength varied with density. [F] — [Ma et al. 2007, Ann Bot](https://consensus.app/papers/details/a223995d0e9a5b8a9b55622412ecd75b/?utm_source=claude_desktop)
- With sparse data, the coefficient of variation was below 15% for most fitted parameters, and the beta-function parameters varied more than the relative sink strengths. [F] — [Ma et al. 2006, PMA06](https://consensus.app/papers/details/d3765092bd38557e90ba1a8db3137fd2/?utm_source=claude_desktop)
- Tomato at three spacings: the sink-strength and light-interception parameters "proved to be environment-dependent", and parameter stability was lost at very high density. Shade plasticity is therefore not fully captured by fixed sink parameters. [F] — [Dong et al. 2007, Ann Bot](https://consensus.app/papers/details/8c3d7b17ff5158b994e30518be1de8ff/?utm_source=claude_desktop)
- The plant-level supply/demand ratio (Q/D) from GreenLab-tomato related sigmoidally to fruit set and also related to specific leaf area (SLA). The authors use Q/D as a generic "competition index". [F] — [Louarn et al. 2007](https://consensus.app/papers/details/2f3fdfa5fa4d59e794f1425139a6863a/?utm_source=claude_desktop)
- GreenLab-Maize-Kernel added reserve remobilisation (non-structural carbohydrates from reserve organs to kernels), so later GreenLab versions do have reserve pools. They remain organ-class pools, not graph-local ones. [F] — [Ma et al. 2018, Ann Bot](https://consensus.app/papers/details/67a8a115e4115e8b9da4ce3b02b76518/?utm_source=claude_desktop). A GreenLab/DSSAT coupling paper names remobilisation parameters k_b and k_i. [F] — [Wang et al. 2023](https://consensus.app/papers/details/4f7739336f715df0a048df36118997d6/?utm_source=claude_desktop)
- An Arabidopsis GreenLab adaptation found three competition phases: none in the vegetative phase, rising in the reproductive phase, and sharply higher when lateral inflorescences develop. [F] — [Christophe et al. 2008, Funct Plant Biol](https://consensus.app/papers/details/c18846d76f945d4fb51b4b0fff7bb4e8/?utm_source=claude_desktop)
- State-of-the-art review (dual-scale automaton, substructures, inverse estimation of source-sink parameters, potential structure, software). [F] — [Wang et al. 2023, Plant Phenomics](https://consensus.app/papers/details/e4e3f4b56b5c5bd4b6a46c929ad70070/?utm_source=claude_desktop)
- GreenLab equations as usually written [K, from the GreenLab literature above, not re-read in this session]:
  - Biomass production per cycle: Q(n) = E(n) · (Sp/r) · (1 − exp(−k · ΣS_leaf / Sp)).
  - Demand: D(n) = Σ_organs P_o · f_o(age).
  - Increment to an organ: Q(n−1) · P_o · f_o(age) / D(n).
  - Sink variation f_o is a discrete beta law over the expansion duration T_o: f(t) ∝ ((t+0.5)/T)^(a−1) · (1 − (t+0.5)/T)^(b−1), normalised to sum to 1. Each organ type has its own a, b, T and sink strength P_o (leaf blade conventionally P = 1).
- GreenLab's sink strengths are relative values fitted per crop. No absolute g C h⁻¹ values transfer.
- No GreenLab calibration for common bean or soybean was found. Crops found: maize, wheat, tomato, cotton, cucumber, rapeseed, Arabidopsis.

### Inferences
- In GreenLab every growing organ gets the same fraction Q/D of its demand, with no lags and no distance effects. **The common pool means a shaded branch cannot starve on its own account.** All organs share the plant's Q/D, so self-pruning and branch autonomy cannot emerge. Our per-organ reserves are needed for the self-pruning goal (§3.1 principle 2). This is the main reason to diverge, and it is justified.
- **The cost of diverging:** GreenLab's main strength is that its parameters can be identified from destructive organ-mass harvests. A local-transport model has extra parameters (conductances, thresholds) that organ-mass data barely constrain. Calibration then becomes "plausibility tuning", not fitting.
  - **Recommendation:** keep a "common-pool mode" switch, equivalent to infinite conductance. Calibrate sink strengths and expansion curves in that mode against bean organ-mass data, GreenLab-style. Then turn on finite conductance and tune only conductance and the starvation thresholds. This also gives a test: with very large k, per-organ results must converge to the common-pool result.
- The plant-level Q/D ratio is a well-used, cheap state variable. Compute it each hour or day and expose it in the inspector. It is also a candidate driver for pod set (Louarn et al. 2007 found a sigmoid relation between Q/D and fruit set in tomato). Our pod-set rule uses the raceme's parent-internode reserve over 3 days, which is a local analogue; both are defensible.

### Gaps
- I did not verify the exact GreenLab beta-law formula or typical a, b values against a primary source in this session. Typical fitted values exist in Guo et al. 2006 and Kang et al. 2007 (tables not read).
- I found no legume (bean or soybean) GreenLab calibration.

---

## 2. Data structures: MTG / OpenAlea, L-systems / L-Py, GroIMP / XL relational growth grammars, Helios

### Takeaway
The field standard is a multiscale tree graph (MTG): nodes are plant components, with typed edges "<" (follows, same axis) and "+" (branches), and scale decomposition (plant / axis / growth unit / metamer). L-systems and L-Py hold the plant as a bracketed string rewritten each step. GroIMP/XL generalises this to graph rewriting. Our SoA organ graph with parent pointers and ordered children matches MTG topology at one scale. Adding the edge type ("<" or "+") and an axis id per node costs little and pays off.

### Cited Findings
- MTG: plant topology as a tree graph at several scales, with "successor" (<) and "branching" (+) edges and decomposition (/) between scales. [K] — [Godin & Caraglio 1998, J Theor Biol 191:1-46](https://doi.org/10.1006/jtbi.1997.0561)
- OpenAlea is a component platform built around the MTG. [K] — [Pradal et al. 2008, Funct Plant Biol 35:751-760](https://doi.org/10.1071/FP08084)
- L-Py: an L-system engine embedded in Python. Modules carry parameters, and the bracketed string can be converted to and from an MTG. [K] — [Boudon et al. 2012, Front Plant Sci 3:76](https://doi.org/10.3389/fpls.2012.00076)
- L-systems: parallel rewriting of bracketed strings, with context-sensitive rules for signal propagation (acropetal and basipetal). [K] — [Prusinkiewicz & Lindenmayer 1990, The Algorithmic Beauty of Plants](http://algorithmicbotany.org/papers/#abop)
- L-PEACH stores the tree as an L-system string. Carbon transport is solved by "folding" the network from the tips to the base (reducing series and parallel resistances, as in an electric circuit), then "unfolding" back to the tips to get node concentrations and fluxes. Nonlinearity is handled with "an L-system implementation of the Newton–Raphson method". [F] — [Allen, Prusinkiewicz & DeJong 2005, New Phytol 166:869-880](https://www.algorithmicbotany.org/papers/lpeach.nphyto2005.pdf)
- GroIMP/XL: relational growth grammars, i.e. graph rewriting over a typed graph with successor and branch edges, plus queries over the whole graph. [K] — [Hemmerling et al. 2008, Funct Plant Biol 35:739-750](https://doi.org/10.1071/FP08052)
- Helios plant architecture has a hierarchy of plant → shoots → phytomers.
  - Shoot types are species-specific labels; the documentation gives bean's `unifoliate` and `trifoliate`.
  - Each phytomer holds an internode, petiole(s) with leaflets, a peduncle and an inflorescence.
  - Buds are vegetative or floral and have a `BudState`. "Killed buds are marked dead rather than removed".
  - Shoots form a tree that can be queried by parent, child, rank and depth.
  - `advanceTime()` sub-steps at most one shortest phyllochron.
  - The library includes `bean`, `cowpea` and `soybean`.
  - [F] — [PyHelios PlantArchitecture docs](https://plantsimulationlab.github.io/PyHelios/PlantArchitectureDoc.html)
- CPlantBox (from CRootBox) is an organ-based whole-plant framework with roots, stems and leaves. Its phloem module couples to PiafMunch. [K for the details; title F] — [Zhou et al. 2020, in silico Plants 2:diaa001](https://doi.org/10.1093/insilicoplants/diaa001); [CPlantBox: fully coupled platform (Giraud et al. 2023)](https://research.dial.uclouvain.be/handle/2078.5/216298)

### Inferences
- **Adopt from MTG:**
  - Store an explicit edge type (successor or branch) per node, plus an axis id and axis order. Our "axis order" field already implies this.
  - Make "an axis dies when its basal internode dies" a cheap subtree operation.
  - Index nodes per axis so that apical-dominance distance along the axis is trivial to compute.
  - Keep the organ graph at one scale, and treat phytomer and axis as derived groupings. Helios uses an explicit shoot → phytomer hierarchy; we can derive it.
- **Adopt from Helios:** mark dead organs as tombstones (keep their ids) rather than deleting them. This keeps SoA indices and the action log stable, which suits determinism.
- **Transport cost:** L-PEACH's fold/unfold is O(n) Gaussian elimination on a tree. A tree-structured linear system can be solved exactly in linear time, and this matters for section 3.
- L-systems and graph grammars bring little for a Rust SoA engine beyond conceptual clarity. They help authoring, not runtime. Our trait-based architecture modules play the role of production rules.

### Gaps
- I did not fetch the MTG, L-Py or XL papers in this session; their claims are from prior knowledge.

---

## 3. Carbon transport models: discretisation, numerical stability, conductances; is one explicit pass per hour reasonable?

### Takeaway
The established hierarchy has three levels:
- **Transport-resistance:** flux = ΔC / resistance (Thornley; Minchin et al.; L-PEACH).
- **Münch / osmotic-pressure flow coupled to xylem water:** PiafMunch (Daudet, Lacointe, Minchin), CPlantBox.
- **Empirical distance or threshold rules:** QualiTree; Helios' threshold transfer.

Mechanistic models solve the coupled network implicitly: Newton-Raphson in L-PEACH, stiff ODE solvers in PiafMunch. None uses a single explicit relaxation sweep. For a bean-sized plant at hourly steps, an explicit scheme can work, but only with a flux limiter tied to equilibration and a Jacobi (simultaneous) update. An implicit O(n) tree solve is cheap and removes the stability risk entirely.

### Cited Findings
- L-PEACH:
  - Transport follows Thornley & Johnson (1990) and Minchin et al. (1993). Flux is "inversely proportional to a resistance to transport".
  - Sinks take carbon as i = f_a(v) · f_b(q) · f_c(w), where f_a is Michaelis–Menten in local concentration, f_b is a growth cap as the organ nears its mature size, and f_c is water stress.
  - Leaves grow to a set maximum size; fruit has a dynamic target; roots are open-ended sinks.
  - Storage acts as a sink until the ratio of non-structural to structural carbon reaches a user-set maximum, with sink strength falling as the cap approaches.
  - If supply is short for growth or maintenance, "organs (fruits, leaves or branches) are shed".
  - Time step is user-defined (e.g. days), with Newton-Raphson iteration to convergence every step.
  - The paper gives no numeric resistances, Km values or rates ("more data will be required before the model is fully calibrated").
  - [F] — [Allen et al. 2005](https://www.algorithmicbotany.org/papers/lpeach.nphyto2005.pdf)
- PiafMunch (Lacointe & Minchin 2008) extends the modular approach of Daudet et al. (2002) to complex architectures. Coupled phloem-xylem Münch flow is solved with "recently developed numerical tools"; the paper cites the SUNDIALS suite (Hindmarsh et al. 2005), a stiff implicit ODE solver family. The key result: "sink priority is an emergent property of the Münch flow". Source-leaf transpiration can also shift allocation between sinks. [F] — [Lacointe & Minchin 2008, Funct Plant Biol 35:772-780](https://publish.csiro.au/FP/FP08085)
- **Bean-specific:** PiafMunch was tested on a heavily pruned dwarf bean (*Phaseolus vulgaris*) with ¹¹C labelling, as a 2-source-3-sink system. It predicted the responses to source and sink manipulation. However, tracer from either labelled leaf always reached both the apex and the root. A single phloem path cannot explain this, which points to bidirectional flow in the stem; a slightly more complex network captured it. [F] — [Thorpe, Lacointe & Minchin 2011, Funct Plant Biol 38:127-138, doi:10.1071/FP10156](https://connectsci.au/fp/article/38/2/127/54249/Modelling-phloem-transport-within-a-pruned-dwarf)
- Minchin, Thorpe & Farrar (1993): a transport-resistance model with Michaelis–Menten unloading at sinks explains sink priority. When supply is limited, the sink with the lower Km keeps importing. [K] — [J Exp Bot 44:947-955](https://doi.org/10.1093/jxb/44.5.947)
- Minchin & Lacointe (2005) review the consequences of phloem physiology for modelling long-distance carbon transport. [K/title F] — [New Phytol 166:771-779](https://doi.org/10.1111/j.1469-8137.2005.01323.x)
- Thornley's transport-resistance model (root:shoot partitioning from C and N substrate gradients and resistances). [K] — [Thornley 1972, Ann Bot 36:861-871](https://doi.org/10.1093/oxfordjournals.aob.a084646)
- Daudet et al. 2002: generalised Münch coupling between sugar and water fluxes. [K] — [J Theor Biol 214:481-498](https://doi.org/10.1006/jtbi.2001.2473)
- Helios carbohydrate model (defaults from the API reference):
  - `carbon_conductance_down` = 0.95; `carbon_conductance_up` = 0.475 (half of down). Units are not stated.
  - `carbohydrate_transfer_threshold_down` = 0.025; `carbohydrate_transfer_threshold_up` = 0.04.
  - `starch_sequestration_ratio` = 0.025 g C g⁻¹ DW, described as the stem concentration threshold for transferring carbon to child shoots.
  - [F] — [Helios CarbohydrateParameters](https://plantsimulationlab.github.io/Helios/struct_carbohydrate_parameters.html)
- CPlantBox's phloem module couples PiafMunch to CPlantBox architecture, and has been used to show that phloem anatomy can restrict root-system development. [title F, details K] — [Phloem anatomy restricts root system architecture development](https://research.dial.uclouvain.be/handle/2078.5/216294); [Zhou et al. 2020](https://doi.org/10.1093/insilicoplants/diaa001)
- QualiTree (peach) splits the tree into compartments. Exchange between them depends on assimilate availability and on a distance-dependent proximity term between sources and sinks. [K, specifics to verify] — [Lescourret et al. 2011, Trees 25:519-530](https://doi.org/10.1007/s00468-010-0531-9)
- LIGNUM is built from tree segments, branching points and buds. It uses a carbon budget, the pipe model for thickening, and an annual time step; shoot growth is iterated "until carbohydrates available for growth are used up". This is effectively a whole-tree pool each year, with light and position setting the relative extension of shoots. [F] — [Sievänen, Nikinmaa & Perttunen 1997, Silva Fennica 31:329-340](https://silvafennica.fi/pdf/5629); primary model papers: [Perttunen et al. 1996, Ann Bot 77:87-98](https://doi.org/10.1006/anbo.1996.0011) [K]

### Inferences (numerics; my analysis, not from sources)
- **How others discretise:** none of the mechanistic models found uses one explicit sweep. L-PEACH iterates Newton to convergence each step, and PiafMunch uses a stiff ODE solver. Our "one relaxation pass per hour, k(Ci−Cj)·dt" is a forward-Euler diffusion step on a graph. It is closest to Helios' threshold-and-conductance transfer, a game-oriented, empirical scheme.
- **Stability condition for our scheme:**
  - For one edge, C_i = R_i/W_i and flux F = k·(C_i−C_j)·dt.
  - The concentration difference after the step is multiplied by (1 − k·dt·(1/W_i + 1/W_j)).
  - It stays monotone (no overshoot) only if k·dt·(1/W_i + 1/W_j) ≤ 1, and is stable only if that quantity is ≤ 2.
  - At a node with many neighbours, under a simultaneous (Jacobi) update, the bound is roughly dt·Σ_j k_ij / W_i ≤ 1.
- **Where it breaks:** newly initiated organs have W → 0. With k proportional to cross-sectional area over length (not mass), k·dt/W_i becomes huge for a tiny new leaf, internode or root tip. The result is oscillation or overshoot. The plan's clamp ("transport alone never takes a reserve below zero") prevents negative reserves, but not overshoot past equilibrium, so concentrations can flip-flop hour to hour.
- **The problem is stiff:** time constants range from big structural organs (hypocotyl, mature leaves) to milligram buds.
- **Mitigations, from cheapest:**
  - (a) Limit each edge's flux to the equilibrating amount, (C_i − C_j) / (1/W_i + 1/W_j), not to zero reserve.
  - (b) Use an effective capacity W_i + W_min (e.g. a minimum of a few mg) when converting R to C.
  - (c) Compute every flux from start-of-step state, then apply them all (Jacobi). Sequential Gauss-Seidel sweeps are deterministic but depend on the order of the edge list, which creates directional bias.
  - (d) **Best option: implicit (backward-Euler) diffusion on the tree, solved exactly in O(n)** by leaf-to-root elimination and root-to-leaf back-substitution. This is the same fold/unfold that L-PEACH uses. It is unconditionally stable and monotone, keeps concentrations non-negative when sources are non-negative, conserves mass to round-off, is deterministic, and costs about the same as one explicit pass.
- **Sources and sinks inside the solve:** if sinks remain separate (step 12 after step 10), operator splitting is first-order accurate; at hourly dt that is acceptable.
- **The three-pool fallback (§3.9) may be unnecessary:** with option (d), the main instability risk that motivated it disappears. Keep the fallback, but try (d) before collapsing pools.
- **Is hourly fine for bean?** Measured phloem velocities are on the order of 0.1 to 1 m h⁻¹ [K, general phloem literature; I did not fetch a value]. A bean is 0.3 to 0.5 m tall, so real transport crosses the plant within about an hour. Hourly resolution is physically adequate. A single explicit pass moves carbon only one edge per hour, though, which makes long-path transport (leaf → petiole → node → several internodes → hypocotyl → root) artificially slow: about 10 to 20 edges is 10 to 20 hours of lag. **This is a real plausibility risk with one explicit pass. An implicit solve reaches the whole tree every step, so it avoids the lag.**
- **Negative reserves:** the plan lets R go briefly negative as a starvation signal. Under concentration-driven transport, a negative C draws carbon in from neighbours. That is physically odd but self-correcting. It is cleaner to keep R ≥ 0 and track the deficit (maintenance not paid) in a separate counter that drives starvation.
- **Conductance values:** no source found gives a ready-made conductance in g C h⁻¹ per (g C g⁻¹ DW). Mechanistic Münch models parameterise sieve-tube radius, sap viscosity and unloading Km instead. Helios' 0.95 and 0.475 look like dimensionless transfer fractions per step applied above thresholds (inference from names and defaults; the source code was not readable from this session).
- **Practical calibration route (inference):** choose k so that the time constant W/k for a mature leaf-to-stem edge is on the order of hours. A source leaf should export most of a day's assimilate within the night, which matches the known diurnal starch turnover pattern. Then set k ∝ area/length relative to that anchor.
- **Bidirectional flow:** the dwarf-bean finding of bidirectional stem flow (Thorpe et al. 2011) suggests a single concentration-diffusion stem is a simplification. A diffusion-like scheme also lets carbon move both ways depending on gradients; that is sufficient for a game.

### Gaps
- I did not obtain numeric parameter values from PiafMunch, CPlantBox or L-PEACH: sieve-tube conductivity, Km for unloading (often quoted around 10 to 100 mol m⁻³ in the Münch literature [K, unverified]), or sap viscosity. The HAL preprint of Lacointe & Minchin's 2019 methods chapter was blocked by bot protection.
- The Helios `CarbohydrateModel.cpp` source could not be read: the GitHub API was not enabled for this session. So the exact transfer algorithm, and whether it is per-shoot, explicit or threshold-based, is unconfirmed.
- I found no published stability analysis of explicit graph-diffusion carbon schemes in FSPMs. The stability bounds above are my derivation.

---

## 4. Priority-based vs proportional (relative sink strength) allocation

### Takeaway
In FSPMs and greenhouse crop models, the established practice is **proportional allocation by relative sink strength, with sink strength = potential growth rate**: GreenLab, TOMSIM, the Marcelis 1996 review. Strict priority hierarchies appear mainly in field crop models such as CROPGRO/BEANGRO, which fill seeds and pods first. In mechanistic transport models, priority *emerges* from unloading kinetics (lower Km wins under scarcity). Our plan multiplies demand by a priority factor; that is neither scheme, and it has a side effect that looks like a bug.

### Cited Findings
- Marcelis (1996): dry matter partitioning is mainly regulated by the sinks. Sink strength can be quantified as the potential growth rate (growth under non-limiting assimilate supply), and partitioning among sinks can be described by their relative sink strengths. [K] — [Marcelis 1996, J Exp Bot 47 (Special Issue):1281-1291](https://doi.org/10.1093/jxb/47.Special_Issue.1281)
- TOMSIM: partitioning among fruit trusses and vegetative units is proportional to their potential growth rates (relative sink strength), and was validated dynamically. [K] — [Heuvelink 1996, Ann Bot 77:71-80](https://doi.org/10.1006/anbo.1996.0009)
- GreenLab uses potential growth rates as estimates of relative sink strength and shares biomass "according to their relative demand". [F] — [Yan et al. 2004](https://consensus.app/papers/details/b29026a2d78b595288c3c0d065ff83dd/?utm_source=claude_desktop)
- Minchin et al. (1993) and Lacointe & Minchin (2008): sink priority emerges from Michaelis–Menten unloading or from Münch flow, rather than being imposed. [K]/[F] — [Lacointe & Minchin 2008](https://publish.csiro.au/FP/FP08085)
- L-PEACH sink uptake uses Michaelis–Menten kinetics in local concentration, multiplied by the growth-cap and water terms. [F] — [Allen et al. 2005](https://www.algorithmicbotany.org/papers/lpeach.nphyto2005.pdf)
- SUCROS-type models partition with fixed, development-stage-dependent coefficients. [K] — [Goudriaan & van Laar 1994, Modelling Potential Crop Growth Processes](https://doi.org/10.1007/978-94-011-0750-1). CROPGRO-Drybean (the successor of BEANGRO) allocates new assimilate to reproductive organs first (a priority rule) and the rest to vegetative organs by stage-dependent fractions. [K] — [Hoogenboom et al. 1994, BEANGRO, Agron J 86:182-190](https://doi.org/10.2134/agronj1994.00021962008600010032x)

### Inferences
- **Probable defect in §3.9 step 4.** "Realized growth = min(demand × priority, max(0, R − 0.05 W))" scales demand by priority even when carbon is plentiful.
  - A leaf at priority 0.7 never gets more than 70% of its potential.
  - If unmet demand is defined as demand − realized (step 5), every leaf and internode builds a 30% shortfall every hour.
  - The step-5 rule then shrinks its potential final size.
  - Result: non-reproductive organs can never reach the species' potential size, even in full sun.
  - Unless "unmet" is defined against demand × priority, this is a bug. Either way it is not established practice.
- **Recommended fix (inference, grounded in Minchin 1993 and L-PEACH):** draw = demand · C/(Km_type + C), or a piecewise-linear version, with organ-type Km. Seeds get a low Km (strong at low concentration) and reserve refill a high one. Under plenty, every organ gets about its full demand. Under scarcity, low-Km organs keep growing. This recovers "priority" as a property of competition, keeps the priority table's meaning, and fits local transport. The table converts directly to Km ranks.
- **Where competition happens:** with per-organ reserves, organs compete only through transport. A sink's draw lowers its local C, which steepens the gradient towards it. Proportional or relative-sink-strength behaviour then arises at plant level through the gradients, which is acceptable.
- **The root term** "0.6 + 0.4·(1 − FTSW)" is an encoded functional-balance rule, similar in spirit to Thornley's root:shoot model. It is fine to label it as such, as the plan does.

### Gaps
- I did not re-read Marcelis 1996 or Heuvelink 1996 in this session; PubMed was blocked. Their exact wording should be checked. I found no quantitative bean data on relative sink strengths of pods against leaves.

---

## 5. Reserves, non-structural carbohydrate thresholds, starvation and self-pruning

### Takeaway
Threshold-based starvation on local carbohydrate concentration, with a persistence window in days, is used in Helios. It is the closest precedent to our plan and gives concrete thresholds. Branch autonomy and correlative competition (Sprugel; Sachs & Novoplansky) back the idea that shaded branches die locally. Our 0.05 W reserve floor and 3-day window are in the same range as Helios defaults.

### Cited Findings
- Helios carbohydrate parameter defaults [F] — [Helios CarbohydrateParameters](https://plantsimulationlab.github.io/Helios/struct_carbohydrate_parameters.html):

  | Parameter | Default | Meaning |
  | --- | --- | --- |
  | `carbohydrate_abortion_threshold` | 0.1 g C g⁻¹ DW (stem) | Concentration below which a flowering bud may be aborted |
  | `bud_death_threshold_days` | 2 d | Days below the threshold before a bud dies |
  | `branch_death_threshold_days` | 5 d | Days below the threshold before a shoot dies |
  | `carbohydrate_pruning_threshold` | 0.025 g C g⁻¹ DW | Threshold for pruning a shoot |
  | `carbohydrate_phyllochron_threshold` | 0.05 | Below this, the phyllochron is slowed |
  | `carbohydrate_vegetative_break_threshold` | 0.05 | Below this, the probability of vegetative bud break falls |
  | `carbohydrate_growth_threshold` | 0.2 | Needed for radial growth |
  | `growth_respiration_fraction` | 0.211 | Share of growth carbon respired |
  | `r_m_w_20` | 5.25e-5 mol C mol C⁻¹ d⁻¹ | Stem maintenance respiration at 20 °C |
  | `r_m_r_20` | 5.25e-3 mol C mol C⁻¹ d⁻¹ | Root maintenance respiration at 20 °C |
  | `living_wood_fraction` | 0.5 | Share of wood that is alive |
  | `stem_carbon_percentage` | 0.457 | Carbon fraction of stem dry weight |
  | `leaf_total_carbon_percentage` | 0.453 | Carbon fraction of leaf dry weight |
  | `fruit_carbon_percentage` | 0.475 | Carbon fraction of fruit dry weight |
  | `total_flower_cost` | 8.33e-4 mol C flower⁻¹ | About 0.01 g C per flower |
  | `shoot_root_ratio` | 3 | Shoot internode to root dry weight |

  Note: the defaults look tuned for almond and other trees; they may not be bean-specific.
- With its carbohydrate model enabled, native Helios "prunes branches on carbon starvation instead of using the shedding process". [F] — [PyHelios docs](https://plantsimulationlab.github.io/PyHelios/PlantArchitectureDoc.html)
- L-PEACH caps storage by a maximum ratio of non-structural to structural carbon, with storage sink strength falling as the cap approaches. Organs are shed when supply cannot cover growth or maintenance. [F] — [Allen et al. 2005](https://www.algorithmicbotany.org/papers/lpeach.nphyto2005.pdf)
- LIGNUM handles foliage mortality and sapwood-to-heartwood conversion. Branch-whorl death is linked to heartwood formation, citing Kaipiainen & Hari 1985. [F] — [Sievänen et al. 1997](https://silvafennica.fi/pdf/5629)
- Branch autonomy: branches are largely carbon-autonomous once mature and do not import to survive shade, so shaded branches die. [K] — [Sprugel, Hinckley & Schaap 1991, Annu Rev Ecol Syst 22:309-334](https://doi.org/10.1146/annurev.es.22.110191.001521)
- Correlative competition: branch fate depends on relative, not absolute, performance among branches. [K] — [Sachs & Novoplansky 1995, Isr J Plant Sci 43:203-212](https://doi.org/10.1080/07929978.1995.10676605)
- Self-organising tree models allocate a resource ("vigour") by light-driven competition between branches, basipetally gathered and then acropetally distributed. [K] — [Palubicki et al. 2009, ACM TOG 28(3):58](https://doi.org/10.1145/1531326.1531364)

### Inferences
- **Floor:** our 0.05 W floor (in g C per g DW) sits between Helios' pruning threshold (0.025) and its bud-break and phyllochron thresholds (0.05). It is plausible.
- **Window:** a 3-day starvation window for bean leaves and branches sits between Helios' bud (2 d) and branch (5 d) windows. Plausible for a fast annual.
- **Recommendation:** adopt Helios' graded thresholds idea. Low concentration first slows initiation (phyllochron) and bud break, then aborts flowers, and only then kills shoots. That gives a graceful response chain rather than one cliff.
- **Correlative competition:** Sachs & Novoplansky's evidence implies that bud and branch fate should respond to *relative* local carbon or light versus the plant mean (e.g. C_local / C_plant). That is more robust to whole-plant scaling than absolute thresholds. Consider normalising the bud-activation input by the plant-level Q/D or mean C.
- **Leaf starvation rule:** the 30% resorption at death is reasonable for nitrogen. For structural carbon it is high: cell walls are not remobilised much; mostly NSC and protein are. A smaller figure for structural C (~5 to 10%) plus "all remaining reserve R returns to parent" may be more defensible. This is inference; I found no source value.
- **Diurnal storage:** Arabidopsis-style diurnal starch turnover means leaf reserves should buffer the night. The hourly per-organ reserve does this naturally. This is an advantage over daily-step models.

### Gaps
- I found no bean-specific values for NSC concentrations or starvation durations (e.g. starch and sugar in leaves and stems, in g per g DW, over the day).

---

## 6. Helios (Bailey) framework

### Takeaway
Helios is an open-source C++/Python framework for virtual plants and canopies. It has a procedural plant-architecture plugin (plant → shoot → phytomer; library includes bean, cowpea and soybean), GPU ray-traced radiation, photosynthesis, stomatal conductance and energy balance plugins, and an optional carbohydrate model with per-shoot thresholds and up/down conductances. It is the closest existing analogue to our plan, and a good source of parameter defaults and a cross-check.

### Cited Findings
- "An open-source C++ and Python framework for building virtual plants, canopies, and landscapes", licensed LGPL-2.1+. Radiation uses GPU ray tracing (Vulkan or OptiX); capabilities include photosynthesis, stomatal conductance and energy balance/ET. [F] — [Helios GitHub](https://www.github.com/PlantSimulationLab/Helios)
- PlantArchitecture: shoot types per species, with bean `unifoliate`/`trifoliate`. Phenology runs dormant → vegetative → reproductive → senescent by thresholds. `advanceTime` sub-steps at most one phyllochron. Library species include `bean`, `cowpea` and `soybean`. With the carbohydrate model enabled, branches are pruned on carbon starvation. [F] — [PyHelios PlantArchitecture docs](https://plantsimulationlab.github.io/PyHelios/PlantArchitectureDoc.html)
- Carbohydrate parameter defaults: see section 5 [F] — [Helios CarbohydrateParameters](https://plantsimulationlab.github.io/Helios/struct_carbohydrate_parameters.html)
- Original Helios paper. [K] — [Bailey 2019, Ecol Model 2019](https://doi.org/10.1016/j.ecolmodel.2019.01.020)

### Inferences
- **Use Helios as a reference:** its bean library model gives concrete phytomer and shoot parameters (internode lengths, leaf angles, phyllochron) for visual plausibility.
- **What we add:** its carbohydrate model shows a light, threshold-based scheme is accepted practice in a research framework, which supports our approach. Our plan differs in hourly Farquhar–Medlyn per light class against Helios' per-primitive ray tracing, and in the turbid-medium voxel light model.
- **Licence:** LGPL means we can read Helios code for parameter values, but must not copy code into the Rust engine without considering the licence (inference).

### Gaps
- Bean library parameter values and the carbohydrate algorithm source were not read (GitHub access not enabled for this repo in this session).

---

## 7. Beta growth function (Yin et al. 2003), and ADEL-wheat/maize organ expansion

### Takeaway
Yin et al.'s beta growth function has three parameters (w_max, t_m, t_e). It gives zero rate at start and end and a defined end of growth, and it fits as well as logistic, Richards or Gompertz. It suits our "birth time + expansion curve" design, but it assumes w(0) = 0, so seed-born organs need the offset form. GreenLab's beta law is a different parameterisation (a, b shape over a duration T) of the same idea.

### Cited Findings
- Yin, Goudriaan, Lantinga, Vos & Spiertz 2003 [F] — [Ann Bot 91:361-371, doi:10.1093/aob/mcg029](https://pmc.ncbi.nlm.nih.gov/articles/PMC4244967):
  - Equation: w = w_max · (1 + (t_e − t)/(t_e − t_m)) · (t/t_e)^(t_e/(t_e − t_m)) for 0 ≤ t ≤ t_e; w = w_max for t > t_e.
  - The rate is zero at both ends. The maximum rate c_m occurs at t_m and is computed from the parameters, not fitted. The symmetric case (t_m = t_e/2) is a cubic polynomial.
  - Relative growth rate is infinite at t = 0, so the function is not suitable for estimating early exponential relative growth rate. An extension (eqn 11) handles nonzero initial weight.
  - Fitted values:

    | Dataset | w_max | t_m | t_e | Other |
    | --- | --- | --- | --- | --- |
    | Wheat grain | 39.7 to 52.2 mg | 9.8 to 19.9 d | 21.6 to 43.0 d | c_m 1.72 to 2.88 mg d⁻¹ |
    | Maize plant | 123.5 g | 89.0 d | 108.5 d | |
    | Pea crop | 1213 g m⁻² | 936 °Cd | 1385 °Cd | |

  - Compared with the alternatives: logistic and Gompertz overestimated w_max (maize: logistic 134.5, Gompertz 156.7, against beta 123.5), and Richards' shape parameter rarely differed from 1 and was numerically unstable.
- GreenLab and ADEL use beta or thermal-time expansion curves. ADEL-maize models leaf extension in thermal time, with blade and sheath growth coordinated by leaf-tip and collar emergence events. [K] — [Fournier & Andrieu 1998, Ann Bot 81:233-250](https://doi.org/10.1006/anbo.1997.0549); ADEL-wheat [K] — [Fournier et al. 2003, PMA03](https://hal.inrae.fr/hal-02832000)

### Inferences
- **Adopt Yin's form**, with t_m and t_e in °C·day per organ type.
  - Derived c_m gives the peak hourly demand, which helps when sizing conductances.
  - Use the rate form directly for hourly demand: dw/dt = c_m · ((t_e − t)/(t_e − t_m)) · (t/t_m)^(t_m/(t_e − t_m)) (Yin's rate form; verify against eqn 9 and the appendix before coding).
  - Evaluate analytically, which suits determinism and scrubbing.
- **Initial size:** for a leaf born as a primordium with nonzero mass, use the offset form or start at a small w_0. Otherwise relative growth rate at t → 0 is infinite, which interacts badly with any relative-rate cap or with C = R/W.
- **Final-size reduction:** "potential final size × (1 − unmet / cumulative potential)" (§3.9 step 5) both shrinks the target and feeds back into future demand, so a shortfall compounds. GreenLab and TOMSIM take a simpler path: an organ's size is just the integral of what it received over a fixed expansion window, with no catch-up after t_e. This gives "no catch-up" with less bookkeeping and no compounding. Low risk either way, but simplify (inference).
- **Is "no catch-up" right?** GreenLab reproduced "compensatory growth following pruning" through more carbon going to organs that are still expanding. Our scheme allows the same as long as compensation affects only currently expanding or future organs. That is consistent with the literature.

### Gaps
- No fitted t_m and t_e values for bean leaves, internodes or pods were found in this session.

---

## 8. FSPMs of common bean or soybean

### Takeaway
Bean and soybean FSPMs are few. They include: a 2025 climbing-bean FSPM (GroIMP-type) for maize/bean polycultures; Helios library models for bean, cowpea and soybean; a soybean root and nodulation L-system model; and a PiafMunch phloem study on dwarf bean. I found no GreenLab bean calibration and no full source-sink FSPM of bush bean, so our model would be novel in that respect. The crop-level BEANGRO/CROPGRO-Drybean is the standard source for bean phenology and partitioning parameters.

### Cited Findings
- Climbing common bean FSPM, with twining and physical plant-plant interaction, used for maize/bean polycultures. Sensitivity analysis identified the parameters most influential on leaf area index and aboveground biomass. The model predicted up to 60% overyielding of total peak biomass under Mexican conditions and little bean growth under Dutch or French conditions. [F] — [Rutjens et al. 2025, in silico Plants](https://consensus.app/papers/details/6b15f9dc26685c30a62155fcc74ed1ac/?utm_source=claude_desktop)
- Soybean root architecture with nodulation and autoregulation-of-nodulation signalling, built in L-studio, with a synchronisation algorithm for multi-rate signalling and development. [F] — [Han et al. 2011, Ann Bot](https://consensus.app/papers/details/40a05cf8e8195d4f94bcc14522612e52/?utm_source=claude_desktop)
- Dwarf bean phloem (PiafMunch, ¹¹C): see section 3. [F] — [Thorpe et al. 2011](https://connectsci.au/fp/article/38/2/127/54249/Modelling-phloem-transport-within-a-pruned-dwarf)
- Helios library includes `bean`, `cowpea` and `soybean`. [F] — [PyHelios docs](https://plantsimulationlab.github.io/PyHelios/PlantArchitectureDoc.html)
- BEANGRO, a process-oriented dry bean crop model, is now CROPGRO-Drybean in DSSAT. [K] — [Hoogenboom et al. 1994](https://doi.org/10.2134/agronj1994.00021962008600010032x)
- General FSPM framing: architecture part (topology, coordinated expansion, geometry) and process part. [F] — [Vos et al. 2010, J Exp Bot](https://consensus.app/papers/details/a479e2f8ab325317adb7e1ad2266e18d/?utm_source=claude_desktop); [Godin & Sinoquet 2005, New Phytol](https://consensus.app/papers/details/7fe81cd565915bf4a64c71d353530d4b/?utm_source=claude_desktop)
- FSPM-P: a generic, object-oriented FSPM for annuals with organ-level objects (GroIMP). [F] — [Henke et al. 2016](https://consensus.app/papers/details/bcf53a9943435c3181daaba009288690/?utm_source=claude_desktop)

### Inferences
- **Source of bean parameters:** CROPGRO-Drybean species and cultivar files (DSSAT, open source) are the most practical source for bean phenology, specific leaf area, partitioning and pod/seed parameters to check §3.16 against. A dedicated bean-parameters thread should pull these.
- **Bean phloem behaviour:** the Thorpe et al. 2011 result (bidirectional stem flow; Münch model predicts the response to source and sink manipulation in bean) supports our choice to make transport local for bean specifically.

### Gaps
- I could not confirm a soybean shoot FSPM with carbon allocation (e.g. GroIMP or OpenAlea soybean models) in this session's searches; one may exist.
- I did not find the Rutjens et al. 2025 parameter tables (internode and leaf expansion, phyllochron for bean).

---

## Summary table: plan v3 against established practice (all judgments are inference)

| Plan element | Established practice | Divergence | Risk |
| --- | --- | --- | --- |
| Carbon as the only authority; architecture decides where growth goes (§3.1) | LIGNUM, L-PEACH, GreenLab, Helios | None | Low |
| Per-organ reserves with conductance transport (§3.9) | GreenLab: common pool; L-PEACH and PiafMunch: local with mechanistic transport; Helios: per-shoot thresholds | Middle ground; justified by the self-pruning goal | Medium (calibration); mitigate with a common-pool mode switch |
| One explicit relaxation pass per hour | Implicit or Newton (L-PEACH); stiff ODE solver (PiafMunch) | Numerically weaker; one edge per hour causes transport lag along paths | **High**, unless limited by equilibration or replaced with an O(n) implicit tree solve |
| Priority multiplies demand (step 4) | Relative sink strength (proportional), or emergent priority via Michaelis–Menten Km | Caps non-limited growth at the priority fraction; feeds false unmet demand | **High (likely bug)**; switch to a C/(Km + C) draw |
| No catch-up; final size shrinks with unmet/potential | GreenLab and TOMSIM: size = integral received over a fixed window | Extra feedback can compound | Low to medium; simplify |
| Beta expansion curves in thermal time | Yin 2003, GreenLab, ADEL | None | Low; handle w(0) > 0 |
| 0.05 W reserve floor, 3-day starvation window | Helios: 0.025 to 0.1 g C g⁻¹ DW thresholds, 2 to 5 day windows | Same range | Low |
| Encoded root functional-balance priority | Thornley root:shoot; CROPGRO stress partitioning | Common practice | Low |
| 30% structural C resorbed at death | Remobilisation is mostly NSC and N, not cell wall | Probably high for C | Low to medium |
| MTG-like SoA graph | MTG / OpenAlea, Helios shoot/phytomer, GroIMP graphs | Consistent; add edge type and axis id; keep dead organs as tombstones | Low |
