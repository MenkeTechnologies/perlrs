BEGIN { print "b:", ${^GLOBAL_PHASE}, "\n" }
print "m:", ${^GLOBAL_PHASE}, "\n";
END { print "e:", ${^GLOBAL_PHASE}, "\n" }
