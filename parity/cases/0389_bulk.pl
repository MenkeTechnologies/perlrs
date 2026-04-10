# bulk:389
my @a = grep { $_ > 10 } (10,12,15); printf "%d\n", scalar @a;
