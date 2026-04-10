my @a = qw(x y z);
print join("", grep { $_ eq "x" || $_ eq "y" } @a);
print "\n";
