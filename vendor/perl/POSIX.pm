# Minimal stub: satisfy `use POSIX qw(...)` from pure-Perl deps (real POSIX is XS).
package POSIX;

our $VERSION = '0.01';

sub import {
    my $class = shift;
    # Ignore requested symbols; callers that need real POSIX::* subs will fail at runtime.
    return;
}

1;
