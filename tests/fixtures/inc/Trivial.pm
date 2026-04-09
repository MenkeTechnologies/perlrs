# Test module for @INC / require / use (perlrs)
package Trivial;
our @EXPORT = qw(trivial_answer);
our @EXPORT_OK = qw(trivial_answer);
sub trivial_answer { 42 }
