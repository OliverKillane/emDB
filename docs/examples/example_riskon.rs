use emdb::database;

enum CheckStatus {
    Ready,
    Active,
    InActive,
    Disconnected
}

database!(
    name Riskon;

    table markets {
        market: smalltext[4] @ unique,
        checks: ref checks_states[id],
    }

    table checks {
        id: smalltext[10] @ unique,
        markets: set[ref check_states[id]],
    }

    table check_states {
        status: `::CheckStatus`,
        market: ref markets[market],
        check: ref checks[id],
    } @ unique [market, check]

    query insert_check() = {
        // TODO
    }
);

