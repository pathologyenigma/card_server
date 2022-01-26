use super::types::LoginInput;
use crate::{error_handling::{BadInputErrorHandler, ErrorHandlerWithErrorExtensions}, user_types::User};
use async_graphql::{Context, Error, Object, Result};
use sea_orm::{ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter};
use tracing::{error, info};
#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    async fn log_in(&self, ctx: &Context<'_>, input: LoginInput) -> Result<String> {
        info!("Query.UserQuery.logIn accepted one request");
        let db = ctx.data_unchecked::<DbConn>();
        let mut bad_input_error_handler = ctx.data_unchecked::<BadInputErrorHandler>().clone();
        let user = crate::users::Entity::find()
            .filter(
                Condition::any()
                    .add(crate::users::Column::Username.eq(input.account.clone()))
                    .add(crate::users::Column::Email.eq(input.account.clone())),
            )
            .one(db)
            .await
            .expect("failed to query database");
        if let Some(user) = user {
            if user.password == input.password {
                let token = crate::tokenizer::Token::from(user).encode("just for now, future token will be in a config file".to_string()).expect("failed to parse token");
                info!("Query.UserQuery.logIn send a response token: {}", token);
                return Ok(token);
            } else {
                error!("bad input: wrong password");
                bad_input_error_handler.append("password".to_string(), "wrong password".to_string());
            }
        } else {
            error!("bad input: user not found");
            bad_input_error_handler.append("account".to_string(), "user not found".to_string());
        }
        if !bad_input_error_handler.is_none() {
            return Err(bad_input_error_handler.to_err());
        } else {
            return Err(Error::new("unexpected error"));
        }
    }
    /// you don't have to provide this id when you want yours
    async fn get_user_info_by_id(&self, ctx: &Context<'_>, id: Option<i32>) -> Result<User> {
        let token = ctx.data_opt::<crate::TokenFromHeader>();
        match token {
            Some(token) => {
                match id {
                    Some(id) => {
                        let db = ctx.data_unchecked::<DbConn>();
                        let mut bad_input_error_handler = ctx.data_unchecked::<BadInputErrorHandler>().clone();
                        let user = crate::users::Entity::find()
                        .filter(Condition::all().add(crate::users::Column::Id.eq(id)))
                        .one(db).await.expect("failed to query database");
                        match user {
                            Some(user) => {
                                return Ok(User{username: user.username, email: user.email});
                            },
                            None => {
                                bad_input_error_handler.append("id".to_string(), format!("user of id {} is not exist", id));
                                return Err(bad_input_error_handler.to_err());
                            }
                        }
                    },
                    None => {
                        let token = crate::Token::decode(token.0.clone(), "just for now, future token will be in a config file".to_string()).expect("failed to decode token");
                        return Ok(User{ username: token.username, email: token.email });
                    }
                }
            },
            None => {
                return Err(crate::new_not_authenticated_error());
            }
        }
        
    }
}
