"""Tiny RON writer for the validation schema (scratch tool, not committed)."""
def s(x): return '"' + str(x).replace('\\', '\\\\').replace('"', '\\"') + '"'
def opt(x, f=s): return 'None' if x is None else f'Some({f(x)})'
def num(x):
    x = float(x); r = repr(x)
    return r
def curve(id, source, conditions, xvar, xunit, yvar, yunit, basis, method, points, gate='Info', comment=None):
    lines = []
    if comment:
        lines += ['// ' + c for c in comment.split('\n')]
    pts = ',\n'.join(f'    Point(x: {num(x)}, y: {num(y)}, sd: {opt(sd, num)})' for x, y, sd in points)
    lines.append(f'''Curve(
  schema: 1,
  id: {s(id)},
  source: {s(source)},
  conditions: {s(conditions)},
  x: Axis(variable: {s(xvar)}, unit: {s(xunit)}),
  y: Axis(variable: {s(yvar)}, unit: {s(yunit)}),
  basis: {opt(basis)},
  method: {s(method)},
  gate: {gate},
  points: [
{pts},
  ],
)''')
    return '\n'.join(lines) + '\n'
