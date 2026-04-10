my %h = (a => 1, b => 2, c => 3);
my $s = 0;
foreach my $v (values %h) { $s = $s + $v; }
print $s . "\n";
