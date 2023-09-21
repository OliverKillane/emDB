use emdb::database;

database!(
    name Stats;

    table entities {
        id: text[30] @ unique,
        group: ref groups[name],
        cool: bool,
        score: i32,
    }

    table groups {
        name: smalltext[10] @ unique,
        members: set[ref entities[id]],
    }

    query new_group(group_name: `&str`) = {
        groups <| insert(name = group_name, members = 0)
    }

    query new_entity(name: `&str`, group: `&str`, is_cool: bool) = {
        entities 
            <| insert(
                id = name, 
                group = group, 
                cool = is_cool, 
                score = 0
            );
    }

    query bump_score(name: `&str`) = {
        entities |> unique(id, name) <| update(score += 1);
    }

    query get_cools_per_group() = {
        groups * entities
            |> where(it[entities][group] in it[groups][members])
            |> flatten()
            |> groupby(it[group])
            |> map(
                group     = it[key], 
                cools     = it[value] |> where( it[cool]) |> size(), 
                non_cools = it[value] |> where(!it[cool]) |> size(),
              )
            |> return;
    }
)

fn demo() {
    let db = Stats::DB::new();
    
    assert!(db.query_bump_score("bob").is_err());

    assert!(db.new_group("blue").is_ok());
    assert!(db.new_entity("jim", "blue", true).is_ok());
    assert!(db.bump_score("jim").is_ok());
}
