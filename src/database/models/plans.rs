use diesel::{
    query_builder::AsChangeset, BoolExpressionMethods, ExpressionMethods, Insertable, QueryDsl,
    Queryable, RunQueryDsl,
};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

use crate::database::{connection::DbConn, schema::plans};

/// Plan struct
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, AsChangeset)]
#[diesel(table_name = plans)]
pub struct Plan {
    /// Plan name
    name: String,
    user_id: i32,
    /// Last time the plan was modified
    last_modified: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "plans"]
pub struct NewPlan {
    pub name: String,
    user_id: i32,
}

impl Plan {
    /// Create a new plan for a user
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to the database
    /// * `name` - Name of the plan, must be unique
    /// * `user_id` - User ID
    ///
    /// # Returns
    ///
    /// The newly created plan
    pub fn new(conn: &mut DbConn, name: &str, user_id: i32) -> Result<Self, AppError> {
        let new_plan = NewPlan {
            name: name.to_string(),
            user_id,
        };

        diesel::insert_into(plans::table)
            .values(&new_plan)
            .get_result::<Plan>(conn)
            .map_err(|e| {
                tracing::error!("Failed creating new plan \"{name}\" for user {user_id} ({e})");
                AppError::Diesel(e)
            })
    }

    /// Get a plan by user ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to the database
    /// * `user_id` - User ID
    ///
    /// # Returns
    ///
    /// A vector of plans associated with a user
    pub fn get_all(conn: &mut DbConn, user_id: i32) -> Result<Vec<Self>, AppError> {
        plans::table
            .filter(plans::user_id.eq(user_id))
            .load::<Plan>(conn)
            .map_err(|e| {
                tracing::error!("Failed getting plans for user {user_id} ({e})");
                AppError::Diesel(e)
            })
    }

    /// Delete a plan by name and user ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to the database
    /// * `name` - Name of the plan
    /// * `user_id` - User ID
    ///
    /// # Returns
    ///
    /// The number of rows deleted
    pub fn delete(conn: &mut DbConn, name: &str, user_id: i32) -> Result<bool, AppError> {
        let rows = diesel::delete(
            plans::table.filter(plans::name.eq(name).and(plans::user_id.eq(user_id))),
        )
        .execute(conn)
        .map_err(|e| {
            tracing::error!("Failed deleting plan \"{name}\" for user {user_id} ({e})");
            AppError::Diesel(e)
        })?;

        Ok(rows > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{connection::DbPool, models::users::User};
    use diesel::prelude::*;

    #[test]
    fn test_new_plan() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let user = User::default(conn).unwrap();
        let user_id = user.id();
        let name = "Test Plan";

        // Create a new plan
        let plan = Plan::new(conn, name, user_id).unwrap();
        assert_eq!(plan.name, name);
        assert_eq!(plan.user_id, user_id);

        // Get all plans
        let plans = Plan::get_all(conn, user_id).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].name, name);

        // Create another plan
        let name2 = "Test Plan 2";
        let plan2 = Plan::new(conn, name2, user_id).unwrap();
        assert_eq!(plan2.name, name2);
        assert_eq!(plan2.user_id, user_id);

        // Get all plans
        let plans = Plan::get_all(conn, user_id).unwrap();
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].name, name);
        assert_eq!(plans[1].name, name2);

        // Delete the plan
        let deleted = Plan::delete(conn, name, user_id).unwrap();
        assert!(deleted);
    }

    #[test]
    fn test_duplicate_plan() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let user = User::default(conn).unwrap();
        let user_id = user.id();
        let name = "Test Plan";

        // Create a new plan
        let plan = Plan::new(conn, name, user_id).unwrap();
        assert_eq!(plan.name, name);
        assert_eq!(plan.user_id, user_id);

        // Try to create the same plan again
        let result = Plan::new(conn, name, user_id);
        assert!(result.is_err());
    }
}
