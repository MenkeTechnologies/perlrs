# bulk:833
my @b = unpack("C*", pack("C", 33)); printf "%d\n", $b[0];
