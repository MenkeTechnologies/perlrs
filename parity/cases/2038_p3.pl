# parity:2038
my $o = bless { x => 3, y => 14 }, "P99"; printf "%d\n", $o->{x} + $o->{y};
