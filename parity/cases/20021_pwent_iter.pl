# setpwent / getpwent / endpwent
setpwent();
my @first = getpwent();
endpwent();
print defined($first[0]) ? "pwent:ok" : "pwent:fail", "\n";
print length($first[0]) > 0 ? "name:ok" : "name:empty", "\n";
