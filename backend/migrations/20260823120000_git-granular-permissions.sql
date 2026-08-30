-- Add migration script here
-- Replace the coarse `ManageContent`/`ManageBranches` permissions with one permission
-- per git.rs function, so groups can be granted access at a finer grain.
-- `ManageUsers` is untouched, it's unrelated to git.rs.
DELETE FROM group_permissions WHERE group_id = 1 AND permission IN ( "ManageContent", "ManageBranches" );

-- Reads
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GetDoc" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GetAsset" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GetDocTree" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GetAssetTree" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GetCurrentBranch" );

-- Content mutation
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "PutDoc" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "PutAsset" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "DeleteDoc" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "DeleteAsset" );

-- Version control
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GitAdd" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GitCommit" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GitPush" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "Pull" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "GitPullBranch" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "CheckoutOrCreateBranch" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "Reclone" );