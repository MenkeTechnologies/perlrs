# pack/unpack: f d (single/double float)
my $packed = pack("d", 3.14);
my ($v) = unpack("d", $packed);
printf "d:%.2f\n", $v;
my $packed2 = pack("f", 2.5);
my ($v2) = unpack("f", $packed2);
printf "f:%.1f\n", $v2;
