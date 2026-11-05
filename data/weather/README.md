# Weather files

Hourly and daily weather files a scenario can name as `File("weather/<name>.csv")` or
`DailyFile(...)`. The format is in design plan §6.2 and `ps_weather::csv`. Timestamps are
local mean solar time (decision 0005).

| File | What it is | Source |
| --- | --- | --- |
| sample_hourly_45n.csv | Synthetic hourly weather at 45 N, 10 May to 16 Sep 2026: a seasonal temperature curve with a diurnal sine, a daily clearness index and rain events from a fixed-seed generator | `scripts/make_sample_weather.py`; not observed weather. It exists so the reader and a full run are tested end to end (WP25, M2 exit). Replace it with a station file when one with a clear licence is chosen. |
