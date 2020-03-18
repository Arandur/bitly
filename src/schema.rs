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
    }
}

table! {
    visits (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        visit -> Timestamptz,
        ip_addr -> Nullable<Varchar>,
    }
}

joinable!(visits -> stats (name));

allow_tables_to_appear_in_same_query!(
    canonical_shortlinks,
    custom_shortlinks,
    stats,
    visits,
);
