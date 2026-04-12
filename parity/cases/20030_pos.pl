my $str = "hello world";
if ($str =~ /hello/g) {
    my $p = pos($str);
    print "pos after hello: $p\n";
}

if ($str =~ /world/g) {
    my $p = pos($str);
    print "pos after world: $p\n";
}

# pos on no match
my $str2 = "abc";
if ($str2 =~ /xyz/g) {
    print "should not reach\n";
}
my $p = pos($str2);
my $def = defined $p;
if ($def) { print "pos no match: defined\n"; }
else { print "pos no match: undef\n"; }
