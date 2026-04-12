# pack/unpack: i I (signed/unsigned native int)
my $packed = pack("i", -42);
my ($v) = unpack("i", $packed);
print "i:-42=$v\n";
my $packed2 = pack("I", 42);
my ($v2) = unpack("I", $packed2);
print "I:42=$v2\n";
