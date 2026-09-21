use diesel::prelude::*;
use std::collections::HashMap;

use crate::{
    dto::course_assignment::{CourseAssignmentResponse, CreateCourseAssignmentRequest},
    error::app_error::AppError,
    models::course_assignment::NewCourseAssignment,
    repository::{
        course_assignment_repository::CourseAssignmentRepository,
        course_curriculum_repository::CourseCurriculumRepository,
        course_pool_repository::CoursePoolRepository,
        course_preference_repository::CoursePreferenceRepository,
        course_repository::CourseRepository, professor_repository::ProfessorRepository,
    },
};

pub struct CourseAssignmentService;

impl CourseAssignmentService {
    pub fn create(
        conn: &mut PgConnection,
        request: CreateCourseAssignmentRequest,
    ) -> Result<CourseAssignmentResponse, AppError> {
        CourseRepository::find_by_id(conn, request.course_id)
            .map_err(|_| AppError::CourseNotFound)?;

        ProfessorRepository::find_by_id(conn, request.professor_id)
            .map_err(|_| AppError::ProfessorNotFound)?;

        let query_params = HashMap::from([
            ("course_id".to_string(), request.course_id.to_string()),
            ("professor_id".to_string(), request.professor_id.to_string()),
        ]);

        if !CourseAssignmentRepository::find_all(conn, &query_params)
            .unwrap_or_else(|_| Vec::new())
            .is_empty()
        {
            return Err(AppError::CourseAssignmentAlreadyExists);
        }

        let new_course_assignment = NewCourseAssignment {
            course_id: request.course_id,
            professor_id: request.professor_id,
        };

        CourseAssignmentRepository::create(conn, &new_course_assignment)
            .map_err(|_| AppError::DatabaseError)?;

        let course_assignment = CourseAssignmentRepository::find_all(conn, &query_params)
            .map_err(|_| AppError::DatabaseError)?
            .into_iter()
            .next()
            .unwrap_or_else(|| unreachable!());

        Ok(course_assignment.into())
    }

    pub fn create_auto_in_new_semester(
        conn: &mut PgConnection,
        semester_id: i64,
    ) -> Result<Vec<CourseAssignmentResponse>, AppError> {
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

        // Init hungarian matrix(rows: professors, columns: courses, values: course_preference_score)
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

        // ToDo: Implement Round-Based Hungarian algorithm

        Ok(Vec::new())
    }

    pub fn get_all(
        conn: &mut PgConnection,
        params: &HashMap<String, String>,
    ) -> Result<Vec<CourseAssignmentResponse>, AppError> {
        let course_assignments = CourseAssignmentRepository::find_all(conn, params)
            .map_err(|_| AppError::DatabaseError)?;

        Ok(course_assignments.into_iter().map(Into::into).collect())
    }

    pub fn get_by_id(
        conn: &mut PgConnection,
        id: i64,
    ) -> Result<CourseAssignmentResponse, AppError> {
        let course_assignment = CourseAssignmentRepository::find_by_id(conn, id)
            .map_err(|_| AppError::CourseAssignmentNotFound)?;

        Ok(course_assignment.into())
    }

    pub fn delete(conn: &mut PgConnection, id: i64) -> Result<(), AppError> {
        CourseAssignmentRepository::find_by_id(conn, id)
            .map_err(|_| AppError::CourseAssignmentNotFound)?;

        CourseAssignmentRepository::delete(conn, id).map_err(|_| AppError::DatabaseError)?;

        Ok(())
    }
}
