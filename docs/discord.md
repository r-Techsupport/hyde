## Introduction
Hyde makes use of Discord OAuth2 for permission management.

## Setup (Discord)
1. Log into https://discord.com/developers
2. On the "Applications" page choose "New Application" in the top right corner
3. Once the app is created navigate to "OAuth2" in the right hand menu
4. You will be provided the "Application ID" which is "OAUTH_CLIENT_ID"
5. Click "Reset" under  "Client Secret" to be provided the value for "OAUTH_SECRET"
6. In the "Redirects" box enter the URL for your Hyde server in the following format:
    - https://domain.contoso.com/api/oauth
6. In the "OAuth2 URL Generator" box tick "identify" then select the proper URI in "Select Redirect URL".
7. The "Generated URL" will be "OAUTH_URL"