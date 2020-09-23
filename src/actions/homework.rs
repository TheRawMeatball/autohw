use std::time;

use crate::models::homework::*;
use crate::pub_imports::*;
use chrono::{Datelike, Duration, NaiveDate, Utc};
use diesel::{prelude::*, PgConnection};
use models::{hw_progress::HWProgressModel, user::User};
use num_traits::FromPrimitive;

pub fn change_progress(
    user: &User,
    amount: i16,
    id: i32,
    use_delta: bool,
    conn: &PgConnection,
) -> Result<(), HomeworkApiError> {
    use schema::hw_progress::dsl::*;

    let mut model: HWProgressModel = hw_progress
        .filter(homework_id.eq(id).and(user_id.eq(user.id)))
        .first(conn)
        .map_err(|e| HomeworkApiError::DieselError(e))?;

    if use_delta {
        model.delta += amount;
        if model.delta_date != now() {
            model.progress += model.delta;
            model.delta = 0;
            model.delta_date = now();
        }
    } else {
        model.progress += amount;
    }

    model
        .save_changes::<HWProgressModel>(conn)
        .map(|_| ())
        .map_err(|e| HomeworkApiError::DieselError(e))
}

pub fn add_homework(
    model: &AddHomeworkModel,
    user: AuthUser,
    conn: &PgConnection,
) -> Result<(), HomeworkApiError> {
    let due_date = model.due_date.map(|m| m.0);
    let day_of_week = if due_date.is_some() {
        None
    } else if let Some(dow) = model.weekday {
        Some(dow)
    } else {
        return Err(HomeworkApiError::UnspecifiedTime);
    };

    let (user_id, class_id) = if model.for_self {
        (Some(user.id), None)
    } else {
        (None, user.class_id)
    };

    let insert = NewHomeworkModel {
        due_date,
        day_of_week,
        user_id,
        class_id,
        amount: model.amount,
        detail: &model.detail,
    };

    let hw_id = {
        use schema::homework::dsl::*;
        diesel::insert_into(homework)
            .values(insert)
            .get_result::<DbHomeworkModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
            .id
    };

    let insert = if model.for_self {
        vec![HWProgressModel {
            user_id: user.id,
            delta: 0,
            progress: 0,
            delta_date: now(),
            homework_id: hw_id,
        }]
    } else {
        use schema::users::dsl::*;

        users
            .filter(class_id.eq(user.class_id))
            .select(id)
            .load(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
            .iter()
            .map(|user_id| HWProgressModel {
                user_id: *user_id,
                delta: 0,
                progress: 0,
                delta_date: now(),
                homework_id: hw_id,
            })
            .collect()
    };

    {
        use schema::hw_progress::dsl::*;

        diesel::insert_into(hw_progress)
            .values(insert)
            .execute(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?;
    }

    Ok(())
}

pub fn get_homework_for_user(
    user: &User,
    conn: &PgConnection,
) -> Result<Vec<UserHomework>, HomeworkApiError> {
    let source = schema::homework::table.inner_join(schema::hw_progress::table);
    let now = now();

    let result = {
        use schema::homework::dsl::*;

        source
            .filter(
                user_id
                    .eq(user.id)
                    .or(class_id.eq(user.class_id))
                    .and(due_date.gt(now)),
            )
            .select((
                id,
                due_date,
                detail,
                amount,
                day_of_week,
                schema::hw_progress::progress,
                schema::hw_progress::delta,
                schema::hw_progress::delta_date,
            ))
            .load::<ProgressHomeworkModel>(conn)
            .map_err(|e| HomeworkApiError::DieselError(e))?
    };

    for hw in result.iter() {
        if hw.delta_date != Utc::now().date().naive_local() {
            use schema::hw_progress::dsl::*;
            let mut x: HWProgressModel = hw_progress
                .filter(homework_id.eq(hw.id))
                .first(conn)
                .unwrap();
            x.progress += x.delta;
            x.delta = 0;
            x.delta_date = now;
            x.save_changes::<HWProgressModel>(conn).unwrap();
        }
    }

    Ok(result
        .into_iter()
        .map(|model| UserHomework::from(model))
        .collect())
}

#[derive(Clone, Debug)]
struct HwModel {
    hw: UserHomework,
    due: i32,
}

impl HwModel {
    fn left(&self) -> i16 {
        self.hw.amount - self.hw.progress
    }
}

pub fn create_schedule(all: &Vec<UserHomework>, weights: &[i16; 7]) -> Vec<(i32, Vec<DailyHomework>)> {
    let now = now();

    let last_date = all
        .iter()
        .filter_map(|m| match &m.due_date {
            DueDate::Date(d) => Some(d.clone()),
            _ => None,
        })
        .max()
        .unwrap_or(now.succ());

    let mut last_day = (last_date - now).num_days().abs();

    let mut all: Vec<_> = all
        .iter()
        .map(|m| match m.due_date {
            DueDate::Repeat(day_of_week) => {
                let dow = chrono::Weekday::from_i32(day_of_week).unwrap();
                let mut distance = i64::MAX;
                let mut date = NaiveDate::from_weekday_of_month(now.year(), now.month(), dow, 1);

                let mut v = vec![];
                let one_week =
                    Duration::from_std(time::Duration::from_secs(60 * 60 * 24 * 7)).unwrap();

                let mut progress = m.progress;

                loop {
                    if date < now {
                        date = date + one_week;
                        continue;
                    }

                    let day = (last_date - date).num_days().abs();
                    if distance > day {
                        distance = day;

                        if last_day < (date - now).num_days() {
                            last_day = (date - now).num_days();
                        }

                        if progress >= m.amount {
                            progress -= m.amount;
                        } else {
                            let mut m = m.clone();
                            m.detail = format!("{} ({})", m.detail, date);
                            m.progress = progress;
                            progress = 0;
                            v.push(HwModel {
                                hw: m,
                                due: (date - now).num_days() as i32 - 1,
                            });
                        }

                        date = date + one_week;
                    } else {
                        break;
                    }
                }
                v
            }
            DueDate::Date(d) => vec![HwModel {
                hw: m.clone(),
                due: (d - now).num_days() as i32 - 1,
            }],
        })
        .flatten()
        .collect();

    let one_day = Duration::from_std(time::Duration::from_secs(60 * 60 * 24)).unwrap();

    let mut workload = vec![0i16; last_day as usize];
    let mut work_split: Vec<_> = (0..last_day as i32)
        .into_iter()
        .map(|day| (weights[(now + one_day * day).weekday().num_days_from_monday() as usize], 0))
        .collect();

    for hw in all.iter() {
        workload[hw.due as usize] += hw.hw.amount - hw.hw.progress;
    }

    all.sort_by_key(|x| x.due);

    create_work_split(workload,  &mut work_split[..]);

    work_split
        .iter()
        .enumerate()
        .fold((all, vec![]), |(mut all, mut v), (day, (_, load))| {
            let mut load = *load;
            let mut for_today = vec![];
            while load > 0 {
                if all[0].left() > load {
                    for_today.push(DailyHomework {
                        hw: all[0].hw.clone(),
                        amount: load as i32,
                    });
                    all[0].hw.progress += load;
                    load = 0; // break;
                } else {
                    let hw = all.remove(0);
                    let left = hw.left();
                    for_today.push(DailyHomework {
                        hw: hw.hw,
                        amount: left as i32,
                    });
                    load -= left;
                }
            }
            v.push((day as i32, for_today));
            (all, v)
        }).1
}

fn create_work_split(workload: Vec<i16>, work_split: &mut[(i16, i16)]) {
    let last_day = workload.len();

    for day in 0..last_day {
        for start in 0..=day {
            let slice = &mut work_split[start as usize..=day as usize];
            let counts = || slice.iter().map(|(_, c)| c);
            let effective_len = slice.iter().map(|(w, _)| w).sum::<i16>();
            let sum = counts().sum::<i16>() + workload[day as usize];
            let avg = sum as f32 / effective_len as f32;
            if avg > *counts().max().unwrap() as f32 {
                let added = avg.floor() as i16;

                for load in slice.iter_mut() {
                    load.1 = added * load.0;
                }

                let mut excess = sum % effective_len;

                for i in 0.. {
                    let load = excess.min(slice[i].0);
                    slice[i].1 += load;
                    excess -= load;
                    if excess == 0 {
                        break;
                    }
                }

                break;
            }
        }
    }
}

#[derive(Debug)]
pub enum HomeworkApiError {
    UnspecifiedTime,
    DieselError(diesel::result::Error),
}
impl std::fmt::Display for HomeworkApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self)).unwrap();
        Ok(())
    }
}
impl std::error::Error for HomeworkApiError {}
impl From<diesel::result::Error> for HomeworkApiError {
    fn from(e: diesel::result::Error) -> Self {
        HomeworkApiError::DieselError(e)
    }
}

#[inline]
fn now() -> NaiveDate {
    Utc::now().date().naive_local()
}
