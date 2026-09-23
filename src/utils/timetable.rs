use std::collections::HashMap;

use diesel::PgConnection;

use crate::{
    error::app_error::AppError,
    models::{
        course::Course, course_assignment::NewCourseAssignment,
        course_curriculum::CourseCurriculum, curriculum::Curriculum, major::Major,
        master_course::MasterCourse, professor::Professor, semester::Semester, user::User,
    },
    repository::{
        course_assignment_repository::CourseAssignmentRepository,
        course_curriculum_repository::CourseCurriculumRepository,
        course_pool_repository::CoursePoolRepository,
        course_preference_repository::CoursePreferenceRepository,
    },
};

pub fn init_hungarian_matrix(
    conn: &mut PgConnection,
    courses: &Vec<(
        Course,
        CourseCurriculum,
        MasterCourse,
        Curriculum,
        Semester,
        Major,
    )>,
    professors: &Vec<(Professor, User, Semester)>,
    semester_id: i64,
) -> Result<Vec<Vec<i32>>, AppError> {
    let mut hungarian_matrix = vec![vec![0; courses.len()]; professors.len()];
    for (row_idx, (professor, _, _)) in professors.iter().enumerate() {
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

pub fn minimize_hungarian_matrix(
    hungarian_matrix: &mut Vec<Vec<i32>>,
    courses_len: usize,
    professors_len: usize,
) {
    for row_idx in 0..professors_len {
        let min_value = *hungarian_matrix[row_idx].iter().min().unwrap();
        for row_idx in 0..professors_len {
            for col_idx in 0..courses_len {
                hungarian_matrix[row_idx][col_idx] -= min_value;
            }
        }
    }
    for col_idx in 0..courses_len {
        let min_value = hungarian_matrix
            .iter()
            .map(|row| row[col_idx])
            .min()
            .unwrap();
        for row_idx in 0..professors_len {
            hungarian_matrix[row_idx][col_idx] -= min_value;
        }
    }
    for row_idx in 0..professors_len {
        for col_idx in 0..courses_len {
            hungarian_matrix[row_idx][col_idx] = match hungarian_matrix[row_idx][col_idx] {
                -1 => 0,
                _ => hungarian_matrix[row_idx][col_idx],
            }
        }
    }
}

pub fn execute_round(
    conn: &mut PgConnection,
    courses: &mut Vec<(
        Course,
        CourseCurriculum,
        MasterCourse,
        Curriculum,
        Semester,
        Major,
    )>,
    professors: &mut Vec<(Professor, User, Semester)>,
    hungarian_matrix: &mut Vec<Vec<i32>>,
) -> Result<bool, AppError> {
    let mut is_changed = false;

    let courses_len = courses.len();
    let professors_len = professors.len();

    minimize_hungarian_matrix(hungarian_matrix, courses_len, professors_len);

    let mut col_idx = 0;
    while col_idx < courses.len() {
        let mut check_zero = vec![];
        hungarian_matrix.iter().for_each(|row| {
            for row_idx in 0..professors_len {
                if row[col_idx] == 0 {
                    check_zero.push(row_idx);
                }
            }
        });
        if check_zero.len() == 1 {
            let row_idx = check_zero[0];

            CourseAssignmentRepository::create(
                conn,
                &NewCourseAssignment {
                    course_id: courses[col_idx].0.id,
                    professor_id: professors[row_idx].0.id,
                },
            )
            .map_err(|_| AppError::DatabaseError)?;

            // ToDo: Implement professor_quota check and remove professor from the list if the quota is full

            courses.remove(col_idx);
            hungarian_matrix.remove(row_idx);
            for row in hungarian_matrix.iter_mut() {
                row.remove(col_idx);
            }

            is_changed = true;
        } else {
            col_idx += 1;
        }
    }

    Ok(is_changed)
}
