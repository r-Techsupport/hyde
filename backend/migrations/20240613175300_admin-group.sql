-- Add migration script here
INSERT INTO groups ( name ) VALUES ( "Admin" ); -- this *should* be group 1
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "ManageContent" );
INSERT into group_permissions ( group_id, permission ) VALUES ( 1, "ManageUsers" );
