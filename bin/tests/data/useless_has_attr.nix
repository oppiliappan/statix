[
  # trivial
  (if x ? a then x.a else default)
  (if x.a ? b then x.a.b else default)
  (if x ? a.b then x.a.b else default)

  # complex body
  (if x ? a then x.a else if b then c else d)
  (if x ? a then x.a else b.c)
]
