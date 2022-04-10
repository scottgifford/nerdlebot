#!/usr/bin/perl

use strict;
use warnings;
use feature 'unicode_strings';
binmode(STDIN, ':utf8');
binmode(STDOUT, ':utf8');

$/ = "\n===";
# Skip first record (compilation info, etc.)
<>;

my $totalgames = 0;
my $slowgames = 0;
my $failures = 0;
while (<>) {
    s/\s+$//;
    if (/^Game (\d+) completed in (\d+\.\d+)(.*?)\s*$/m) {
        my($game, $time, $units) = ($1, $2, $3);
        $totalgames++;
        # print "GAME $game TIME $time$units.\n";
        if ($units eq 's' && $time > 10) {
            print;
            ++$slowgames;
        }
    } else {
        $failures++;
        print "Line $. did not match!\n";
        print;
    }
}

print "Analyzed $totalgames, $slowgames were slow and $failures failed to parse\n";
exit $failures;
