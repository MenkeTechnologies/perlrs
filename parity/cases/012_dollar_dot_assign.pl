# $. assignment resets/advances the per-handle line counter (perlvar).
open my $fh, "<", $0 or die;
my $x = <$fh>;
print "after_first_read:" . $. . "\n";
$. = 10;
print "after_assign:" . $. . "\n";
$x = <$fh>;
print "after_second_read:" . $. . "\n";
close $fh;
