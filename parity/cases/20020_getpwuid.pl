#!/usr/bin/perl
use strict;
use warnings;

# getpwuid in list context
my @pw = getpwuid($<);
print "name: $pw[0]\n";
print "uid: $pw[2]\n";
my $has_dir = (defined $pw[7] && length($pw[7]) > 0) ? 1 : 0;
print "has_dir: $has_dir\n";

# scalar context
my $name = getpwuid($<);
print "scalar_name: $name\n";
print "match: ", ($name eq $pw[0] ? 1 : 0), "\n";

# getpwnam round-trip
my @pw2 = getpwnam($name);
print "round_trip_uid: ", ($pw2[2] == $pw[2] ? 1 : 0), "\n";
