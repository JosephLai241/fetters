// @generated automatically by Diesel CLI.

diesel::table! {
    interview_stages (id) {
        id -> Integer,
        job_id -> Integer,
        stage_number -> Integer,
        name -> Nullable<Text>,
        status -> Text,
        scheduled_date -> Text,
        notes -> Nullable<Text>,
        created -> Text,
    }
}

diesel::table! {
    jobs (id) {
        id -> Integer,
        created -> Text,
        company_name -> Text,
        title_id -> Integer,
        status_id -> Integer,
        link -> Nullable<Text>,
        notes -> Nullable<Text>,
        sprint_id -> Integer,
    }
}

diesel::table! {
    sprints (id) {
        id -> Integer,
        name -> Text,
        start_date -> Text,
        end_date -> Nullable<Text>,
        num_jobs -> Integer,
    }
}

diesel::table! {
    statuses (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    titles (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::joinable!(interview_stages -> jobs (job_id));
diesel::joinable!(jobs -> sprints (sprint_id));
diesel::joinable!(jobs -> statuses (status_id));
diesel::joinable!(jobs -> titles (title_id));

diesel::allow_tables_to_appear_in_same_query!(
    interview_stages,
    jobs,
    sprints,
    statuses,
    titles,
);
