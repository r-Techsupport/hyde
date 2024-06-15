## Introduction
Hyde requires a (Github App)[https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app] private key to function so that it
can interact with the wiki repository. This enables functionality like pushing commits, or creating/managing PRs.

See (Github's documentation)[https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/managing-private-keys-for-github-apps#generating-private-keys].
Save the generated private key in the `hyde-data/` directory  as `key.pem`.

The generated key needs the *"Access: Read and Write"* permission for Contents.

<https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/choosing-permissions-for-a-github-app#choosing-permissions-for-git-access>
> If you want your app to use an installation or user access token to authenticate for HTTP-based Git access, you should request the "Contents" repository permission.

After creating your Github App, make note of the client ID, you'll need it later.

Then, you'll install the app under the org, and configure repository access to "Only select repositories". You'll select the wiki, and save.