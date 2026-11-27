-- Add migration script here
-- A default group that brand new users are added to when they first log in. It lets them
-- read the wiki and save edits, but deliberately does NOT include CheckoutOrCreateBranch,
-- which is what pins a user to a branch named after them. Grant that permission to a
-- user's group to let them work on any branch.
--
-- Looked up by name rather than assuming the new group's id.
INSERT INTO groups ( name )
	SELECT 'Contributor' WHERE NOT EXISTS ( SELECT 1 FROM groups WHERE name = 'Contributor' );

INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GetDoc' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GetAsset' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GetDocTree' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GetAssetTree' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GetCurrentBranch' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'PutDoc' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GitAdd' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GitCommit' FROM groups WHERE name = 'Contributor';
INSERT INTO group_permissions ( group_id, permission ) SELECT id, 'GitPush' FROM groups WHERE name = 'Contributor';
