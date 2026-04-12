# pack/unpack: s S (signed/unsigned 16-bit native)
my $packed = pack("s", -1);
my ($v) = unpack("s", $packed);
print "s:-1=$v\n";
my $packed2 = pack("S", 65535);
my ($v2) = unpack("S", $packed2);
print "S:65535=$v2\n";
my $packed3 = pack("s3", 1, 2, 3);
my @vals = unpack("s3", $packed3);
print "s3:", join(",", @vals), "\n";
