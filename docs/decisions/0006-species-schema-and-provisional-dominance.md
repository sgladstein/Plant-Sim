# 0006: Species schema 1 and a provisional dominance form

- **Date:** 2026-11-05
- **Status:** accepted (implementer; §6.1 shows only a sketch with "// ...")

## Context
Design plan §6.1 sketches the species file and elides most parameters. Later work
packages (light, physiology, architecture, roots, reproduction) need named parameters
with units. §3.12 says each locus has "an additive effect and a dominance deviation"
but does not give the formula for continuous allele values.

## Decision
- Schema 1 fixes the parameter set in `crates/ps-species/src/schema.rs`: 39 constants,
  16 heritable parameters, 6 phenology fields and 5 numeric fields per root type, each
  with an exact unit string. Unknown keys are errors; all keys are required.
- `constants` and `phenology` use RON struct syntax (as in §6.1); `heritable` is a map.
- Phenotype mapping: `value = clamp(default + Σ [effect · (a + b)/2 + dominance · |a − b|], min, max)`
  over the loci for that trait, with allele values `a`, `b` drawn from N(0, allele_sd).
  Homozygotes have no dominance deviation; heterozygotes deviate in proportion to how
  different their alleles are.
- Phenology thresholds are in the `phenology` section, not heritable, in M0. They become
  heritable when breeding arrives (M3), with a schema bump.

## Consequences
- Later WPs that need a new parameter add it to the schema and the bean file together,
  and bump the schema if files already exist outside the repo.
- The dominance form is revisited in M3 when crosses exist; with zero variance in M0 it
  has no effect.
