#!/usr/bin/perl
use strict;
use warnings;

# formline + $^A accumulator
$^A = "";
formline("@<<<< @>>>>", "foo", "bar");
print "formline: [$^A]\n";
my $len = length($^A);
print "len_ok: ", ($len > 0 ? 1 : 0), "\n";

# Reset accumulator
$^A = "";
formline("@####.##", 3.14159);
print "numeric: [$^A]\n";
