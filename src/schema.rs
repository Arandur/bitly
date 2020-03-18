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

allow_tables_to_appear_in_same_query!(
    canonical_shortlinks,
    custom_shortlinks,
);
