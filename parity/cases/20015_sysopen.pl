#!/usr/bin/perl
use strict;
use warnings;

# sysopen test using numeric flag values
# On macOS: O_WRONLY=1, O_CREAT=0x200, O_TRUNC=0x400, O_APPEND=0x0008
my $f = "/tmp/perlrs_test_sysopen_$$.tmp";
my $flags = 1 | 0x200 | 0x400;  # O_WRONLY | O_CREAT | O_TRUNC
sysopen(SFH, $f, $flags, 0644) or die "sysopen: $!";
print SFH "sysopen works\n";
close(SFH);

open(RFH, '<', $f) or die;
my $line = <RFH>;
close(RFH);
chomp $line;
print "content: $line\n";

unlink $f;
