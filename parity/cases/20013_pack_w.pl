#!/usr/bin/perl
use strict;
use warnings;

# pack w (BER compressed integer)
my $p1 = pack("w", 0);
my $p2 = pack("w", 127);
my $p3 = pack("w", 128);
my $p4 = pack("w", 16383);
my $p5 = pack("w", 16384);

# Unpack and check round-trip
print "w0: ", unpack("w", $p1), "\n";
print "w127: ", unpack("w", $p2), "\n";
print "w128: ", unpack("w", $p3), "\n";
print "w16383: ", unpack("w", $p4), "\n";
print "w16384: ", unpack("w", $p5), "\n";

# Check length of encoding
print "len0: ", length($p1), "\n";
print "len127: ", length($p2), "\n";
print "len128: ", length($p3), "\n";
