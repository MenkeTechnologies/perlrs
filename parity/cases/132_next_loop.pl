my $s = 0;
for my $i (1..3) {
    next if $i == 2;
    $s += $i;
}
print $s;
print "\n";
