# pack/unpack multiple new templates combined
my $data = pack("slCd", -1, 100000, 255, 1.5);
my ($s_val, $l_val, $c_val, $d_val) = unpack("slCd", $data);
print "s:$s_val\n";
print "l:$l_val\n";
print "C:$c_val\n";
printf "d:%.1f\n", $d_val;
