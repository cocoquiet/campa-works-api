use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    dto::master_course::{
        CreateMasterCourseRequest, MasterCourseResponse, UpdateMasterCourseRequest,
    },
    error::app_error::AppError,
    service::master_course_service::MasterCourseService,
    state::app_state::AppState,
};

pub async fn create_master_course(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateMasterCourseRequest>,
) -> Result<(StatusCode, Json<MasterCourseResponse>), AppError> {
    let conn = state
        .pool
        .get()
        .await
        .map_err(|_| AppError::DatabaseError)?;

    let course = conn
        .interact(move |conn| MasterCourseService::create(conn, request))
        .await
        .map_err(|_| AppError::DatabaseError)??;

    Ok((StatusCode::CREATED, Json(course)))
}

pub async fn get_master_courses(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<MasterCourseResponse>>, AppError> {
    let conn = state
        .pool
        .get()
        .await
        .map_err(|_| AppError::DatabaseError)?;

    let courses = conn
        .interact(move |conn| MasterCourseService::get_all(conn, &params))
        .await
        .map_err(|_| AppError::DatabaseError)??;

    Ok(Json(courses))
}

pub async fn get_master_course(
    State(state): State<Arc<AppState>>,
    Path(master_course_id): Path<i64>,
) -> Result<Json<MasterCourseResponse>, AppError> {
    let conn = state
        .pool
        .get()
        .await
        .map_err(|_| AppError::DatabaseError)?;

    let course = conn
        .interact(move |conn| MasterCourseService::get_by_id(conn, master_course_id))
        .await
        .map_err(|_| AppError::DatabaseError)??;

    Ok(Json(course))
}

pub async fn update_master_course(
    State(state): State<Arc<AppState>>,
    Path(master_course_id): Path<i64>,
    Json(request): Json<UpdateMasterCourseRequest>,
) -> Result<Json<MasterCourseResponse>, AppError> {
    let conn = state
        .pool
        .get()
        .await
        .map_err(|_| AppError::DatabaseError)?;

    let course = conn
        .interact(move |conn| MasterCourseService::update(conn, master_course_id, request))
        .await
        .map_err(|_| AppError::DatabaseError)??;

    Ok(Json(course))
}

pub async fn delete_master_course(
    State(state): State<Arc<AppState>>,
    Path(master_course_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let conn = state
        .pool
        .get()
        .await
        .map_err(|_| AppError::DatabaseError)?;

    conn.interact(move |conn| MasterCourseService::delete(conn, master_course_id))
        .await
        .map_err(|_| AppError::DatabaseError)??;

    Ok(StatusCode::NO_CONTENT)
}
