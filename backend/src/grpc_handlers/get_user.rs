use crate::db::DATABASE_POOL;
use crate::proto;
use crate::proto_structs::user::User;

pub async fn handle_get_user(
    request: tonic::Request<proto::GetUserRequest>,
) -> Result<tonic::Response<proto::GetUserResponse>, tonic::Status> {
    let get_user_request = request.get_ref();

    let query = format!(
        "SELECT id, first_name, last_name FROM users WHERE id = {}",
        get_user_request.user_id
    );

    let row = sqlx::query_as::<_, User>(&query)
        .fetch_one(&*DATABASE_POOL)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

    let user_response = proto::GetUserResponse {
        user_id: row.id.unwrap(),
        first_name: row.first_name,
        last_name: row.last_name,
    };

    Ok(tonic::Response::new(user_response))
}