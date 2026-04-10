BEGIN { print "b1:", ${^GLOBAL_PHASE}, "\n" }
UNITCHECK { print "uc:", ${^GLOBAL_PHASE}, "\n" }
CHECK { print "ch:", ${^GLOBAL_PHASE}, "\n" }
INIT { print "in:", ${^GLOBAL_PHASE}, "\n" }
print "m:", ${^GLOBAL_PHASE}, "\n";
END { print "e:", ${^GLOBAL_PHASE}, "\n" }
