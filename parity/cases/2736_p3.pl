# parity:2736
my @a = (6, 60, 23); @a = sort { $a <=> $b } @a; printf "%d\n", $a[2];
