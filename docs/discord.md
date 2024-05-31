## Introduction
`rts-crm` makes use of Discord OAuth2 for permission management.

## Setup (Discord)
First go to <https://discord.com/developers>, and create a new application.

Populate the `OAUTH_CLIENT_ID` field of your `.env` file, then populate the `OAUTH_SECRET` field of your `.env` file. You'll need to add a URI Redirect in the developer dashboard, this is where the user will be sent after authenticating, and the `rts-crm` backend is configured to handle oauth2 requests from `/api/oauth`. For development purposes, you can use `http://localhost:8080/api/oauth`.
