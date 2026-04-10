# Multi-key / multi-index `+=` applies only to the last slot (Perl 5).
use strict;
use warnings;
my $h = { a => 10, b => 20 };
my $r = $h;
@$r{qw/a b/} += 5;
print $r->{a}, ",", $r->{b}, "\n";
# Anonymous array ref (same slice `+=` semantics as `\@a` in Perl 5).
my $ar = [ 10, 20, 30 ];
@$ar[ 0, 2 ] += 7;
print join( ",", @$ar ), "\n";
