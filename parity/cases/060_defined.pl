# Perl `print` renders false as empty; use numeric coercion for a single-line check.
print 0 + defined("x"), 0 + defined(undef);
print "\n";
