# bulk:645
my @b = unpack("C*", pack("C", 45)); printf "%d\n", $b[0];
