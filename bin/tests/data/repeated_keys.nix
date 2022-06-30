[
  # fine
  {
    foo.bar = 1;
  }

  # do not raise on rec
  rec {
    foo.x = foo.y;
    foo.y = 2;
    foo.z = 3;
  }

  # exactly 3 occurrences
  {
    foo.bar = 1;
    foo.bar."hello" = 1;
    foo.again = 1;
  }

  # more than 3, omit the extra
  {
    foo.baz.bar1 = 1;
    foo.baz.bar2 = 2;
    foo.baz.bar3 = 3;
    foo.baz.bar4 = 4;
    foo.baz.bar5 = 5;
  }
]
