use std::collections::HashMap;

use diesel::prelude::*;
use diesel::result::QueryResult;

use crate::{
    models::{
        enums::SemesterType,
        semester::{NewSemester, Semester, UpdateSemester},
    },
    schema::semester::dsl::*,
};

pub struct SemesterRepository;

impl SemesterRepository {
    pub fn create(conn: &mut PgConnection, new_semester: &NewSemester) -> QueryResult<Semester> {
        diesel::insert_into(semester)
            .values(new_semester)
            .returning(Semester::as_returning())
            .get_result(conn)
    }

    pub fn find_all(
        conn: &mut PgConnection,
        params: &HashMap<String, String>,
    ) -> QueryResult<Vec<Semester>> {
        let mut query = semester
            .select(Semester::as_select())
            .order((year.desc(), semester_.asc()))
            .into_boxed();

        if let Some(semester_id) = params.get("id").and_then(|value| value.parse::<i64>().ok()) {
            query = query.filter(id.eq(semester_id));
        }
        if let Some(semester_year) = params
            .get("year")
            .and_then(|value| value.parse::<i32>().ok())
        {
            query = query.filter(year.eq(semester_year));
        }
        if let Some(semester_type) = params
            .get("semester_")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            query = query.filter(semester_.eq(SemesterType::from(semester_type)));
        }

        query.load(conn)
    }

    pub fn find_by_id(conn: &mut PgConnection, semester_id: i64) -> QueryResult<Semester> {
        semester
            .filter(id.eq(semester_id))
            .select(Semester::as_select())
            .first(conn)
    }

    pub fn update(
        conn: &mut PgConnection,
        semester_id: i64,
        update_semester: &UpdateSemester,
    ) -> QueryResult<Semester> {
        diesel::update(semester.filter(id.eq(semester_id)))
            .set(update_semester)
            .returning(Semester::as_returning())
            .get_result(conn)
    }

    pub fn delete(conn: &mut PgConnection, semester_id: i64) -> QueryResult<usize> {
        diesel::delete(semester.filter(id.eq(semester_id))).execute(conn)
    }
}
