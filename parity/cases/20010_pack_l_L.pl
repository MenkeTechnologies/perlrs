#!/usr/bin/perl
use strict;
use warnings;

# pack/unpack L with moderate values (avoid u32::MAX edge case)
my $p1 = pack("L", 0);
my $p2 = pack("L", 256);
my $p3 = pack("L", 65535);
my $p4 = pack("L", 1000000);

print "L0: ", unpack("L", $p1), "\n";
print "L256: ", unpack("L", $p2), "\n";
print "L65535: ", unpack("L", $p3), "\n";
print "L1M: ", unpack("L", $p4), "\n";

# l (signed 32-bit)
my $p5 = pack("l", -1);
my $p6 = pack("l", -32768);
print "l-1: ", unpack("l", $p5), "\n";
print "l-32768: ", unpack("l", $p6), "\n";
