//! User permissions for the wiki (manage content, manage users, et cetera)

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Permission {
    ManageContent, // TODO
    ManageUsers,
    ManageBranches,
    // TODO: Submit for review
}

impl From<Permission> for String {
    fn from(value: Permission) -> Self {
        match value {
            Permission::ManageContent => "ManageContent",
            Permission::ManageUsers => "ManageUsers",
            Permission::ManageBranches => "ManageBranches",
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
            "ManageBranches" => Ok(Permission::ManageBranches),
            _ => Err("Not a valid permission level"),
        }
    }
}
