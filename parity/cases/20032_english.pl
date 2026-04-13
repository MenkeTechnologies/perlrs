use English;
# Basic aliases
print "PID ok\n" if $PID == $$;
print "PROCESS_ID ok\n" if $PROCESS_ID == $$;
print "PROGRAM_NAME ok\n" if defined $PROGRAM_NAME;

# Separator aliases
$OUTPUT_FIELD_SEPARATOR = ",";
print "a", "b", "c";
print "\n";
$OUTPUT_FIELD_SEPARATOR = "";
$LIST_SEPARATOR = ":";
my @arr = (1,2,3);
print "@arr\n";
$LIST_SEPARATOR = " ";

# Regex match aliases (use concatenation — $& interpolation is a separate issue)
"hello world" =~ /(\w+)\s+(\w+)/;
print "MATCH=" . $MATCH . "\n";
print "LAST_PAREN_MATCH=" . $LAST_PAREN_MATCH . "\n";

# Error aliases
eval { die "test error\n" };
print "EVAL_ERROR=" . $EVAL_ERROR;

# System info aliases
print "OSNAME=", (length($OSNAME) > 0 ? "ok" : "fail"), "\n";
print "PERL_VERSION=", (defined($PERL_VERSION) ? "ok" : "fail"), "\n";
print "BASETIME=", ($BASETIME > 0 ? "ok" : "fail"), "\n";

# UID/GID aliases
print "UID=", (defined($UID) ? "ok" : "fail"), "\n";
print "EUID=", (defined($EUID) ? "ok" : "fail"), "\n";

# Writable aliases
$INPUT_RECORD_SEPARATOR = ":";
print "RS=" . $INPUT_RECORD_SEPARATOR . "\n";
$INPUT_RECORD_SEPARATOR = "\n";

# -no_match_vars
no English;
use English qw(-no_match_vars);
print "PID still ok\n" if $PID == $$;
print "ERRNO ok\n" if defined $ERRNO;
"foo bar" =~ /(foo)/;
# $MATCH should NOT be an alias now; it's a regular (undef) variable
print "MATCH after no_match=" . (defined($MATCH) ? $MATCH : "undef") . "\n";
