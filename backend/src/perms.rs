//! User permissions for the wiki (manage content, manage users, et cetera)

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Permission {
    ManageContent, // TODO
    ManageUsers,
    // TODO: Submit for review
}

impl From<Permission> for String {
    fn from(value: Permission) -> Self {
        match value {
            Permission::ManageContent => "ManageContent",
            Permission::ManageUsers => "ManageUsers",
        }
        .to_string()
    }
}

impl TryInto<Permission> for &str {
    type Error = &'static str;
    fn try_into(self) -> Result<Permission, Self::Error> {
        match self {
            "ManageContent" => Ok(Permission::ManageContent),
            "ManageUsers" => Ok(Permission::ManageUsers),
            _ => Err("Not a valid permission level"),
        }
    }
}
