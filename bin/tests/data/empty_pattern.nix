[
  # match
  ({ ... }: 42)

  # don't match
  ({ a, ... }: a)
  ({ ... } @ inputs: inputs)
]

