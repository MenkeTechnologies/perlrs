sub target {
    print "target got: @_\n";
}

sub bouncer {
    goto &target;
}

bouncer("hello", "world");

# goto &sub preserves @_
sub add_them {
    my $sum = $_[0] + $_[1];
    print "sum=$sum\n";
}

sub trampoline {
    goto &add_them;
}

trampoline(3, 7);

# chained goto
sub final_dest {
    print "final: $_[0]\n";
}

sub middle {
    goto &final_dest;
}

sub start {
    goto &middle;
}

start("chain");
