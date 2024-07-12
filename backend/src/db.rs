//! Database specific interfaces and abstractions

use crate::perms::Permission;
use color_eyre::{eyre::bail, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::debug;

pub const DATABASE_URL: &str = "file:hyde-data/data.db?mode=rwc";

// the ids have to be i64 because that's what sql uses
#[derive(Debug, PartialEq, Eq, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    /// The oauth2 auth token
    pub token: String,
    /// ISO-8601/RFC-3339 string
    pub expiration_date: String,
    /// The CDN url to the user's profile picture
    pub avatar_url: String,
}

#[derive(Debug, PartialEq, Eq, sqlx::FromRow, Serialize, Deserialize)]
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

/// A wrapper around the sqlite database, and how consumers should interact with the database in any capacity.
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create or connect to the database located at `DATABASE_URL`.
    pub async fn new() -> Result<Self> {
        let pool = SqlitePool::connect(DATABASE_URL).await?;

        debug!("Running SQL migrations...");
        // this should embed the migrations into the executable itself
        sqlx::migrate!("./migrations").run(&pool).await?;
        debug!("SQL migrations complete");

        Ok(Self { pool })
    }

    /// Create or connect to the database with the provided url, useful for testing so that
    /// you can initialize a database in memory.
    pub async fn from_url(url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(url).await?;

        debug!("Running SQL migrations...");
        // this should embed the migrations into the executable itself
        sqlx::migrate!("./migrations").run(&pool).await?;
        debug!("SQL migrations complete");

        Ok(Self { pool })
    }

    /// Add a new user to the database, returning the created user. This does not overwrite an existing user
    pub async fn create_user(
        &self,
        username: String,
        token: String,
        expiration_date: String,
        avatar_url: String,
    ) -> Result<User> {
        let query_results: User = sqlx::query_as(
            r"
            INSERT INTO users (username, token, expiration_date, avatar_url)
            VALUES (?, ?, ?, ?) RETURNING *;
            ",
        )
        .bind(username)
        .bind(token)
        .bind(expiration_date)
        .bind(avatar_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(query_results)
    }

    /// Returns a user from the database associated with the provided user id.
    pub async fn get_user(&self, user_id: i64) -> Result<Option<User>> {
        let query_results: Option<User> = sqlx::query_as(r"SELECT * FROM  users WHERE id = ?;")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(query_results)
    }

    /// Returns a user from the database associated with the provided
    /// authentication token.
    pub async fn get_user_from_token(&self, token: String) -> Result<Option<User>> {
        let query_results: Option<User> = sqlx::query_as(r"SELECT * FROM  users WHERE token = ?;")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;
        Ok(query_results)
    }

    /// Returns a list of all groups a user is a member of.
    pub async fn get_user_groups(&self, user_id: i64) -> Result<Vec<Group>> {
        let groups: Vec<Group> = sqlx::query_as(
            "SELECT groups.* FROM group_membership 
            RIGHT JOIN groups ON group_membership.group_id = groups.id
            WHERE group_membership.user_id = ? ORDER BY groups.id;",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    /// Returns a list of all of the permissions a user has.
    pub async fn get_user_permissions(&self, user_id: i64) -> Result<Vec<Permission>> {
        // TODO include get_user_permissions in tests
        let query_result: Vec<GroupPermissions> = sqlx::query_as(
            "SELECT DISTINCT gp.* FROM group_permissions gp
            INNER JOIN group_membership gm ON gp.group_id = gm.group_id WHERE gm.user_id = ?;",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let permissions_vec = query_result
            .into_iter()
            .map(|e| e.permission.as_str().try_into().unwrap())
            .collect();

        Ok(permissions_vec)
    }

    /// Returns a list of every user in the database.
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let query_results: Vec<User> = sqlx::query_as(r"SELECT * FROM users;")
            .fetch_all(&self.pool)
            .await?;

        Ok(query_results)
    }

    /// Refresh the database entry for the provided user based off of the user id.
    ///
    /// The ID of the user will not be updated.
    pub async fn update_user(&self, user: &User) -> Result<()> {
        let query_result = sqlx::query(
            r"
            UPDATE users SET username = ?, token = ?, expiration_date = ?
            WHERE id = ?;",
        )
        .bind(&user.username)
        .bind(&user.token)
        .bind(&user.expiration_date)
        .bind(user.id)
        .execute(&self.pool)
        .await?;

        if query_result.rows_affected() != 1 {
            bail!(
                "Update user impacted unexpected number of rows, impacted {} rows",
                query_result.rows_affected()
            )
        }

        Ok(())
    }

    /// Delete the user associated with the provided user ID from the database.
    pub async fn delete_user(&self, user_id: i64) -> Result<()> {
        let query_result = sqlx::query(r"DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if query_result.rows_affected() != 1 {
            bail!(
                "Delete user impacted unexpected number of rows, impacted {} rows",
                query_result.rows_affected()
            )
        }

        Ok(())
    }

    /// Create a group, returning the created group upon completion.
    pub async fn create_group(&self, group_name: String) -> Result<Group> {
        let query_results: Group = sqlx::query_as(
            r"
            INSERT INTO groups (name) VALUES (?) RETURNING *;
            ",
        )
        .bind(group_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(query_results)
    }

    /// Returns a group from the database associated with the provided
    /// group id.
    pub async fn get_group(&self, group_id: i64) -> Result<Option<Group>> {
        let query_results: Option<Group> =
            sqlx::query_as("SELECT * FROM groups WHERE id = ? LIMIT 1;")
                .bind(group_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(query_results)
    }

    /// Returns a list of every group in the database.
    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let query_results: Vec<Group> = sqlx::query_as(r"SELECT * FROM groups;")
            .fetch_all(&self.pool)
            .await?;

        Ok(query_results)
    }

    /// Returns a list of every member in the provided group (by id).
    pub async fn get_group_members(&self, group_id: i64) -> Result<Vec<User>> {
        let users: Vec<User> = sqlx::query_as(
            "SELECT users.* FROM group_membership 
            RIGHT JOIN users ON group_membership.user_id = users.id
            WHERE group_membership.group_id = ? ORDER BY users.id;",
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    /// Whether a user is a member of a group.
    pub async fn group_has_member(&self, group_id: i64, user_id: i64) -> Result<bool> {
        let query_result = sqlx::query(
            "SELECT * FROM group_membership WHERE group_id = ? AND user_id = ? LIMIT 1;",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match query_result {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Add a user to a group (by id).
    ///
    /// Returns true if the user was added successfully, returns false if the user is already
    /// a member of the group.
    pub async fn add_group_membership(&self, group_id: i64, user_id: i64) -> Result<bool> {
        let already_has_member = self.group_has_member(group_id, user_id).await?;

        if already_has_member {
            Ok(false)
        } else {
            sqlx::query("INSERT INTO group_membership (group_id, user_id) VALUES (?, ?);")
                .bind(group_id)
                .bind(user_id)
                .execute(&self.pool)
                .await?;

            Ok(true)
        }
    }

    /// Remove a user from a group (by id).
    ///
    /// Returns `true` if the user was removed successfully, returns `false` if the user was not
    /// a member of the provided group.
    pub async fn remove_group_membership(&self, group_id: i64, user_id: i64) -> Result<bool> {
        let already_has_member = self.group_has_member(group_id, user_id).await?;

        if already_has_member {
            sqlx::query("DELETE FROM group_membership WHERE group_id = ? AND user_id = ?;")
                .bind(group_id)
                .bind(user_id)
                .execute(&self.pool)
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Modify the database entry for the given group.
    ///
    /// The id of the group will not be updated.
    pub async fn update_group(&self, group: &Group) -> Result<()> {
        let query_result = sqlx::query(
            r"
            UPDATE groups SET name = ?
            WHERE id = ?;",
        )
        .bind(&group.name)
        .bind(group.id)
        .execute(&self.pool)
        .await?;

        if query_result.rows_affected() != 1 {
            bail!(
                "Update user impacted unexpected number of rows, impacted {} rows",
                query_result.rows_affected()
            )
        }

        Ok(())
    }

    /// Delete the provided group (by id). All users that were a member of that group will be removed from the group upon deletion.
    pub async fn delete_group(&self, group_id: i64) -> Result<()> {
        let query_result = sqlx::query(r"DELETE FROM groups WHERE id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await?;

        if query_result.rows_affected() != 1 {
            bail!(
                "Delete user impacted unexpected number of rows, impacted {} rows",
                query_result.rows_affected()
            )
        }

        Ok(())
    }

    /// Get a list of all of the permissions tied to a particular group.
    pub async fn get_group_permissions(&self, group_id: i64) -> Result<Vec<Permission>> {
        let query_result: Vec<GroupPermissions> =
            sqlx::query_as("SELECT * FROM group_permissions WHERE group_id = ?;")
                .bind(group_id)
                .fetch_all(&self.pool)
                .await?;

        let permissions_vec: Vec<Permission> = query_result
            .iter()
            .map(|e| e.permission.as_str().try_into().unwrap())
            .collect();

        Ok(permissions_vec)
    }

    /// Check whether a group has a permission.
    pub async fn group_has_permission(
        &self,
        group_id: i64,
        permission: Permission,
    ) -> Result<bool> {
        let string_permission = String::from(permission);
        let query_result = sqlx::query(
            "SELECT * FROM group_permissions WHERE group_id = ? AND permission = ? LIMIT 1;",
        )
        .bind(group_id)
        .bind(string_permission)
        .fetch_optional(&self.pool)
        .await?;

        match query_result {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Assign a group a new permission.
    ///
    /// Returns `true` if the permission was added successfully, returns `false` if the group
    /// already had that permission.
    pub async fn add_group_permission(
        &self,
        group_id: i64,
        permission: Permission,
    ) -> Result<bool> {
        let already_has_permission = self.group_has_permission(group_id, permission).await?;

        if already_has_permission {
            Ok(false)
        } else {
            let string_permission = String::from(permission);

            sqlx::query("INSERT INTO group_permissions (group_id, permission) VALUES (?, ?);")
                .bind(group_id)
                .bind(string_permission)
                .execute(&self.pool)
                .await?;

            Ok(true)
        }
    }

    /// Remove a permission from a group.
    ///
    /// Returns `true` if the permission was removed successfully, returns `false` if the group
    /// didn't have that permission in the first place.
    pub async fn remove_group_permission(
        &self,
        group_id: i64,
        permission: Permission,
    ) -> Result<bool> {
        let already_has_permission = self.group_has_permission(group_id, permission).await?;

        if already_has_permission {
            let string_permission = String::from(permission);

            sqlx::query("DELETE FROM group_permissions WHERE group_id = ? AND permission = ?;")
                .bind(group_id)
                .bind(string_permission)
                .execute(&self.pool)
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
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
        let mock_db = Database::from_url(":memory:").await.unwrap();

        let mock_user = mock_db
            .create_user(
                s!("username"),
                s!("token"),
                s!("expiration_date"),
                s!("https://foo.bar"),
            )
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
        assert_eq!(
            mock_user.avatar_url, "https://foo.bar",
            "create_user: the new user's expiration date should be the input"
        );

        let fetched_user = mock_db.get_user(mock_user.id).await.unwrap().unwrap();
        assert_eq!(
            mock_user, fetched_user,
            "get_user: The fetched user's id should be the same as the created user"
        );

        let token_user = mock_db
            .get_user_from_token(s!("token"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(token_user, mock_user, "get_user_from_token: should work");

        let mut mock_user2 = mock_db
            .create_user(
                s!("username2"),
                s!("token2"),
                s!("expiration_date2"),
                s!("https://foo.bar"),
            )
            .await
            .unwrap();
        let all_users = mock_db.get_all_users().await.unwrap();
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

        "username2_updated".clone_into(&mut mock_user2.username);
        mock_db.update_user(&mock_user2).await.unwrap();
        let updated_user2 = mock_db.get_user(mock_user2.id).await.unwrap().unwrap();
        let all_users2 = mock_db.get_all_users().await.unwrap();
        assert_eq!(
            updated_user2.username, "username2_updated",
            "update_user: The new username should be updated"
        );
        assert_eq!(
            all_users2.len(),
            2,
            "update_user: the function should not create any more users"
        );

        mock_db.delete_user(mock_user2.id).await.unwrap();
        let all_users3 = mock_db.get_all_users().await.unwrap();
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
        let mock_db = Database::from_url(":memory:").await.unwrap();

        let user1 = mock_db
            .create_user(
                s!("username1"),
                s!("token1"),
                s!("exp1"),
                s!("https://foo.bar"),
            )
            .await
            .unwrap();
        let group1 = mock_db.create_group(s!("groupname1")).await.unwrap();
        assert_eq!(
            group1.name, "groupname1",
            "create_group: The output group's name should be the same as provided"
        );

        let fetched_group1 = mock_db.get_group(group1.id).await.unwrap().unwrap();
        assert_eq!(
            fetched_group1, group1,
            "get_group: The fetched group should be equal to the created group"
        );

        let mut group2 = mock_db.create_group(s!("groupname2")).await.unwrap();
        let all_groups = mock_db.get_all_groups().await.unwrap();
        assert_eq!(
            all_groups.len(),
            3, // includes admin
            "get_all_groups: should return the right number of groups"
        );
        assert_eq!(
            all_groups[1], group1,
            "get_all_groups: should return the right groups in the right order"
        );
        assert_eq!(
            all_groups[2], group2,
            "get_all_groups: should return the right groups in the right order"
        );

        "groupname2_updated".clone_into(&mut group2.name);
        mock_db.update_group(&group2).await.unwrap();
        let fetched_group2 = mock_db.get_group(group2.id).await.unwrap().unwrap();
        assert_eq!(fetched_group2, group2, "update_group: should work");
        let all_groups2 = mock_db.get_all_groups().await.unwrap();
        assert_eq!(
            all_groups2.len(),
            3, // includes admin group
            "update_group: should not create or delete any groups"
        );

        let user2 = mock_db
            .create_user(
                s!("username2"),
                s!("token2"),
                s!("exp2"),
                s!("https://foo.bar"),
            )
            .await
            .unwrap();
        let address1 = mock_db
            .add_group_membership(group1.id, user1.id)
            .await
            .unwrap();
        let address2 = mock_db
            .add_group_membership(group1.id, user1.id)
            .await
            .unwrap();
        mock_db
            .add_group_membership(group1.id, user2.id)
            .await
            .unwrap();
        mock_db
            .add_group_membership(group2.id, user1.id)
            .await
            .unwrap();
        assert!(
            address1,
            "add_group_membership: returns true if there is not already a membership"
        );
        assert!(
            !address2,
            "add_group_membership: returns false if the membership already exists"
        );

        let user1_in_group1 = mock_db.group_has_member(group1.id, user1.id).await.unwrap();
        let user2_in_group2 = mock_db.group_has_member(group2.id, user2.id).await.unwrap();
        assert!(
            user1_in_group1,
            "group_has_member: returns true if a group has a user"
        );
        assert!(
            !user2_in_group2,
            "group_has_member: returns false if a group does not have a user"
        );

        let group1_members = mock_db.get_group_members(group1.id).await.unwrap();
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
        let user1_groups = mock_db.get_user_groups(user1.id).await.unwrap();
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

        let remres1 = mock_db
            .remove_group_membership(group1.id, user2.id)
            .await
            .unwrap();
        let remres2 = mock_db
            .remove_group_membership(group2.id, user2.id)
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
        let group1_members2 = mock_db.get_group_members(group1.id).await.unwrap();
        assert_eq!(
            group1_members2.len(),
            1,
            "remove_group_membership: should work"
        );
        assert_eq!(
            group1_members2[0], user1,
            "remove_group_membership: removes the right user"
        );

        mock_db.delete_group(group1.id).await.unwrap();
        let all_groups3 = mock_db.get_all_groups().await.unwrap();
        assert_eq!(
            all_groups3.len(),
            2, // includes admin group
            "delete_group: should work"
        );
        assert_eq!(
            all_groups3[1], group2,
            "delete_group: should delete the right group"
        );
        let fetched_group_fail = mock_db.get_group(group1.id).await.unwrap();
        assert_eq!(
            fetched_group_fail, None,
            "get_group: does not get a deleted group"
        );
        let group1_members3 = mock_db.get_group_members(group1.id).await.unwrap();
        assert_eq!(
            group1_members3.len(),
            0,
            "delete_group: deletes all associated memberships along with the group"
        );
        mock_db.delete_user(user1.id).await.unwrap();
        let group2_members = mock_db.get_group_members(group2.id).await.unwrap();
        assert_eq!(
            group2_members.len(),
            0,
            "delete_user: deletes all associated group memberships along with the user"
        );
    }

    #[tokio::test]
    async fn permissions_management() {
        let mock_db = Database::from_url(":memory:").await.unwrap();
        let group1 = mock_db.create_group(s!("groupname1")).await.unwrap();

        let permissions1 = mock_db.get_group_permissions(group1.id).await.unwrap();
        assert_eq!(
            permissions1.len(),
            0,
            "get_group_permissions: returns 0 when no permissions are added"
        );
        let has_manage_content1 = mock_db
            .group_has_permission(group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            !has_manage_content1,
            "group_has_permission: should return false if the group does not have the permission"
        );

        let permission_added = mock_db
            .add_group_permission(group1.id, Permission::ManageContent)
            .await
            .unwrap();
        let has_manage_content2 = mock_db
            .group_has_permission(group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            permission_added,
            "add_group_permission: returns true when the permission has been added"
        );
        assert!(has_manage_content2, "add_group_permission: works, group_has_permission: should return true if the group does have the permission");
        let permissions2 = mock_db.get_group_permissions(group1.id).await.unwrap();
        assert_eq!(
            permissions2,
            vec![Permission::ManageContent],
            "get_group_permissions: returns the right thing"
        );
        let already_added = mock_db
            .add_group_permission(group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            !already_added,
            "add_group_permission: should return false if group already has the permission"
        );

        let permission_removed = mock_db
            .remove_group_permission(group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            permission_removed,
            "remove_group_permission: returns true when permission has been removed"
        );
        let already_removed = mock_db
            .remove_group_permission(group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(
            !already_removed,
            "remove_group_permission: returns false when the group didn't have the permission"
        );
        let has_manage_content3 = mock_db
            .group_has_permission(group1.id, Permission::ManageContent)
            .await
            .unwrap();
        assert!(!has_manage_content3, "remove_group_permission: works");
        let permissions3 = mock_db.get_group_permissions(group1.id).await.unwrap();
        assert_eq!(
            permissions3.len(),
            0,
            "get_group_permissions: returns 0 when the permission has been removed"
        );

        let admin_permissions = mock_db.get_group_permissions(1).await.unwrap();
        assert_eq!(
            admin_permissions,
            vec![Permission::ManageContent, Permission::ManageUsers],
            "admin group should have the right permissions"
        );
    }
}
