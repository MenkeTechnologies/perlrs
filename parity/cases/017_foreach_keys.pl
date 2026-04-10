my %h = (x => 1, y => 2);
my $s = 0;
foreach my $k (keys %h) { $s = $s + $h{$k}; }
print $s . "\n";
