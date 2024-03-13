use crate::db::DATABASE_POOL;
use crate::proto;
use crate::proto_structs::user::User;

pub async fn handle_create_user(
    request: tonic::Request<proto::CreateUserRequest>,
) -> Result<tonic::Response<proto::CreateUserResponse>, tonic::Status> {
    let user_create_request = request.get_ref();

    let user_params = User {
        id: None,
        first_name: user_create_request.first_name.to_string(),
        last_name: user_create_request.last_name.to_string(),
    };

    let query = "
            INSERT INTO users (first_name, last_name)
            VALUES ($1, $2)
            RETURNING id, first_name, last_name
        ";

    let row = sqlx::query_as::<_, User>(query)
        .bind(&user_params.first_name)
        .bind(&user_params.last_name)
        .fetch_one(&*DATABASE_POOL)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

    let user_response = proto::CreateUserResponse {
        user_id: row.id.unwrap(),
        first_name: row.first_name,
        last_name: row.last_name,
    };

    Ok(tonic::Response::new(user_response))
}