use strict; use warnings;
# TSV (commande, niveau, console, aide) -> annexe Markdown groupée par famille.
binmode STDOUT, ':raw';
my %fam;
open my $f, '<:raw', $ARGV[0] or die;
while (<$f>) {
    chomp; s/\r$//;
    my ($cmd, $lvl, $con, $help) = split /\t/, $_, 4;
    my ($first) = split / /, $cmd;
    push @{ $fam{$first} }, [$cmd, $lvl, $con];
}
my @single = grep { @{ $fam{$_} } == 1 && $fam{$_}[0][0] eq $_ } sort keys %fam;
my @multi  = grep { !(@{ $fam{$_} } == 1 && $fam{$_}[0][0] eq $_) } sort keys %fam;
print "### Commandes simples\n\n| Commande | Niveau | Console |\n|---|---|---|\n";
for my $k (@single) { my $e = $fam{$k}[0]; printf "| `.%s` | %s | %s |\n", $e->[0], $e->[1], $e->[2] ? 'oui' : '' }
print "\n";
for my $k (@multi) {
    printf "### .%s (%d)\n\n| Commande | Niveau | Console |\n|---|---|---|\n", $k, scalar @{ $fam{$k} };
    for my $e (@{ $fam{$k} }) { printf "| `.%s` | %s | %s |\n", $e->[0], $e->[1], $e->[2] ? 'oui' : '' }
    print "\n";
}
