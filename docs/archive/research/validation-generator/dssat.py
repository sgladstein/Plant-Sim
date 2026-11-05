import re, csv, io, os
MD = os.path.join(os.path.dirname(__file__), '..', 'research', 'dssat_timeseries.md')
def blocks():
    t = open(MD).read()
    out = {}
    for m in re.finditer(r'### (.*?)\n.*?```csv\n(.*?)```', t, re.S):
        out[m.group(1)] = list(csv.DictReader(io.StringIO(m.group(2))))
    for m in re.finditer(r'## (\d\. .*?|Bonus: .*?)\n.*?```csv\n(.*?)```', t, re.S):
        out[m.group(1)] = list(csv.DictReader(io.StringIO(m.group(2))))
    return out
if __name__ == '__main__':
    for k, v in blocks().items(): print(k, len(v))
