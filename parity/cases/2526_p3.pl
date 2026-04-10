# parity:2526
my @a = (16, 12, 20); @a = sort { $a <=> $b } @a; printf "%d\n", $a[2];
