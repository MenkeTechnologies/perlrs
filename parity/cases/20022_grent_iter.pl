# setgrent / getgrent / endgrent
setgrent();
my @first = getgrent();
endgrent();
print defined($first[0]) ? "grent:ok" : "grent:fail", "\n";
print length($first[0]) > 0 ? "name:ok" : "name:empty", "\n";
