use crate::perms::Permission;
use color_eyre::{eyre::bail, Result};
use log::debug;
use sqlx::{Sqlite, SqlitePool};

pub static DATABASE_URL: &str = "file:cms-data/data.db?mode=rwc";

// the ids have to be i64 because that's what sql uses
#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    /// The oauth2 auth token
    pub token: String,
    /// ISO-8601/RFC-3339 string
    pub expiration_date: String,
}

#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Group {
    pub id: i64,
    /// Group name
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct GroupMembership {
    user_id: i64,
    /// Group name
    group_id: i64,
}

#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct GroupPermissions {
    group_id: i64,
    permission: String,
}

/// Initialize a new database at the provided URL. Not hardcoded in so that a memory db can be used
/// for testing
pub async fn init(url: &str) -> Result<sqlx::Pool<Sqlite>> {
    let pool = SqlitePool::connect(url).await?;

    debug!("Running SQL migrations...");
    // this should embed the migrations into the executable itself
    sqlx::migrate!("./migrations").run(&pool).await?;
    debug!("SQL migrations complete");

    Ok(pool)
}

pub async fn create_user(
    pool: &SqlitePool,
    username: String,
    token: String,
    expiration_date: String,
) -> Result<User> {
    let query_results: User = sqlx::query_as(
        r"
        INSERT INTO users (username, token, expiration_date)
        VALUES (?, ?, ?) RETURNING *;
        ",
    )
    .bind(username)
    .bind(token)
    .bind(expiration_date)
    .fetch_one(pool)
    .await?;
    /*if query_results.rows_affected() != 1 {
        bail!(
            "Create user impacted unexpected number of rows, impacted {} rows",
            query_results.rows_affected()
        )
    }*/

    Ok(query_results)
}

/// Attempt to read a user from the database, returning the found user, or None
pub async fn get_user(pool: &SqlitePool, user_id: i64) -> Result<Option<User>> {
    let query_results: Option<User> = sqlx::query_as(r"SELECT * FROM  users WHERE id = ?;")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(query_results)
}

pub async fn get_user_from_token(pool: &SqlitePool, token: String) -> Result<Option<User>> {
    let query_results: Option<User> = sqlx::query_as(r"SELECT * FROM  users WHERE token = ?;")
        .bind(token)
        .fetch_optional(pool)
        .await?;
    Ok(query_results)
}

