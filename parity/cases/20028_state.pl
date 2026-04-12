use feature "state";

sub counter {
    state $n = 0;
    $n++;
    return $n;
}

print counter(), "\n";
print counter(), "\n";
print counter(), "\n";

sub greet {
    state $count = 10;
    $count++;
    return $count;
}

print greet(), "\n";
print greet(), "\n";

# state with no initializer
sub tracker {
    state $x;
    if (!defined $x) {
        $x = 100;
    }
    $x++;
    return $x;
}

print tracker(), "\n";
print tracker(), "\n";
