use crate::error::AppError;


pub trait SqlxConstraints<T> {
    fn map_unique_err(self, target_type: &str, data: &str) -> crate::error::Result<T>;
    fn map_non_existence_err(self, target_type: &str, data: &str) -> crate::error::Result<T>; 
}

impl<T> SqlxConstraints<T> for std::result::Result<T, sqlx::Error> {
    fn map_unique_err(self, target_type: &str, data: &str) -> crate::error::Result<T> {
        self.map_err(|e| 
                e.as_database_error()
                    .is_some_and(|dbe| dbe.is_unique_violation())
                    .then(|| AppError::AlreadyExists { 
                        target_type: target_type.to_owned(), 
                        data: data.to_owned()
                    })
                    .unwrap_or(AppError::Sqlx(e))
            )
    }

    fn map_non_existence_err(self, target_type: &str, data: &str) -> crate::error::Result<T> {
        self.map_err(|e| 
            if let sqlx::Error::RowNotFound = e {
                AppError::DoesNotExist { 
                    target_type: target_type.to_owned(), 
                    data: data.to_owned()
                }
            } else {
                AppError::Sqlx(e)
            }
        )
    }
}
