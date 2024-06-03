use crate::perms::Permission;
use color_eyre::{eyre::bail, Result};
use log::{debug, info};
use sqlx::{Row, Sqlite, SqlitePool};

pub static DATABASE_URL: &str = "file:cms-data/data.db?mode=rwc";
static SCHEMA_VERSION: u32 = 1;

#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct User {
    id: String,
    username: String,
    /// The oauth2 auth token
    token: String,
    /// ISO-8601/RFC-3339 string
    expiration_date: String,
}

#[derive(Debug, PartialEq, sqlx::FromRow)]
struct Group {
    /// User ID
    uid: String,
    /// Group name
    group_name: String,
}

/// Permissions for a particular group
#[derive(Debug, sqlx::FromRow)]
pub struct GroupPermissions {
    #[sqlx(rename = "group_name")]
    pub name: String,
    /// Permissions are serialized as a comma delimited list of `Permission`s
    // While there is overhead that comes from deserializing this list every time,
    // it was preferred over the added complexity of needing multiple structs and more
    // advanced deserialization from sqlite
    #[sqlx(rename = "permissions")]
    serialized_perms: String,
}

impl GroupPermissions {
    /// Check and see if the provided permission is contained within the list of permissions for the group
    pub fn has(&self, perm: Permission) -> bool {
        self.serialized_perms
            .split(',')
            .map(|p| {
                p.try_into()
                    .expect("Permission stored in the database is not a valid permission")
            })
            .any(|p: Permission| p == perm)
    }

    // pub async fn
}

/// Initialize a new database at the provided URL. Not hardcoded in so that a memory db can be used
/// for testing
pub async fn init(url: &str) -> Result<sqlx::Pool<Sqlite>> {
    let pool = SqlitePool::connect(url).await?;
    // to handle scheme stuff, the version of the schema currently in use is stored a user_version
    // variable. If it's set to 0, assume the table hasn't been populated yet. Then each schema change should increment by 1, allowing migrations to take place
    // update user version
    let mut user_version = sqlx::query("PRAGMA user_version;")
        .fetch_one(&pool)
        .await?
        .get::<u32, _>(0);
    while user_version != SCHEMA_VERSION {
        match user_version {
            // the table hasn't been created, create it with the latest schema
            0 => {
                // Apparently SQLite doesn't have great support for u64s, so it's stored as a string here
                // https://github.com/launchbadge/sqlx/issues/499
                sqlx::query(
                    r"
                    CREATE TABLE user 
                    (id TEXT PRIMARY KEY, username TEXT, token TEXT, expiration_date TEXT)
                    STRICT;
                    ",
                )
                .execute(&pool)
                .await?;
                sqlx::query(
                    r"
                    CREATE TABLE group_membership
                    (uid TEXT, group_name TEXT, FOREIGN KEY(uid) REFERENCES user(id) ON DELETE CASCADE)
                    STRICT;
                    ",
                )
                .execute(&pool)
                .await?;
                debug!("Initialized the group_membership table");
                sqlx::query(
                    r"
                    CREATE TABLE group_permission
                    (group_name TEXT, permissions TEXT)
                    STRICT;
                    ",
                )
                .execute(&pool)
                .await?;
                debug!("Initialized the group_permission table");
                user_version = SCHEMA_VERSION;
                info!("Initialized fresh database");
            }
            _ => {
                panic!(
                    "The database does not have handling for the stored schema version: {user_version}"
                );
            }
        }
        sqlx::query(&format!("PRAGMA user_version = {user_version};"))
            .execute(&pool)
            .await?;
    }
    Ok(pool)
}

