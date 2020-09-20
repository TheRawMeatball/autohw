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
    }
}

table! {
    homework (id) {
        id -> Int4,
        due_date -> Nullable<Date>,
        detail -> Text,
        amount -> Int2,
        weekday -> Nullable<Int4>,
        class_id -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
    }
}

table! {
    hw_progress (homework_id, user_id) {
        homework_id -> Int4,
        user_id -> Int4,
        progress -> Int2,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        pwhs -> Varchar,
        class_id -> Nullable<Int4>,
    }
}

joinable!(cancels -> homework (homework_id));
joinable!(homework -> classes (class_id));
joinable!(homework -> users (user_id));
joinable!(hw_progress -> homework (homework_id));
joinable!(hw_progress -> users (user_id));
joinable!(users -> classes (class_id));

allow_tables_to_appear_in_same_query!(cancels, classes, homework, hw_progress, users,);
