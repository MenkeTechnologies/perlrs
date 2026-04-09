my @data = (1..5000);
my @result = pmap { $_ * $_ + $_ * 3 + 7 } @data;
print scalar @result, "\n";
