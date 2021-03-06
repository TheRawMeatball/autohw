table! {
    cancels (cancel_id) {
        cancel_id -> Int4,
        homework_id -> Int4,
        on_date -> Date,
    }
}

table! {
    classes (id) {
        id -> Int4,
        name -> Text,
        blackboard -> Text,
    }
}

table! {
    homework (id) {
        id -> Int4,
        due_date -> Nullable<Date>,
        detail -> Text,
        amount -> Int2,
        day_of_week -> Nullable<Int4>,
        class_id -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
    }
}

table! {
    hw_progress (homework_id, user_id) {
        homework_id -> Int4,
        user_id -> Int4,
        progress -> Int2,
        delta -> Int2,
        delta_date -> Date,
        weight -> Int4,
        last_repeat_reset -> Nullable<Date>,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        pwhs -> Varchar,
        class_id -> Nullable<Int4>,
        day_weights -> Array<Int4>,
    }
}

joinable!(cancels -> homework (homework_id));
joinable!(homework -> classes (class_id));
joinable!(homework -> users (user_id));
joinable!(hw_progress -> homework (homework_id));
joinable!(hw_progress -> users (user_id));
joinable!(users -> classes (class_id));

allow_tables_to_appear_in_same_query!(
    cancels,
    classes,
    homework,
    hw_progress,
    users,
);
