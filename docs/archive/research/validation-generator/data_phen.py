# Phenology and biomass sources (from research/phenology_biomass.md and dssat_timeseries.md)
SOURCES = {
 "pinzon2024": dict(citation="Pinzón-Sandoval E.H., Carvalho Silva W., Marchiori P.E.R. (2024) Phenology, mass accumulation patterns and growing degree days in common bean. Revista de Ciencias Agrícolas 41(3): e3239.",
    doi="10.22267/rcia.20244103.239", url="https://revistas.udenar.edu.co/index.php/rfacia/article/view/9029", licence="CC BY-NC 4.0",
    cultivar="BRSMG Marte (red seed; habit not stated in the paper)",
    conditions="Greenhouse, Lavras MG, Brazil, 919 m; one plant per pot; period 1 April-June 2023, period 2 July-September 2023; GDD base 8.3 C, mean method, no upper threshold"),
 "cavalcante2020": dict(citation="Cavalcante A.G., Lemos L.B., Meirelles F.C., Cavalcante A.C.P., Aquino L.A. de (2020) Thermal sum and phenological descriptions of growth stages of the common bean according to the BBCH scale. Annals of Applied Biology 176(3): 342-349. Values read from the identical chapter in Cavalcante A.G. (2020) doctoral thesis, UNESP/FCAV, http://hdl.handle.net/11449/202139.",
    doi="10.1111/aab.12571", url="https://repositorio.unesp.br/bitstreams/3dd90c34-e24b-4e0c-9b93-8e8c28948610/download", licence="article: Wiley standard; thesis: not checked",
    cultivar="Type I: BRS Artico, BRS Embaixador, IPR Garca (pooled); Type II: BRS Executivo, BRS FS 305",
    conditions="Field, Jaboticabal SP, Brazil, 565 m, Koppen Aw, 2018/2019; thermal sum base 10 C (Arnold 1960)"),
 "dapaah1999": dict(citation="Dapaah H.K., McKenzie B.A., Hill G.D. (1999) Effects of irrigation and sowing date on phenology and yield of pinto beans (Phaseolus vulgaris L.) in Canterbury, New Zealand. New Zealand Journal of Crop and Horticultural Science 27(4): 297-305.",
    doi="10.1080/01140671.1999.9514109", url="https://scispace.com/papers/effects-of-irrigation-and-sowing-date-on-phenology-and-yield-4coasw55bo", licence="not stated",
    cultivar="Othello (pinto)", conditions="Field, Lincoln, Canterbury NZ (temperate), 60 plants/m2, six sowing dates Oct-Dec 1994-95, irrigated and dryland pooled; thermal time base 7 C, mean method"),
 "ventura1991": dict(citation="Ventura Elías R. (1991) Fenología y fenometría de una variedad y una línea de frijol (Phaseolus vulgaris) en la zona occidental de El Salvador. Agronomía Mesoamericana 2: 56-60. (The design plan cites it as 2016, the date of the online record.)",
    doi="10.15517/am.v2i0.25224", url="https://archivo.revistas.ucr.ac.cr/index.php/agromeso/article/view/25224", licence="CC BY-NC-ND 4.0",
    cultivar="Rojo de Seda and line APN 83 (pooled)", conditions="Field, Ahuachapan, western El Salvador, 615-640 m, warm tropical second season, 1987-1989"),
 "baldinotti2026": dict(citation="Baldinotti H. da S., Conceição M.P. da, Kondo Y.R., Ferreira E.P. de B., Hungria M., Kaschuk G. (2026) Interaction among growth cycle, photosynthesis, and nodule enzymatic activity in Phaseolus vulgaris L. inoculated with Rhizobium tropici CIAT 899. Annals of Applied Biology. (The design plan cites it as Conceição 2026.)",
    doi="10.1111/aab.70142", url="https://scispace.com/papers/interaction-among-growth-cycle-photosynthesis-and-nodule-llcwlzokpkc7", licence="not found (abstract only read)",
    cultivar="BRS FC104 (super-early), BRS Pontal (intermediate), BRS Vereda (long)", conditions="Greenhouse, Brazil; values from the abstract"),
 "croptime2020": dict(citation="Andrews N., Coop L., Stoven H., Noordijk H. (2020) Beans, Phaseolus vulgaris L. Vegetable Phenology (Degree-Day) Model Documentation, version 1.0, 11 June 2020. Oregon State University CROPTIME project (USDA Western SARE).",
    doi=None, url="https://instar.biossys.oregonstate.edu/CROPTIME/bean_varieties_models.pdf", licence="not stated",
    cultivar="Snap bush beans Provider, 5630, Sahara", conditions="Field, Willamette Valley, western Oregon (temperate), 2013-2015, 4-9 site-years per variety; single-sine degree-days, thresholds 40 F and 90 F, from direct seeding"),
 "maldonado1993": dict(citation="Maldonado G., Corchuelo G. (1993) Dinámica de crecimiento de dos variedades de frijol (Phaseolus vulgaris L.). I. Análisis de crecimiento por periodos fijos. Agronomía Colombiana 10(2): 114-121.",
    doi=None, url="https://revistas.unal.edu.co/index.php/agrocol/article/view/21269", licence="open access journal (licence not checked)",
    cultivar="ICA-Cerinza (Type I) and ICA-Tundama (Type II)", conditions="Field, Mosquera, Colombia, 2640 m, mean temperature 14 C, 0.50 x 0.12 m spacing, late 1992"),
 "cropgro_drybean_spe": dict(citation="DSSAT CSM CROPGRO-Drybean species, ecotype and cultivar files BNGRO048.SPE/.ECO/.CUL (DSSAT CSM v4.8), DSSAT Foundation.",
    doi=None, url="https://github.com/DSSAT/dssat-csm-os/tree/develop/Data/Genotype", licence="BSD-3-Clause (dssat-csm-os)",
    cultivar="Generic and calibrated cultivars; Type I ecotypes MESDET and ANDDET", conditions="Model calibration tables, not measurements"),
}
DSSAT_DATA = "DSSAT Foundation, dssat-csm-data repository (Drybean and GreenBean folders), commit 99c95b34 (2026-09-30)"
for k, exp, cond, cv in [
  ("dssat_ccpa9001", "CCPA9001", "Field, CIAT Palmira, Colombia, 1990, 16-cultivar seed-size trial, 25 plants/m2, irrigated (experimenters P. Saxton, J.W. White, K.J. Boote)", "Redkloud (treatment 15) and 15 others"),
  ("dssat_ccpo9002", "CCPO9002", "Field, CIAT Popayan, Colombia (cooler, higher), 1990, 16-cultivar trial, 25 plants/m2", "Redkloud (treatment 15) and 15 others"),
  ("dssat_ufga8501", "UFGA8501", "Field, Gainesville FL, 1985, sandy soil, 51.7 plants/m2, irrigated vs rainfed", "Seafarer, Isabella, Manitou (Type I), C-20"),
  ("dssat_ufga0701", "UFGA0701", "Field, Gainesville FL, spring 2007, snap bean, 34 plants/m2, N rate x harvest stage (data noted unpublished; ask Djidonou and Boote)", "Bronco (Habit 1)"),
]:
    SOURCES[k] = dict(citation=f"DSSAT CROPGRO bean experiment {exp} (.BNX/.BNT/.BNA or .GBX/.GBT/.GBA). {DSSAT_DATA}.",
        doi=None, url="https://github.com/DSSAT/dssat-csm-data", licence="no licence file in the repository; values cited with attribution to the DSSAT Foundation and the experimenters",
        cultivar=cv, conditions=cond)