/// Attempt to read a user from the database, returning the found user, or None
pub async fn get_user(pool: &SqlitePool, uid: String) -> Result<Option<User>> {
    let query_results = sqlx::query_as::<_, User>(
        r"
        SELECT *
        FROM  user
        WHERE id = ?;
        ",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?;
    Ok(query_results)
}

pub async fn create_user(pool: &SqlitePool, user: &User) -> Result<()> {
    let query_results = sqlx::query(
        r"
        INSERT INTO user
        VALUES (?, ?, ?, ?);
        ",
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.token)
    .bind(&user.expiration_date)
    .execute(pool)
    .await?;
    if query_results.rows_affected() != 1 {
        bail!(
            "Create user impacted unexpected number of rows, impacted {} rows",
            query_results.rows_affected()
        )
    }
    Ok(())
}

/// Add the provided group to the user's list of groups
pub async fn add_group(pool: &SqlitePool, uid: String, group_name: String) -> Result<()> {
    let query_results = sqlx::query(
        r"
        INSERT INTO group_membership
        VALUES (?, ?);
        ",
    )
    .bind(uid)
    .bind(group_name)
    .execute(pool)
    .await?;
    if query_results.rows_affected() != 1 {
        bail!(
            "Group add impacted unexpected number of rows, impacted {} rows",
            query_results.rows_affected()
        );
    }
    Ok(())
}

/// Remove the provided user from the provided group
pub async fn remove_group(pool: &SqlitePool, uid: String, group_name: String) -> Result<()> {
    let query_results = sqlx::query(
        r"
        DELETE FROM group_membership
        WHERE (uid = ? AND group_name = ?);
        ",
    )
    .bind(uid)
    .bind(group_name)
    .execute(pool)
    .await?;
    if query_results.rows_affected() != 1 {
        bail!(
            "Group delete impacted unexpected number of rows, impacted {} rows",
            query_results.rows_affected()
        );
    }
    Ok(())
}

/// Returns a list of groups the user is in
pub async fn get_groups(pool: &SqlitePool, uid: String) -> Result<Vec<String>> {
    Ok(sqlx::query(
        r"
        SELECT group_name
        FROM group_membership
        WHERE uid = ?;
        ",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| r.get::<String, _>(0))
    .collect())
}

/// Create a new group with the provided permissions
pub async fn create_group(
    pool: &SqlitePool,
    group_name: String,
    perms: &[Permission],
) -> Result<()> {
    let query_results = sqlx::query(
        r"
        INSERT INTO group_permission
        VALUES (?, ?);
        ",
    )
    .bind(group_name)
    .bind(
        perms
            .iter()
            .map(|p| String::from(*p))
            .collect::<Vec<String>>()
            .join(","),
    )
    .execute(pool)
    .await?;
    if query_results.rows_affected() != 1 {
        bail!(
            "Group add impacted unexpected number of rows, impacted {} rows",
            query_results.rows_affected()
        );
    }
    Ok(())
}

/// Returns the permissions for the provided group
pub async fn get_perms(pool: &SqlitePool, group_name: String) -> Result<GroupPermissions> {
    let query_results = sqlx::query_as::<_, GroupPermissions>(
        r"
        SELECT * FROM group_permission
        WHERE group_name = ?;
        ",
    )
    .bind(group_name)
    .fetch_one(pool)
    .await?;
    Ok(query_results)
}

#[cfg(test)]
mod tests {
    use super::User;
    use super::{add_group, create_user, get_groups, get_user, init};
    use chrono::{DateTime, Utc};
    use std::time::UNIX_EPOCH;
    #[tokio::test]
    async fn user_management() {
        let mock_db = init(":memory:").await.unwrap();
        let mock_id = "hi mom".to_string();
        assert_eq!(
            None,
            get_user(&mock_db, mock_id).await.unwrap(),
            "get_user should return None when no user is found with that id"
        );
        let mock_user = User {
            id: "1234".to_string(),
            username: "foo".to_string(),
            token: "bar".to_string(),
            expiration_date: DateTime::<Utc>::from(UNIX_EPOCH).to_rfc3339(),
        };

        create_user(&mock_db, &mock_user).await.unwrap();
        if let Some(user) = get_user(&mock_db, "1234".to_string()).await.unwrap() {
            assert_eq!(
                mock_user, user,
                "The user set should equal the user returned from the database"
            );
        }
        // TODO: delete user, update user
    }

    #[tokio::test]
    async fn group_management() {
        let mock_db = init(":memory:").await.unwrap();
        let mock_user = User {
            id: "1234".to_string(),
            username: "foo".to_string(),
            token: "bar".to_string(),
            expiration_date: DateTime::<Utc>::from(UNIX_EPOCH).to_rfc3339(),
        };

        create_user(&mock_db, &mock_user).await.unwrap();
        assert_eq!(
            get_groups(&mock_db, "1234".to_string()).await.unwrap(),
            Vec::<String>::new(),
            "User should have no groups when none were added"
        );
        add_group(&mock_db, "1234".to_string(), "foo".to_string())
            .await
            .unwrap();
        add_group(&mock_db, "1234".to_string(), "bar".to_string())
            .await
            .unwrap();
        assert_eq!(
            get_groups(&mock_db, "1234".to_string())
                .await
                .unwrap()
                .len(),
            2
        );
        // TODO: delete group, update group
    }

    // TODO: perm management
}
