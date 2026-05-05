use strict;
use warnings;
use Test::More;

# die
{
    eval { die "test error" };
    like($@, qr/test error/, "die: terminates execution with message");
}
{
    eval { die "error\n" };
    is($@, "error\n", "die: trailing newline suppresses file/line info");
}
{
    eval { die "error" };
    like($@, qr/at .+ line \d+/, "die: without trailing newline, appends file and line");
}
{
    my $reached = 0;
    eval {
        die "stop";
        $reached = 1;
    };
    is($reached, 0, "die: code after die is not executed");
}

# warn
{
    my $warned = 0;
    local $SIG{__WARN__} = sub { $warned = 1 };
    warn "test warning";
    is($warned, 1, "warn: triggers warning handler");
}
{
    my $reached = 0;
    {
        local $SIG{__WARN__} = sub {};
        warn "test";
        $reached = 1;
    }
    is($reached, 1, "warn: does NOT terminate execution");
}

# croak / confess
{
    use Carp;
    eval { croak "test croak" };
    like($@, qr/test croak/, "croak: dies with message like die");
}
{
    use Carp;
    eval { confess "test confess" };
    like($@, qr/test confess/, "confess: dies with message and stack trace");
}
{
    use Carp;
    my $reached = 0;
    eval {
        croak "stop";
        $reached = 1;
    };
    is($reached, 0, "croak: code after croak is not executed");
}

# eval recovery
{
    my $result = eval { die "error"; 42 } // 0;
    is($result, 0, "die: eval returns undef when die is thrown");
}
{
    my $result = eval { 42 };
    is($result, 42, "die: eval returns last expression when no die");
}

done_testing;
