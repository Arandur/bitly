table! {
    canonical_shortlinks (name) {
        name -> Varchar,
        target -> Varchar,
    }
}

table! {
    custom_shortlinks (name) {
        name -> Varchar,
        target -> Varchar,
    }
}

table! {
    stats (name) {
        name -> Varchar,
        created_on -> Timestamptz,
        visits -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    canonical_shortlinks,
    custom_shortlinks,
    stats,
);
