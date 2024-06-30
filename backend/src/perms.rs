//! User permissions for the wiki (manage content, manage users, et cetera)

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub enum Permission {
    ManageContent, // TODO
    ViewUsers,
    ManageUsers,
    // TODO: Submit for review
}

impl From<Permission> for String {
    fn from(value: Permission) -> Self {
        match value {
            Permission::ManageContent => "manage_content",
            Permission::ViewUsers => "view_users",
            Permission::ManageUsers => "manage_users",
        }
        .to_string()
    }
}

impl TryInto<Permission> for &str {
    type Error = &'static str;
    fn try_into(self) -> Result<Permission, Self::Error> {
        match self {
            "manage_content" => Ok(Permission::ManageContent),
            "view_users" => Ok(Permission::ViewUsers),
            "manage_users" => Ok(Permission::ManageUsers),
            _ => Err("Not a valid permission level"),
        }
    }
}
