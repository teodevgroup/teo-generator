use std::borrow::Cow;

fn action_result_type<'a>(&self, action: Action, model_name: &'a str) -> Cow<'a, str> {
    Cow::Owned(match action.to_u32() {
        FIND_UNIQUE_HANDLER => format!("Response<undefined, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        FIND_FIRST_HANDLER => format!("Response<undefined, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        FIND_MANY_HANDLER => format!("Response<PagingInfo, CheckSelectInclude<T, {model_name}[], {model_name}GetPayload<T>[]>>"),
        CREATE_HANDLER => format!("Response<undefined, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        UPDATE_HANDLER => format!("Response<undefined, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        UPSERT_HANDLER => format!("Response<undefined, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        DELETE_HANDLER => format!("Response<undefined, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        CREATE_MANY_HANDLER => format!("Response<PagingInfo, CheckSelectInclude<T, {model_name}[], {model_name}GetPayload<T>[]>>"),
        UPDATE_MANY_HANDLER => format!("Response<PagingInfo, CheckSelectInclude<T, {model_name}[], {model_name}GetPayload<T>[]>>"),
        DELETE_MANY_HANDLER => format!("Response<PagingInfo, CheckSelectInclude<T, {model_name}[], {model_name}GetPayload<T>[]>>"),
        COUNT_HANDLER => format!("Response<undefined, CheckSelectInclude<T, number, {model_name}GetPayload<T>>>"),
        AGGREGATE_HANDLER => format!("Response<undefined, CheckSelectInclude<T, never, {model_name}GetPayload<T>>>"),
        GROUP_BY_HANDLER => format!("Response<undefined, CheckSelectInclude<T, never, {model_name}GetPayload<T>>>"),
        SIGN_IN_HANDLER => format!("Response<TokenInfo, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        IDENTITY_HANDLER => format!("Response<undefined, CheckSelectInclude<T, {model_name}, {model_name}GetPayload<T>>>"),
        _ => unreachable!()
    })
}