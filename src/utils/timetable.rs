use std::collections::HashMap;

use diesel::PgConnection;

use crate::{
    error::app_error::AppError,
    repository::{
        course_curriculum_repository::CourseCurriculumRepository,
        course_pool_repository::CoursePoolRepository,
        course_preference_repository::CoursePreferenceRepository,
        course_repository::CourseRepository, professor_repository::ProfessorRepository,
    },
};

pub fn init_hungarian_matrix(
    conn: &mut PgConnection,
    semester_id: i64,
) -> Result<Vec<Vec<i32>>, AppError> {
    let courses = CourseRepository::find_all(
        conn,
        &HashMap::from([("semester_id".to_string(), semester_id.to_string())]),
    )
    .map_err(|_| AppError::DatabaseError)?;
    let professors = ProfessorRepository::find_all(
        conn,
        &HashMap::from([("professor_status".to_string(), "ACTIVE".to_string())]),
    )
    .map_err(|_| AppError::DatabaseError)?;

    let mut hungarian_matrix = vec![vec![0; courses.len()]; professors.len()];
    for (row_idx, (professor, _, _)) in professors.iter().enumerate() {
        let professor_course_preferences = CoursePreferenceRepository::find_all(
            conn,
            &HashMap::from([("professor_id".to_string(), professor.id.to_string())]),
        )
        .map_err(|_| AppError::DatabaseError)?;

        for (col_idx, (course, _, _, _, _, _)) in courses.iter().enumerate() {
            let master_course_id =
                CourseCurriculumRepository::find_by_id(conn, course.course_curriculum_id)
                    .map_err(|_| AppError::DatabaseError)?
                    .1
                    .id;

            let course_preferencec_score_query_params = HashMap::from([
                ("professor_id".to_string(), professor.id.to_string()),
                ("semester_id".to_string(), semester_id.to_string()),
                ("master_course_id".to_string(), master_course_id.to_string()),
            ]);
            hungarian_matrix[row_idx][col_idx] = match CoursePreferenceRepository::find_all(
                conn,
                &course_preferencec_score_query_params,
            )
            .map_err(|_| AppError::DatabaseError)?
            .into_iter()
            .next()
            {
                Some((course_preference, _, _, _, _)) => course_preference.priority,
                None => {
                    if CoursePoolRepository::find_all(
                        conn,
                        &HashMap::from([
                            ("professor_id".to_string(), professor.id.to_string()),
                            ("master_course_id".to_string(), master_course_id.to_string()),
                        ]),
                    )
                    .unwrap()
                    .is_empty()
                    {
                        9
                    } else {
                        8
                    }
                }
            };
        }
    }

    Ok(hungarian_matrix)
}
