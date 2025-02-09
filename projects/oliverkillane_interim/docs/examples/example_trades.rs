use emdb::database;

database!(
    name Trading;

    table trades {
        tradeid: u64 @ unique,
        vol: i16 @ check(it != 0),
        instrument: ref instruments[feedcode],
    };

    table instruments {
        feedcode: text[30] @ unique,
        underlying: option[ref instruments[feedcode]],
    };

    query get_trades_for_same_underlying(fc) = {
        instruments
            |> unique(it[feedcode], fc)
            |> get_some(underlying);
            |> let under;

        instruments
            |> where(some(it[underlying]) = under)
            |> let instrs;
    } -> { // can get trades in different transaction
        trades
            |> where(it[instrument] in instrs)
            |> select(tradeid, instrument)
            |> collect(`Vec`)
            |> return;
    };

    query trades_per_underlying() = {
        instruments
            |> filter(is_some(it[underlying]))
            |> let instruments_with_underlying;

        instruments * trades
            |> where(some(it[instruments][underlying]) = it[trades][feedcode])
            |> group(instruments[underlying])
            |> map(size)
            |> collect(`Vec`)
            |> return;
    }

    query insert_instr(instr: single[instruments]) -> usize = {
        instruments
            <| insert(instr)
            |> size()
            |> return;
    };

    query insert_trade(vol: i16, fc: `&str`) -> u64 = {
        trades
            |> size()
            |> let new_id;

        trades
            <| insert(tradeid = new_id, vol=vol, feedcode = fc)
            |> size()
            |> return;
    };
);

pub fn demo() {
    let db = trading::DB::new();
    println!("The schema is: {}", Trading::Schema);

    match db.insert_instr(Trading::InstrumentSingle {feedcode: "hello", underlying: None}) {
        Error(_) => ; // A Trading::Error::InsertInstr (Unique, NoRef)
        Ok(size) => println!(size);
    }

    if let Ok(trades) = get_trades_for_all_underlying("hello") {
        for t in trades {
            // ...
        }
    }
}