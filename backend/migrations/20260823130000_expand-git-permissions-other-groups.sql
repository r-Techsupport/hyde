-- Add migration script here
-- Extends 20260823120000_git-granular-permissions.sql, which already handled group 1

-- (ManageContent -> content mutation + git plumbing)
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "PutDoc" FROM group_permissions WHERE permission = "ManageContent" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "PutAsset" FROM group_permissions WHERE permission = "ManageContent" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "DeleteDoc" FROM group_permissions WHERE permission = "ManageContent" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "DeleteAsset" FROM group_permissions WHERE permission = "ManageContent" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "GitAdd" FROM group_permissions WHERE permission = "ManageContent" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "GitCommit" FROM group_permissions WHERE permission = "ManageContent" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "GitPush" FROM group_permissions WHERE permission = "ManageContent" AND group_id != 1;

-- (ManageBranches -> branch/VCS operations)
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "CheckoutOrCreateBranch" FROM group_permissions WHERE permission = "ManageBranches" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "GitPullBranch" FROM group_permissions WHERE permission = "ManageBranches" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "Pull" FROM group_permissions WHERE permission = "ManageBranches" AND group_id != 1;
INSERT INTO group_permissions ( group_id, permission )
	SELECT group_id, "Reclone" FROM group_permissions WHERE permission = "ManageBranches" AND group_id != 1;

-- Remove every remaining coarse permission row, for every group (group 1's were
-- already removed by the prior migration; this catches everyone else).
DELETE FROM group_permissions WHERE permission IN ( "ManageContent", "ManageBranches" );