pub async fn get_user_groups(pool: &SqlitePool, user_id: i64) -> Result<Vec<Group>> {
    let groups: Vec<Group> = sqlx::query_as(
        "SELECT groups.* FROM group_membership 
        RIGHT JOIN groups ON group_membership.group_id = groups.id
        WHERE group_membership.user_id = ? ORDER BY groups.id;",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(groups)
}

pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>> {
    let query_results: Vec<User> = sqlx::query_as(r"SELECT * FROM users;")
        .fetch_all(pool)
        .await?;

    Ok(query_results)
}

/// note: the id of the user will not be updated.
pub async fn update_user(pool: &SqlitePool, user: &User) -> Result<()> {
    let query_result = sqlx::query(
        r"
        UPDATE users SET username = ?, token = ?, expiration_date = ?
        WHERE id = ?;",
    )
    .bind(&user.username)
    .bind(&user.token)
    .bind(&user.expiration_date)
    .bind(user.id)
    .execute(pool)
    .await?;

    if query_result.rows_affected() != 1 {
        bail!(
            "Update user impacted unexpected number of rows, impacted {} rows",
            query_result.rows_affected()
        )
    }

    Ok(())
}

pub async fn delete_user(pool: &SqlitePool, user_id: i64) -> Result<()> {
    let query_result = sqlx::query(r"DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;

    if query_result.rows_affected() != 1 {
        bail!(
            "Delete user impacted unexpected number of rows, impacted {} rows",
            query_result.rows_affected()
        )
    }

    Ok(())
}

/// Add the provided group to the user's list of groups
pub async fn create_group(pool: &SqlitePool, group_name: String) -> Result<Group> {
    let query_results: Group = sqlx::query_as(
        r"
        INSERT INTO groups (name) VALUES (?) RETURNING *;
        ",
    )
    .bind(group_name)
    .fetch_one(pool)
    .await?;

    Ok(query_results)
}

/// returns a Group object with information about the group
pub async fn get_group(pool: &SqlitePool, group_id: i64) -> Result<Option<Group>> {
    let query_results: Option<Group> = sqlx::query_as("SELECT * FROM groups WHERE id = ? LIMIT 1;")
        .bind(group_id)
        .fetch_optional(pool)
        .await?;

    Ok(query_results)
}

pub async fn get_all_groups(pool: &SqlitePool) -> Result<Vec<Group>> {
    let query_results: Vec<Group> = sqlx::query_as(r"SELECT * FROM groups;")
        .fetch_all(pool)
        .await?;

    Ok(query_results)
}

/// returns the members of a given group
pub async fn get_group_members(pool: &SqlitePool, group_id: i64) -> Result<Vec<User>> {
    let users: Vec<User> = sqlx::query_as(
        "SELECT users.* FROM group_membership 
        RIGHT JOIN users ON group_membership.user_id = users.id
        WHERE group_membership.group_id = ? ORDER BY users.id;",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    Ok(users)
}

/// checks whether the group has a specified user
pub async fn group_has_member(pool: &SqlitePool, group_id: i64, user_id: i64) -> Result<bool> {
    let query_result =
        sqlx::query("SELECT * FROM group_membership WHERE group_id = ? AND user_id = ? LIMIT 1;")
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    match query_result {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// returns true if the user was added successfully, returns false if the user is already
/// a member of the group
pub async fn add_group_membership(pool: &SqlitePool, group_id: i64, user_id: i64) -> Result<bool> {
    let already_has_member = group_has_member(pool, group_id, user_id).await?;

    if already_has_member {
        Ok(false)
    } else {
        sqlx::query("INSERT INTO group_membership (group_id, user_id) VALUES (?, ?);")
            .bind(group_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(true)
    }
}

/// returns true if the user was removed successfully, returns false if the user is not
/// a member of the group
pub async fn remove_group_membership(
    pool: &SqlitePool,
    group_id: i64,
    user_id: i64,
) -> Result<bool> {
    let already_has_member = group_has_member(pool, group_id, user_id).await?;

    if already_has_member {
        sqlx::query("DELETE FROM group_membership WHERE group_id = ? AND user_id = ?;")
            .bind(group_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(true)
    } else {
        Ok(false)
    }
}

/// note: the id of the user will not be updated.
pub async fn update_group(pool: &SqlitePool, group: &Group) -> Result<()> {
    let query_result = sqlx::query(
        r"
        UPDATE groups SET name = ?
        WHERE id = ?;",
    )
    .bind(&group.name)
    .bind(group.id)
    .execute(pool)
    .await?;

    if query_result.rows_affected() != 1 {
        bail!(
            "Update user impacted unexpected number of rows, impacted {} rows",
            query_result.rows_affected()
        )
    }

    Ok(())
}

/// deletes the group. This should also delete all the members
pub async fn delete_group(pool: &SqlitePool, group_id: i64) -> Result<()> {
    let query_result = sqlx::query(r"DELETE FROM groups WHERE id = ?")
        .bind(group_id)
        .execute(pool)
        .await?;

    if query_result.rows_affected() != 1 {
        bail!(
            "Delete user impacted unexpected number of rows, impacted {} rows",
            query_result.rows_affected()
        )
    }

    Ok(())
}

pub async fn get_group_permissions(pool: &SqlitePool, group_id: i64) -> Result<Vec<Permission>> {
    let query_result: Vec<GroupPermissions> =
        sqlx::query_as("SELECT * FROM group_permissions WHERE group_id = ?;")
            .bind(group_id)
            .fetch_all(pool)
            .await?;

    let permissions_vec: Vec<Permission> = query_result
        .iter()
        .map(|e| e.permission.as_str().try_into().unwrap())
        .collect();

    Ok(permissions_vec)
}

/// checks whether the group has a specified permission
pub async fn group_has_permission(
    pool: &SqlitePool,
    group_id: i64,
    permission: Permission,
) -> Result<bool> {
    let string_permission = String::from(permission);
    let query_result = sqlx::query(
        "SELECT * FROM group_permissions WHERE group_id = ? AND permission = ? LIMIT 1;",
    )
    .bind(group_id)
    .bind(string_permission)
    .fetch_optional(pool)
    .await?;

    match query_result {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// returns true if the permission was added successfully, returns false if the user is already
/// a member of the group
pub async fn add_group_permission(
    pool: &SqlitePool,
    group_id: i64,
    permission: Permission,
) -> Result<bool> {
    let already_has_permission = group_has_permission(pool, group_id, permission).await?;

    if already_has_permission {
        Ok(false)
    } else {
        let string_permission = String::from(permission);

        sqlx::query("INSERT INTO group_permissions (group_id, permission) VALUES (?, ?);")
            .bind(group_id)
            .bind(string_permission)
            .execute(pool)
            .await?;

        Ok(true)
    }
}

/// returns true if the permission was removed successfully, returns false if the user is not
/// a member of the group
pub async fn remove_group_permission(
    pool: &SqlitePool,
    group_id: i64,
    permission: Permission,
) -> Result<bool> {
    let already_has_permission = group_has_permission(pool, group_id, permission).await?;

    if already_has_permission {
        let string_permission = String::from(permission);

        sqlx::query("DELETE FROM group_permissions WHERE group_id = ? AND permission = ?;")
            .bind(group_id)
            .bind(string_permission)
            .execute(pool)
            .await?;

        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// I am too lazy to type to_owned everywhere
    macro_rules! s {
        ($str: tt) => {
            $str.to_owned()
        };
    }

    #[tokio::test]
    async fn user_management() {
        let mock_db = init(":memory:").await.unwrap();

        let mock_user = create_user(&mock_db, s!("username"), s!("token"), s!("expiration_date"))
            .await
            .unwrap();

        assert_eq!(
            mock_user.username, "username",
            "create_user: The new user's username should be the input"
        );
        assert_eq!(
            mock_user.token, "token",
            "create_user: The new user's token should be the input"
        );
        assert_eq!(
            mock_user.expiration_date, "expiration_date",
            "create_user: The new user's expiration date should be the input"
        );

        let fetched_user = get_user(&mock_db, mock_user.id).await.unwrap().unwrap();
        assert_eq!(
            mock_user, fetched_user,
            "get_user: The fetched user's id should be the same as the created user"
        );

        let token_user = get_user_from_token(&mock_db, s!("token"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(token_user, mock_user, "get_user_from_token: should work");

        let mut mock_user2 = create_user(
            &mock_db,
            s!("username2"),
            s!("token2"),
            s!("expiration_date2"),
        )
        .await
        .unwrap();
        let all_users = get_all_users(&mock_db).await.unwrap();
        assert_eq!(
            all_users.len(),
            2,
            "get_all_users: The vector of all users should contain the right amount of users"
        );
        assert_eq!(
            all_users[0], mock_user,
            "get_all_users: The user vector's first element should be the first user inserted"
        );
        assert_eq!(
            all_users[1], mock_user2,
            "get_all_users: The user vector's second element should be the second user inserted"
        );

        mock_user2.username = s!("username2_updated");
        update_user(&mock_db, &mock_user2).await.unwrap();
        let updated_user2 = get_user(&mock_db, mock_user2.id).await.unwrap().unwrap();
        let all_users2 = get_all_users(&mock_db).await.unwrap();
        assert_eq!(
            updated_user2.username, "username2_updated",
            "update_user: The new username should be updated"
        );
        assert_eq!(
            all_users2.len(),
            2,
            "update_user: the function should not create any more users"
        );

        delete_user(&mock_db, mock_user2.id).await.unwrap();
        let all_users3 = get_all_users(&mock_db).await.unwrap();
        assert_eq!(
            all_users3.len(),
            1,
            "delete_user: the function should delete exactly one user"
        );
        assert_eq!(
            all_users3[0], mock_user,
            "delete_user: the function should delete the correct user"
        );
    }

    #[tokio::test]
    async fn group_management() {
        let pool = init(":memory:").await.unwrap();

        let user1 = create_user(&pool, s!("username1"), s!("token1"), s!("exp1"))
            .await
            .unwrap();
        let group1 = create_group(&pool, s!("groupname1")).await.unwrap();
        assert_eq!(
            group1.name, "groupname1",
            "create_group: The output group's name should be the same as provided"
        );

        let fetched_group1 = get_group(&pool, 1).await.unwrap().unwrap();
        assert_eq!(
            fetched_group1, group1,
            "get_group: The fetched group should be equal to the created group"
        );

        let mut group2 = create_group(&pool, s!("groupname2")).await.unwrap();
        let all_groups = get_all_groups(&pool).await.unwrap();
        assert_eq!(
            all_groups.len(),
            2,
            "get_all_groups: should return the right number of groups"
        );
        assert_eq!(
            all_groups[0], group1,
            "get_all_groups: should return the right groups in the right order"
        );
        assert_eq!(
            all_groups[1], group2,
            "get_all_groups: should return the right groups in the right order"
        );

        group2.name = s!("groupname2_updated");
        update_group(&pool, &group2).await.unwrap();
        let fetched_group2 = get_group(&pool, group2.id).await.unwrap().unwrap();
        assert_eq!(fetched_group2, group2, "update_group: should work");
        let all_groups2 = get_all_groups(&pool).await.unwrap();
        assert_eq!(
            all_groups2.len(),
            2,
            "update_group: should not create or delete any groups"
        );

        let user2 = create_user(&pool, s!("username2"), s!("token2"), s!("exp2"))
            .await
            .unwrap();
        let addres1 = add_group_membership(&pool, group1.id, user1.id)
            .await
            .unwrap();
        let addres2 = add_group_membership(&pool, group1.id, user1.id)
            .await
            .unwrap();
        add_group_membership(&pool, group1.id, user2.id)
            .await
            .unwrap();
        add_group_membership(&pool, group2.id, user1.id)
            .await
            .unwrap();
        assert!(
            addres1,
            "add_group_membership: returns true if there is not already a membership"
        );
        assert!(
            !addres2,
            "add_group_membership: returns false if the membership already exists"
        );

        let user1_in_group1 = group_has_member(&pool, group1.id, user1.id).await.unwrap();
        let user2_in_group2 = group_has_member(&pool, group2.id, user2.id).await.unwrap();
        assert!(
            user1_in_group1,
            "group_has_member: returns true if a group has a user"
        );
        assert!(
            !user2_in_group2,
            "group_has_member: returns false if a group does not have a user"
        );

        let group1_members = get_group_members(&pool, group1.id).await.unwrap();
        assert_eq!(
            group1_members.len(),
            2,
            "get_group_members: should return the right number of users"
        );
        assert_eq!(
            group1_members[0], user1,
            "get_group_members: should return the right users in the right order"
        );
        assert_eq!(
            group1_members[1], user2,
            "get_group_members: should return the right users in the right order"
        );
        let user1_groups = get_user_groups(&pool, user1.id).await.unwrap();
        assert_eq!(
            user1_groups.len(),
            2,
            "get_user_groups: should return the right number of groups"
        );
        assert_eq!(
            user1_groups[0], group1,
            "get_user_groups: should return the right groups in the right order"
        );
        assert_eq!(
            user1_groups[1], group2,
            "get_user_groups: should return the right groups in the right order"
        );

        let remres1 = remove_group_membership(&pool, group1.id, user2.id)
            .await
            .unwrap();
        let remres2 = remove_group_membership(&pool, group2.id, user2.id)
            .await
            .unwrap();
        assert!(
            remres1,
            "remove_group_membership: returns true if the membership can be removed"
        );
        assert!(
            !remres2,
            "remove_group_membership: returns false if the group membership does not exist"
        );
        let group1_members2 = get_group_members(&pool, group1.id).await.unwrap();
        assert_eq!(
            group1_members2.len(),
            1,
            "remove_group_membership: should work"
        );
        assert_eq!(
            group1_members2[0], user1,
            "remove_group_membership: removes the right user"
        );

        delete_group(&pool, group1.id).await.unwrap();
        let all_groups3 = get_all_groups(&pool).await.unwrap();
        assert_eq!(all_groups3.len(), 1, "delete_group: should work");
        assert_eq!(
            all_groups3[0], group2,
            "delete_group: should delete the right group"
        );
        let fetched_group_fail = get_group(&pool, group1.id).await.unwrap();
        assert_eq!(
            fetched_group_fail, None,
            "get_group: does not get a deleted group"
        );
        let group1_members3 = get_group_members(&pool, group1.id).await.unwrap();
        assert_eq!(
            group1_members3.len(),
            0,
            "delete_group: deletes all associated memberships along with the group"
        );
        delete_user(&pool, user1.id).await.unwrap();
        let group2_members = get_group_members(&pool, group2.id).await.unwrap();
        assert_eq!(
            group2_members.len(),
            0,
            "delete_user: deletes all associated group memberships along with the user"
        );
    }

    #[tokio::test]
    async fn permissions_management() {
        let pool = init(":memory:").await.unwrap();
        let group1 = create_group(&pool, s!("groupname1")).await.unwrap();

        let permissions1 = get_group_permissions(&pool, group1.id).await.unwrap();
        assert_eq!(
            permissions1.len(),
            0,
            "get_group_permissions: returns 0 when no permissions are added"
        );
        let has_manage_content1 = group_has_permission(&pool, group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            !has_manage_content1,
            "group_has_permission: should return false if the group does not have the permission"
        );

        let permission_added = add_group_permission(&pool, group1.id, Permission::ManageContent)
            .await
            .unwrap();
        let has_manage_content2 = group_has_permission(&pool, group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            permission_added,
            "add_group_permission: returns true when the permission has been added"
        );
        assert!(has_manage_content2, "add_group_permission: works, group_has_permission: should return true if the group does have the permission");
        let permissions2 = get_group_permissions(&pool, group1.id).await.unwrap();
        assert_eq!(
            permissions2,
            vec![Permission::ManageContent],
            "get_group_permissions: returns the right thing"
        );
        let already_added = add_group_permission(&pool, group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            !already_added,
            "add_group_permission: should return false if group already has the permission"
        );

        let permission_removed =
            remove_group_permission(&pool, group1.id, Permission::ManageContent)
                .await
                .unwrap();
        assert!(
            permission_removed,
            "remove_group_permission: returns true when permission has been removed"
        );
        let already_removed = remove_group_permission(&pool, group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            !already_removed,
            "remove_group_permission: returns false when the group didn't have the permission"
        );
        let has_manage_content3 = group_has_permission(&pool, group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(!has_manage_content3, "remove_group_permission: works");
        let permissions3 = get_group_permissions(&pool, group1.id).await.unwrap();
        assert_eq!(
            permissions3.len(),
            0,
            "get_group_permissions: returns 0 when the permission has been removed"
        );
    }
}
