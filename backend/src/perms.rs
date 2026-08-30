//! User permissions for the wiki (manage content, manage users, et cetera)
//!
//! Every mutating and read operation exposed by [`crate::git::Interface`] gets its own
//! [`Permission`] variant, so that groups can be granted access on a per-operation basis


use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Permission {
    ManageUsers,

    /// Corresponds to [`crate::git::Interface::get_doc`]
    GetDoc,
    /// Corresponds to [`crate::git::Interface::get_asset`]
    GetAsset,
    /// Corresponds to [`crate::git::Interface::get_doc_tree`]
    GetDocTree,
    /// Corresponds to [`crate::git::Interface::get_asset_tree`]
    GetAssetTree,
    /// Corresponds to [`crate::git::Interface::get_current_branch`]
    GetCurrentBranch,

    /// Corresponds to [`crate::git::Interface::put_doc`]
    PutDoc,
    /// Corresponds to [`crate::git::Interface::put_asset`]
    PutAsset,
    /// Corresponds to [`crate::git::Interface::delete_doc`]
    DeleteDoc,
    /// Corresponds to [`crate::git::Interface::delete_asset`]
    DeleteAsset,

    /// Corresponds to [`crate::git::Interface::git_add`]
    GitAdd,
    /// Corresponds to [`crate::git::Interface::git_commit`]
    GitCommit,
    /// Corresponds to [`crate::git::Interface::git_push`]
    GitPush,
    /// Corresponds to [`crate::git::Interface::pull`]
    Pull,
    /// Corresponds to [`crate::git::Interface::git_pull_branch`]
    GitPullBranch,
    /// Corresponds to [`crate::git::Interface::checkout_or_create_branch`]
    CheckoutOrCreateBranch,
    /// Corresponds to [`crate::git::Interface::reclone`]
    Reclone,
}

impl From<Permission> for String {
    fn from(value: Permission) -> Self {
        match value {
            Permission::ManageUsers => "ManageUsers",

            Permission::GetDoc => "GetDoc",
            Permission::GetAsset => "GetAsset",
            Permission::GetDocTree => "GetDocTree",
            Permission::GetAssetTree => "GetAssetTree",
            Permission::GetCurrentBranch => "GetCurrentBranch",

            Permission::PutDoc => "PutDoc",
            Permission::PutAsset => "PutAsset",
            Permission::DeleteDoc => "DeleteDoc",
            Permission::DeleteAsset => "DeleteAsset",

            Permission::GitAdd => "GitAdd",
            Permission::GitCommit => "GitCommit",
            Permission::GitPush => "GitPush",
            Permission::Pull => "Pull",
            Permission::GitPullBranch => "GitPullBranch",
            Permission::CheckoutOrCreateBranch => "CheckoutOrCreateBranch",
            Permission::Reclone => "Reclone",
        }
        .to_string()
    }
}

impl TryInto<Permission> for &str {
    type Error = &'static str;
    fn try_into(self) -> Result<Permission, Self::Error> {
        match self {
            "ManageUsers" => Ok(Permission::ManageUsers),

            "GetDoc" => Ok(Permission::GetDoc),
            "GetAsset" => Ok(Permission::GetAsset),
            "GetDocTree" => Ok(Permission::GetDocTree),
            "GetAssetTree" => Ok(Permission::GetAssetTree),
            "GetCurrentBranch" => Ok(Permission::GetCurrentBranch),

            "PutDoc" => Ok(Permission::PutDoc),
            "PutAsset" => Ok(Permission::PutAsset),
            "DeleteDoc" => Ok(Permission::DeleteDoc),
            "DeleteAsset" => Ok(Permission::DeleteAsset),

            "GitAdd" => Ok(Permission::GitAdd),
            "GitCommit" => Ok(Permission::GitCommit),
            "GitPush" => Ok(Permission::GitPush),
            "Pull" => Ok(Permission::Pull),
            "GitPullBranch" => Ok(Permission::GitPullBranch),
            "CheckoutOrCreateBranch" => Ok(Permission::CheckoutOrCreateBranch),
            "Reclone" => Ok(Permission::Reclone),

            _ => Err("Not a valid permission level"),
        }
    }
}