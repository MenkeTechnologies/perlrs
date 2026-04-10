# `$` without `/m` matches end of string or before a final newline (perlvar / perlre).
$_ = "foo\n";
print "M" if /foo$/;
