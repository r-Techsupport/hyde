| **[files]**           | **[discord]**             | **[oauth.discord]**  | **[oauth.github]**   | **[database]** |
|-----------------------|---------------------------|----------------------|----------------------|----------------|
| asset_path = `string` | admin_username = `string` | client_id = `string` | client_id = `string` | url = `string` |
| docs_path = `string`  |                           | secret = `string`    |                      |                |
| repo_path = `string`  |                           | url = `string`       |                      |                |
| repo_url = `string`   |                           | token_url = `string` |                      |                |

## Descriptions
### Files
- `asset_path`: Location of the markdown file relative to the root of the project
- `docs_path`: Location of the assets files relative to the root of the project
- `repo_path`: Location of where the jekyll repository will be pulled and used
- `repo_url`: URL of the jekyll repository to use

### Discord
- `admin_username`: Discord username of the administrator account

### OAuth.discord
See: [Hyde Discord Documentation](discord.md)
- `client_id`: Discord OAuth2 Client ID.
- `secret`: Discord Application Secret.
- `url`: Generated Discord Application Scope URL
- `token_url`: OAuth2 Token URL. By default can be: `https://discord.com/api/oauth2/token`

### OAuth.github
See: [Hyde GitHub Documentation](github.md)
- `client_id`: GitHub Application Client ID

### Database
- `url`: Database url for Hyde to use