# gethostbyname
my @h = gethostbyname("localhost");
print defined($h[0]) ? "host:ok" : "host:undef", "\n";
