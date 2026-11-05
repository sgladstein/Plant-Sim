# Procedural plant generation, visualization and plant games: what to adopt

Scope note: about 20 tool calls. I fetched and read primary sources for Palubicki 2009, GPU Gems 3 ch. 16, Hädrich 2017, Prusinkiewicz's L-system overview, the Stava 2014 abstract, the Synthetic Silviculture project page, two Bevy crates, and the Viridi and Equilinox store and devlog pages. Claims tagged **[prior knowledge, unverified this session]** come from training knowledge. The citation shows the paper exists, but I did not re-read the paper this session. The report writer should present those as softer claims.

## 1. Tree and plant development models in graphics: strengths, and is "no space-colonization markers" justified?

### Takeaway
Yes, avoiding markers is justified. Palubicki et al. 2009 present shadow propagation as a peer alternative to space-colonization markers. They also say it handles shedding better, and that it runs about 3× faster than space colonization for trees of similar size. Our turbid-medium light grid plays the same role as their shadow voxels, but more physically. Bud-fate decisions driven by a light signal and an extended Borchert-Honda-style allocation are the parts to adopt. Markers suit artist-driven crown shaping, which we don't need.

### Cited Findings
- **Palubicki et al. 2009, "Self-organizing tree models for image synthesis" (SIGGRAPH / TOG 28(3))**:
  - Buds are the only sites of new growth and compete for space or light. Each iteration, the environment gives each bud an amount Q and a preferred direction V. Resources then set how many metamers each bud makes and how long its internodes are. Tropisms bend the shoots, and shed branches are removed. — [algorithmicbotany.org PDF](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf); [project page](https://www.algorithmicbotany.org/papers/selforg.sig2009.html)
  - The paper offers two environmental inputs as alternatives: space colonization (markers, perception angle ~90°, perception distance 4–6 internode lengths) and shadow propagation. Its words: "As an alternative to space colonization, we use a method proposed by Pałubicki [2007]." In shadow propagation each bud casts a pyramidal penumbra into a voxel grid (typically 200³ voxels, shadow depth 4–8 layers). — [PDF](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf)
  - Space colonization is about 3× slower than shadow propagation for trees of similar size. Shedding works poorly with binary marker input, because branches would be shed "immediately after they stop growing". The authors say shedding is "more suitable for models that rely on shadow propagation rather than space colonization." — [PDF](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf)
  - Extended Borchert-Honda allocation works in two passes. A basipetal pass accumulates light Q in the internodes. An acropetal pass then splits a growth resource at each branch point using λ, where λ > 0.5 favors the main axis. λ from 0.46 to 0.54 sweeps the tree from decurrent to excurrent habit. An alternative priority model ranks branches by light per bud, with the terminal bud first (apical control). — [PDF](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf)
  - Pipe model: branch diameter comes from a basipetal pass, d^n = d1^n + d2^n with n between 2 and 3. Widths are not reduced when branches are shed, so the model needs memory of past structure. — [PDF](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf)
  - Limitations the authors state: branches never reorient or bend under weight, there is no objective validation against measured trees, and large trees are costly (about 700k internodes took 82 s). — [PDF](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf)
- **Runions, Lane & Prusinkiewicz 2007, "Modeling Trees with a Space Colonization Algorithm"** (Eurographics Workshop on Natural Phenomena). The method fills a crown volume with attraction points that guide branch growth, so the user controls the crown envelope directly. — [project page](https://www.algorithmicbotany.org/papers/colonization.egwnp2007.html); [EG diglib](https://diglib.eg.org/handle/10.2312/NPH.NPH07.063-070). It was later extended for artistic control and environmental interaction: [SciTePress 2014](https://www.scitepress.org/Papers/2014/46905/).
- **Makowski et al. 2019, "Synthetic Silviculture: Multi-scale Modeling of Plant Ecosystems"** (TOG / SIGGRAPH 2019). It models individual plants as 3D meshes inside interactive ecosystems of up to 500K plants. Growth, interaction, competition and several tropisms are simulated. It reuses self-similarity within and between plants for efficiency, and its parameter space covers nine plant ecologies. — [project page](https://storage.googleapis.com/pirk.io/projects/synthetic_silviculture/index.html); [bib](https://wp.faculty.wmi.amu.edu.pl/papers/Makowski.etal-2019-Synthetic-Silviculture.html). [prior knowledge, unverified this session] Each plant is built from reusable "modules" (prototype branch clusters) that are instanced and rescaled rather than simulated bud by bud. That is relevant to our M4 field and forest LOD.
- **Stava et al. 2014, "Inverse Procedural Modelling of Trees"** (Computer Graphics Forum). It introduces a new parametric tree-growth model and uses MCMC to fit its parameters so the output resembles an input tree. Inputs can come from interactive modellers, scans of real trees, or developmental models. — [EG diglib](https://diglib.eg.org/items/4c6abd53-959e-4336-af65-3eb719b9c1c3/full)
- **Pirk et al. 2012, "Plastic Trees: Interactive Self-Adapting Botanical Tree Models"** (TOG 31(4), DOI 10.1145/2185520.2185546). — [Purdue page](https://cs.purdue.edu/cgvlab/www/publications/Pirk12ToG); [Konstanz](https://graphics.uni-konstanz.de/publikationen/Pirk2012PlasticTreesInteractive/index.html). [prior knowledge, unverified] Existing tree models re-adapt interactively to new obstacles and light by re-bending branches and pruning, without regrowing.
- **Hädrich et al. 2017, "Interactive Modeling and Authoring of Climbing Plants"** (CGF / Eurographics 2017). Details are in §M2 below. — [PDF](https://www.cs.purdue.edu/homes/bbenes/papers/Haedrich2017EG.pdf)
- I did not fetch Yi et al. 2018 (tree growth modelling), Longay 2012 (TreeSketch), Pirk 2017 (wood combustion / dynamic trees) or Weber & Penn 1995 this session. [prior knowledge, unverified]
  - TreeSketch (Longay, Runions, Boudon, Prusinkiewicz, SBIM 2012) lets users steer space colonization plus Borchert-Honda growth with sketch strokes on a tablet.
  - Weber & Penn 1995 ("Creation and Rendering of Realistic Trees", SIGGRAPH) is a purely geometric, parameter-per-recursion-level generator. It underlies Blender Sapling and Arbaro. It has no environment feedback.
  - Pirk et al. 2017 ("Interactive Wood Combustion for Botanical Tree Models", SIGGRAPH Asia) couples a tree model to heat transfer, combustion and branch deformation.

### Inferences
- [inference] Our §3.5 light grid gives a per-leaf or per-voxel absorbed-PAR signal and a red:far-red proxy. That signal is a strict, more physical superset of Palubicki's shadow-voxel Q, so we do not need markers for "space". Markers would add a second, non-physical environment signal that would have to be reconciled with the light lens, which hurts scientific believability. Keep the plan as written.
- [inference] Gaining space-colonization-style crown filling without markers needs two things. First, occupancy: a bud whose voxel already holds high leaf-area density gets a low Q. Second, the preferred direction V as the light-weighted direction, which we already compute for phototropism. This is exactly Palubicki's shadow-propagation recipe.
- [inference] Adopt the extended Borchert-Honda λ as the heritable "apical dominance / habit" knob for the M4 tree. It is one interpretable parameter that sweeps the tree between habits, and it suits the breeding game. For the bean, our concentration-and-dominance bud-activation rule already plays this role.
- [inference] Pipe model: Palubicki notes that diameters must not shrink after shedding. Our cumulative pipe-model sink, which is mass-based and never decreases, satisfies this naturally. The data model should store cumulative wood, not recompute it from current leaves.
- [inference] Palubicki ignores branch bending. For believability, a cheap static-sag or Plastic-Trees-style bend from per-segment load is a visual-only addition we can add later. Keep it out of the sim so determinism is unaffected.
- [inference] Inverse modelling (Stava) suggests a "match this target tree" challenge mode: fit genotype parameters to a target silhouette. It also suggests a calibration tool that fits species parameters to photos or measurements. Both are low priority.

### Gaps
- I did not re-read the Plastic Trees, TreeSketch, Yi 2018, Pirk 2017 or Weber-Penn full texts this session, so their details are tagged unverified.
- I found no published validation comparing Palubicki-style models to measured tree architecture. The paper itself lists this as a limitation.

## 2. Leaf and stem geometry; real-time rendering (instancing, growth skinning, LOD, impostors, wind, translucency)

### Takeaway
Use generalized-cylinder tubes for stems and roots, instanced leaf templates with a per-instance growth and droop parameter, Crysis-style vertex-color wind (cheap, vertex-shader only), and a wrapped-diffuse back-lighting term for leaf translucency. All are well-proven at GTX 1070 class and fit Bevy's material system.

### Cited Findings
- **GPU Gems 3 ch. 16, "Vegetation Procedural Animation and Shading in Crysis" (Sousa, 2007)**:
  - Main bending: offset vertex xy along the wind's xy direction, scaled by normalized height. Then renormalize the vertex to its original distance from the mesh center, which keeps the motion spherically limited and artifact-free. — [NVIDIA](https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch16.html)
  - Detail bending is driven by vertex colors: R = leaf edge stiffness, G = per-leaf phase, B = overall leaf stiffness, A = baked AO. — [NVIDIA](https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch16.html)
  - Phase sums an object phase (from world position), a branch phase and a vertex phase. It uses four smoothed triangle waves (x²(3−2x)) instead of sines, at frequencies 1.975, 0.793, 0.375 and 0.193. — [NVIDIA](https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch16.html)
  - Translucency: double-sided leaves use alpha test. A low-res (as small as 128²) artist subsurface thickness map encodes veins. Back diffuse is wrapped N·L (dot × 0.6 + 0.4), blended with a view-dependent E·L term and multiplied by a back color and the thickness map. Excess bending breaks believability, so amplitudes are capped. — [NVIDIA](https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch16.html)
- **Generalized cylinders** were added to L-system modelling for "smoothly curving branches". Leaves and petals are predefined surfaces or surfaces bounded by turtle paths. Compound leaves can be implicit contours around a skeleton. "Developmental surfaces" let organ shape change over time in animations. — [Prusinkiewicz, "A look at the visual modeling of plants using L-systems"](https://www.algorithmicbotany.org/papers/a-look-at-the-visual-modeling-of-plants-using-l-systems.pdf)
- [prior knowledge, unverified] Habel, Kusternig & Wimmer 2009, "Physically Guided Animation of Trees" (Eurographics 2009), drives branch motion from a stochastic wind spectrum. It uses precomputed modal analysis per branch level and evaluates it in a vertex shader. It is more physical than Crysis, but needs a hierarchical branch-deformation pass.
- [prior knowledge, unverified] Habel, Kusternig & Wimmer 2007, "Physically Based Real-Time Translucency for Leaves" (EGSR 2007), uses half-life-2-basis precomputed translucency. It is overkill for a stylized look.

### Inferences
- [inference] Stems: build one tube mesh per shoot axis as a generalized cylinder along the internode chain. Use parallel-transport frames (no twist flips), a radius from sim allometry, 6–8 radial segments for the bean, and arc-length UVs. The plan's rule "rebuild when size changes > 2%" is fine. Better still, make the per-vertex radius and length a function of a uniform `t` (fractional growth) so most frames need no rebuild. Encode each vertex's internode id and its fraction along the internode, and let a storage buffer of per-internode (length, radius, orientation) values drive it in the vertex shader. This is "GPU skinning for growth": topology changes only when organs are born.
- [inference] Leaves: author 3–4 templates (cotyledon, unifoliate, trifoliate leaflet, pod) as small meshes. Give each about 30–80 triangles with a midrib crease and a gentle cupping curvature, so lighting reads well without textures. Per-instance data: transform, expansion fraction, droop or wilt angle, color ramp index (senescence/N), lens value. Wilt droop works best as a bend about the petiole/pulvinus pivot plus a fold about the midrib, both done in the vertex shader. The pulvinus is the natural hinge for solar tracking and nyctinastic "sleep" movements in beans; a cheap, charming detail that is also real botany. [The bean-pulvinus botany claim is prior knowledge, not sourced here.]
- [inference] Wind: adopt the Crysis scheme nearly verbatim. Bake the vertex colors into the templates (edge stiffness, phase) and compute the main bend per plant from height above the soil. The cost stays constant per vertex. It must be render-only, as the plan already says. Phase should come from entity id, not RNG, to keep replays identical visually.
- [inference] Translucency: the Crysis wrapped back-diffuse term plus a rim light matches the "smooth ramp + rim light" stylized target. Drop the subsurface texture and use the leaf-blade vertex color for vein darkening. For a GTX 1070 with 30 plants × 300 leaves (about 9k instances, under 1M triangles) this is trivially within budget.
- [inference] LOD: none is needed until M4. At M4, use Synthetic-Silviculture-style module instancing for mid-distance and octahedral impostors at far distance.

### Gaps
- I did not verify Bevy's current support for custom vertex-shader instancing with storage buffers in the latest version, beyond the crates in §6.

## 3. Growth animation, time-lapse, and smooth topology changes

### Takeaway
Our §3.13 plan of organs as functions of time with beta growth curves is the established solution. It is the dL-system idea from Prusinkiewicz, Hammel & Mjolsness 1993: discrete births, continuous development. It gives pop-free time-lapse as long as every new organ is born at size ≈ 0 and every removal animates out.

### Cited Findings
- Prusinkiewicz, Hammel & Mjolsness 1993, "Animation of plant development" (SIGGRAPH 93). It presents a combined discrete/continuous model (L-system productions plus piecewise-continuous differential equations) for animation "in a manner resembling time-lapse photography", shown on Campanula, Lychnis and Hieracium. — [algorithmicbotany.org](https://algorithmicbotany.org/papers/animdev.sig93.html); [SIGGRAPH history](https://history.siggraph.org/?p=118627)
- In differential L-systems, modules are created and removed in discrete events but "develop in a continuous fashion described by differential equations". The equations can read neighbors, which allows signalling. — [Prusinkiewicz overview PDF](https://www.algorithmicbotany.org/papers/a-look-at-the-visual-modeling-of-plants-using-l-systems.pdf); see also [Hammel thesis, U Calgary](https://prism.ucalgary.ca/items/da610020-c00f-4f89-bdfb-e04645fb830d/full)

### Inferences
- [inference] Rules for pop-free rendering of topology changes:
  1. Every organ is born at zero size and grows by its expansion curve. The bud is visible as a tiny bump before it activates.
  2. Death and abscission animate over a short render-only window: the leaf yellows, rotates down at the petiole, then detaches and falls as a particle or ghost with a fade. The sim drops the organ at the tick, and the renderer keeps a "dying" ghost for N frames.
  3. Pruning plays a short cut animation.
  4. When a stem is re-tubed because of branching, keep the parent axis vertices stable and only append, which works because each axis has its own tube.
- [inference] The replay scrubber can evaluate geometry at any fractional time from the snapshot plus the curves. This is the dL-system property applied to scrubbing.
- [inference] Show a timeline with phenological stage bands (V/R stages) and event markers (flowering, first pod). L-studio-style animated developmental sequences are the visual reference for the time-lapse capture feature.

### Gaps
- I did not fetch the 1993 paper's full text, so how it handles organ-birth smoothing in practice is unverified.

## 4. Root visualization and soil cutaway

### Takeaway
I found little prior game or edu work on rendering roots. The rhizobox and glass-front metaphor in our plan is distinctive. [inference] The technical approach is a clipping-plane cutaway plus a soil shader with fading opacity. Both are standard.

### Cited Findings
- No dedicated sources were found this session for root rendering in games. CRootBox, cited in our plan, comes from a separate science thread.

### Inferences
- [inference] Implement the slab as a box with an opaque soil material on five faces, plus a "glass" front face that is a slightly tinted, refractive-looking transparent quad. Behind it, add a cut-face soil material that shows texture layers and the soil-water lens as a 3D texture sampled at the cut plane.
- [inference] For X-ray mode, render roots after soil and lower the soil opacity. Add a depth-fade or fresnel outline so 3D depth stays readable, plus an optional "roots in front of glass plane" emphasis.
- [inference] Roots as thin tubes: thousands of segments at 4–6 sides each. Batch all root segments of a plant into one mesh, rebuilt daily, since root growth is updated daily anyway.
- [inference] Add a subtle soil-particle dust and moisture darkening on the cut face when watering. It shows the water front moving down, a "designed moment" the plan does not yet list but which fits it.

### Gaps
- I found no sourced survey of root visualization in educational tools such as RootBox viewers or OpenSimRoot visualizers. This needs a dedicated search.

## 5. Plant and garden games: what made them fun or believable; lessons from science sims

### Takeaway
Two working patterns exist: ambient companionship (Viridi) and ecosystem building with unlocks (Equilinox). Neither simulates physiology. Our differentiator is "emergent form from real rules, made visible". [inference] The lessons from Kerbal and Universe Sandbox are: let failure be spectacular and legible, give a sandbox before goals, and make the real model the toy.

### Cited Findings
- **Viridi** (Ice Water Games). "Nurture a small pot of succulents that grow in real time." "Leave Viridi open in a second window to keep you company while you work." Succulents "can die from over- or under-watering", and with special care "might even flower for you". It has a nursery and a weekly free seed. — [Steam](https://store.steampowered.com/app/375950?l=english)
- **Equilinox** (ThinMatrix, 2018) is "a nature sandbox game", likened to a city-builder for ecosystems. Placing plants makes terrain fertile. Animals earn Diversity Points, spent in shops. Tasks unlock content. You evolve plants and animals along evolution trees, and selective breeding and genetic modification change traits (e.g. rare flower color). Each species has requirements and a life cycle, and unhealthy animals can get sick and infect others. — [ModDB devlog](https://www.moddb.com/games/equilinox/); [GamingOnLinux](https://www.gamingonlinux.com/2019/01/equilinox-is-a-rather-beautiful-and-peaceful-nature-sim-available-on-linux)
- [prior knowledge, unverified]
  - **Terra Nil** (Free Lives, 2023) is a "reverse city builder" that restores ecosystems and then removes its own buildings. The satisfaction comes from watching barren land turn green, with clear per-biome goals.
  - **Grow Home** (Ubisoft Reflections, 2015) has a procedural star-plant you climb and steer.
  - **Kerbal Space Program** makes a real physics model the toy. Failure is funny and informative, and mastery comes from understanding orbits, not from memorizing recipes.
  - **Universe Sandbox** lets users break things at scale with real gravity and shows quantities.
  - **Stardew Valley** crops are fixed-day stage sprites with no environment response. They are the "fun but not believable" pole.

### Inferences
- [inference] Adopt from Viridi: an "ambient window" mode at 1 sim-hour per second or slower, with a minimal UI, for idle companionship. Make care errors (overwatering) understandable, not punishing.
- [inference] Adopt from Equilinox: unlocks tied to discovery. A new seed or species becomes available when you breed a trait or meet an experiment card. Visible breeding outcomes such as flower or seed coat color are strong hooks. Bean seed-coat color and pattern is a real, highly visible Mendelian trait set and an ideal M3 "rare color" collectible. [The bean genetics claim is prior knowledge, not sourced here.]
- [inference] Adopt from Kerbal-style design: an inspector "why" line (already in the plan) plus a replay to the moment things went wrong (the plan has a scrubber). Failure becomes a story: "this leaf starved on day 23 because…". Add a "flag this moment" bookmark on the timeline.
- [inference] Contrast with Stardew: the plan's side-by-side pots are the anti-Stardew demo. The same seed diverges visibly, proving the plant isn't scripted. Lead marketing and onboarding with it.
- [inference] Terra Nil's lesson: a clear end state and a "before and after" reveal. A per-run time-lapse export of a season, from seed to senescence, is the shareable payoff even without web distribution.

### Gaps
- Not covered: SimPark, SimLife, Spore, Plant Tycoon, Kynseed, Grow: Song of the Evertree, bonsai games, Botanicula, GDC postmortems on growth sims. I found no sourced postmortem on plant-growth game design this session.

## 6. Rust/Bevy ecosystem and GTX 1070 performance

### Takeaway
Two maintained MIT-licensed Bevy crates exist: `bevy_symbios` (L-system skeletons to tube meshes) and `bevy_procedural_tree`. They are useful references for tube meshing, but our geometry comes from the sim graph, not a grammar. [inference] Write our own small tube and instancing code, borrowing ideas from these crates.

### Cited Findings
- **bevy_symbios** (v0.10.0, MIT) is a Bevy integration for the Symbios L-system ecosystem. It turns L-system skeletons into "smooth tube meshes from skeleton strands", one mesh per material id, with per-vertex RGBA colors and arc-length UVs. It has an LRU MeshCache, 57 procedural texture generators (Leaf, Bark…), OBJ/GLB export, and optional Avian3D colliders, an egui palette editor, and a `.lsys` asset loader. — [docs.rs README](https://docs.rs/crate/bevy_symbios/latest/source/README.md)
- **bevy_procedural_tree** (v0.4.0, MIT/Apache-2.0) offers "Procedurally generated 3D trees for bevy", depends on bevy ^0.19, and has an optional bevy-inspector-egui. Its algorithm is not documented on docs.rs. — [docs.rs](https://docs.rs/bevy_procedural_tree); [source](https://docs.rs/crate/bevy_procedural_tree/latest/source)
- Other Rust repos surfaced but were not examined: [sequoia (GitHub)](https://github.com/pi-game/sequoia) and [pixeltree](https://docs.rs/crate/pixeltree/0.2.1/source/README.md) (2D).
- Hädrich 2017 benchmarks on a GTX 970: about 10k particles took ~17–18 ms per step, and about 25k took ~52 ms. Growth and rendering dominated the frame time. — [PDF](https://www.cs.purdue.edu/homes/bbenes/papers/Haedrich2017EG.pdf). [inference] This is similar hardware to a GTX 1070, but that run was full GPU physics. Our CPU sim plus render-only GPU is far lighter.

### Inferences
- [inference] On a GTX 1070, at M2 (30 plants × ~300 leaves plus stems and roots), the risk is not the triangle count. It is draw calls and per-frame mesh uploads. Use one instanced draw per leaf template and one merged stem mesh per plant, and update per-instance buffers rather than meshes.
- [inference] Pin a Bevy version and avoid relying on third-party crates in the render core, because Bevy breaks APIs each release. bevy_procedural_tree already tracks 0.19, so check our Bevy version before borrowing it.

### Gaps
- I could not verify bevy_symbios's Bevy version or the Rust L-system crates (`lsystem`, etc.) this session.

## M2 note: climbing (pole bean) from Hädrich 2017
- Plants are chains of anisotropic particles with meshless shape matching. A contact creates an anchor that pulls the particle toward the surface, and the pull strengthens with growth. "Surface adaptation" rotates the tip to run parallel to the surface. Obstacles use SDFs. Gravity bends branches, and phototropism uses shadow-map light exposure. The paper does not model tendrils or twining; it covers adhesion climbers like ivy and grapevine. For twining mechanics it points to Goriely & Neukirch and to Casati & Bertails-Descoubes. — [PDF](https://www.cs.purdue.edu/homes/bbenes/papers/Haedrich2017EG.pdf)
- [inference] For our pole bean, a kinematic rule is cheaper and deterministic: once a tip has found a support (our tendril cone search), it follows a helix around the support axis, with radius = support radius + stem radius and pitch from the species file. Our plan does not need Hädrich's physics. Use its "anchor + align to surface" idea only for the render-side flop of an unsupported vine.
- Open-source generators to study (Blender Sapling = Weber-Penn, The Grove, Arbaro, tree-it, Thomas Luft's ivy generator, SpeedTree, Plant Factory) were not examined this session. Only [prior knowledge] links Sapling and Arbaro to Weber-Penn and the ivy generator to adhesion-driven ivy growth on meshes.
