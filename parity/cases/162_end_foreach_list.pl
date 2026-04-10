# Iterator `foreach` must evaluate its LIST in list context (ranges, arrays), including in END.
$main::end_out = "";
END { foreach $k (1..3) { $main::end_out .= "k=$k " } }
