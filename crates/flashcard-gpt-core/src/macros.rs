#[macro_export]
macro_rules! single_object_query {
    ($db:expr, $query:expr, $( $binding:expr ),* ) => {{
        let mut query = $db.query($query);

        $(
            query = query.bind($binding);
        )*

        let mut response = query.await?;
        response.errors_or_ok()?;
        let result: Option<_> = response.take(response.num_statements() - 1)?;

        if let Some(result) = result {
            Ok(result)
        } else {
            Err(CoreError::DbQueryResultNotFound(Arc::from(format!("{:?}", $query))))
        }
    }};
}

#[macro_export]
macro_rules! multi_object_query {
    ($db:expr, $query:expr, $( $binding:expr ),* ) => {{
        let mut query = $db.query($query);

        $(
            query = query.bind($binding);
        )*

        let mut response = query.await?;
        response.errors_or_ok()?;
        let result: Vec<_> = response.take(response.num_statements() - 1)?;
        Ok(result)
    }};
}
