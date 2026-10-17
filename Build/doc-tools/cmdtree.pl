use strict; use warnings;
# Reconstitue l'arborescence des commandes VMaNGOS depuis Chat.cpp :
#   static ChatCommand <table>[] = { { "nom", SEC_x, console, &handler, "aide", <sous-table|nullptr> }, ... };
# Sortie TSV : commande complète, niveau, console (1/0), aide.
my %lvl = (SEC_PLAYER => 0, SEC_MODERATOR => 1, SEC_TICKETMASTER => 2, SEC_GAMEMASTER => 3,
           SEC_BASIC_ADMIN => 4, SEC_DEVELOPER => 5, SEC_ADMINISTRATOR => 6, SEC_CONSOLE => 7);
local $/; open my $f, '<:raw', $ARGV[0] or die; my $src = <$f>;
my %tables;
while ($src =~ /static\s+ChatCommand\s+(\w+)\s*\[\]\s*=\s*\{(.*?)\n\s*\};/sg) {
    my ($tname, $body) = ($1, $2);
    my @entries;
    while ($body =~ /\{\s*"([^"]*)"\s*,\s*(SEC_\w+)\s*,\s*(true|false)\s*,\s*([^,]+?)\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*([\w]+)\s*\}/sg) {
        push @entries, { name => $1, sec => $lvl{$2} // $2, console => ($3 eq 'true' ? 1 : 0), handler => $4, help => $5, child => $6 };
    }
    $tables{$tname} = \@entries;
}
my @out;
sub walk {
    my ($tname, $prefix, $depth) = @_;
    return if $depth > 6;
    for my $e (@{ $tables{$tname} || [] }) {
        my $full = $prefix eq '' ? $e->{name} : ($e->{name} eq '' ? $prefix : "$prefix $e->{name}");
        my $has_handler = $e->{handler} !~ /^(nullptr|NULL|0)$/;
        if ($has_handler) { push @out, join("\t", $full, $e->{sec}, $e->{console}, $e->{help}) }
        if ($e->{child} ne 'nullptr' && $e->{child} ne 'NULL' && exists $tables{$e->{child}}) { walk($e->{child}, $full, $depth + 1) }
    }
}
walk('commandTable', '', 0);
my %seen; binmode STDOUT;
print "$_\n" for grep { !$seen{$_}++ } sort @out;
printf STDERR "%d tables, %d commandes\n", scalar(keys %tables), scalar(keys %seen);
