# vec - get bits from a string
my $s = "A";  # 0x41 = 01000001
print "vec0:", vec($s, 0, 8), "\n";  # 65
print "vec1:", vec($s, 0, 1), "\n";  # bit 0 of 'A' (0x41) = 1
