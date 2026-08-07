use std::collections::HashMap;

use diesel::prelude::*;

use crate::{
    models::major::{Major, NewMajor, UpdateMajor},
    schema::major::dsl::*,
};

pub struct MajorRepository;

impl MajorRepository {
    pub fn create(conn: &mut PgConnection, new_major: &NewMajor) -> QueryResult<Major> {
        diesel::insert_into(major)
            .values(new_major)
            .returning(Major::as_returning())
            .get_result(conn)
    }

    pub fn find_all(
        conn: &mut PgConnection,
        params: &HashMap<String, String>,
    ) -> QueryResult<Vec<Major>> {
        let mut query = major.select(Major::as_select()).into_boxed();

        if let Some(name_) = params
            .get("name")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            query = query.filter(name.ilike(format!("%{}%", name_)));
        }

        query.load(conn)
    }

    pub fn find_by_id(conn: &mut PgConnection, major_id: i64) -> QueryResult<Major> {
        major
            .filter(id.eq(major_id))
            .select(Major::as_select())
            .first(conn)
    }

    pub fn update(
        conn: &mut PgConnection,
        major_id: i64,
        update_major: &UpdateMajor,
    ) -> QueryResult<Major> {
        diesel::update(major.filter(id.eq(major_id)))
            .set(update_major)
            .returning(Major::as_returning())
            .get_result(conn)
    }

    pub fn delete(conn: &mut PgConnection, major_id: i64) -> QueryResult<usize> {
        diesel::delete(major.filter(id.eq(major_id))).execute(conn)
    }
}
