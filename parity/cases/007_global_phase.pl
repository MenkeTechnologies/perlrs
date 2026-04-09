BEGIN { print "b1:", ${^GLOBAL_PHASE}, "\n" }
BEGIN { print "b2:", ${^GLOBAL_PHASE}, "\n" }
print "m:", ${^GLOBAL_PHASE}, "\n";
END { print "e:", ${^GLOBAL_PHASE}, "\n" }
