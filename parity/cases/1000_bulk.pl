# bulk:1000
my @a = grep { $_ > 13 } (13,15,18); printf "%d\n", scalar @a;
