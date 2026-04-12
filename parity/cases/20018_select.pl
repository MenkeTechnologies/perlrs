#!/usr/bin/perl
use strict;
use warnings;

# select() with timeout (4-arg form used as sleep)
my $t0 = time();
select(undef, undef, undef, 0.1);
my $t1 = time();
# Should take at most 2 seconds
print "elapsed_ok: ", ($t1 - $t0 < 2 ? 1 : 0), "\n";
print "select_ok: 1\n";
