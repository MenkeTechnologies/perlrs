# parity:2114
my @a = (23, 37, 22); @a = sort { $a <=> $b } @a; printf "%d\n", $a[2];
