## Introduction
Hyde requires a [Github App](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app) private key to function so that it
can interact with the wiki repository. This enables functionality like pushing commits, or creating/managing PRs.

## Creating a Github App
This should be done at either the User level, or Organization level depending on your use case.

Follow the [Github Documentation](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/registering-a-github-app) for creating a GithubApp

### Repository Permissions Required
 - Metadata: Read only
 - Contents: Read and write

### Webhook URL
Under the Webhook header,
set the Webhook URL to `[YOUR_HYDE_URL]/api/hooks/github`.  As an example, if your URL was `https://hyde.rtech.support`, your Webhook URL would be `https://hyde.rtech.support/api/hooks/github`. This is done so that Hyde can automatically pull new changes when they're pushed to Github.

The Webhook Secret value is left empty.

### Notes
- If you want to use this in an organization or on a repo you do not own you must tick "Any Account" under "Where can this GitHub App be installed?"

## Installing the GithubApp
After creating your app you will be taken to its Github page.
1. Choose "Install App" on the right side of the page and 
2. Choose "Install" on the account you wish to use with Hyde. 
3. Select the specific repository you want to use with Hyde.

## Generating a private key
Follow this section of [Github's documentation](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/managing-private-keys-for-github-apps#generating-private-keys) to generate a private key for your app.

Save the generated private key in the `hyde-data/` directory  as `key.pem`.