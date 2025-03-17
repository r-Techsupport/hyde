# Hyde
This project is intended as a Content Management System (CMS) for Jekyll static sites. It is meant to enable users to edit a Jekyll site hosted with Github Sites from
a web browser, and provides a permission management interface for multi-user support.

It is composed of a frontend written in Svelte, and a backend written in Rust.

For more documentation, please check the `docs/` folder at the root of the repo.

## Running
The Hyde executable requires the `hyde-data` folder to be present in the same directory. See: [Populating hyde-data.](#populating-hyde-data)

You can also specify your own `.toml` config with the `hyde` binary using the  `-c`/`--config` switch:
```bash
./hyde -c ./path/to/.toml
```
You can also use relative pathing:
```bash
./hyde -c ../../path/to/.toml
```
This will ignore any `.toml` config inside `hyde-data`.

## Developing
We accept contributions, and we'll happily mentor individuals through their contributions. Feel free to ask for help, either through Github, or in the r/TechSupport discord server.

Development is supported on Windows (With [some caveats](https://github.com/r-Techsupport/hyde/issues/6)), MacOS, and Linux.

### Installing tools
To build the backend, you need to have the Rust toolchain installed (see <https://www.rust-lang.org/tools/install>). 
We currently aim to support the latest stable version of Rust. Once that's installed, `cargo` will automatically download and install the appropriate dependencies for the project when it's first built. 
The source code for the backend is located in `./backend`, so navigate there 

To build the frontend, you'll need to have the appropriate Javascript tooling installed (See <https://nodejs.org/en/download/package-manager>). 
This means Node *and* npm. We aim to use the latest stable version of Node.js (23 at the time of writing).

### Populating `hyde-data`
To keep things organized, the config file and other essential data (sqlite database, Github private key) are stored in a folder in the same directory that the code is run from. This directory is relative to the running process's current directory.

If you're running hyde via local development, place `hyde-data` folder inside `backend` folder

Hyde expects:

- `hyde-data/config.toml` file with all configuration options. `./config.example.toml` is the example config file, copy it to `hyde-data/config.toml`.
Hyde will accept any `.toml` file in that directory, but `config.toml` is a good choice.
- `hyde-data/key.pem`, the private key linked to your Github Application deployment.
- `hyde-data/data.db` - If not created, this file will be automatically created and stores permissions and user account info.

### Running the project in development mode
You can run the backend with `cargo run` from the `backend` folder. This will compile and launch a debug executable, listening on port 8080.

Once the backend is running, in a separate terminal window, run `npm run dev` from the `frontend` folder to start the frontend, listening on `localhost:5173`, viewable from your web browser.

It's recommended that you configure your `rust-analyzer` installation to run `clippy` instead of `check`. 
See <https://users.rust-lang.org/t/how-to-use-clippy-in-vs-code-with-rust-analyzer/41881/2> for a guide, set `Check On Save: Command` to `clippy`. 
At the very least, run `cargo clippy` before committing to make sure your code passes lint.

## Building

### Production
Build scripts exist for Windows (`./build.ps1`) and *nix (`./build.sh`). This will detail what a production build looks like for `hyde`.

The written build scripts should assemble the final product into `./target`, consisting of:

- `hyde`(`.exe`): The actual executable, built by running `cargo run --release`. The generated executable is copied from `./backend/target/release/[HYDE-EXECUTABLE-NAME]` to `./target/hyde`(`.exe`). 
This executable will serve the frontend files stored in `web`, so there's only a single process running. It listens on port `8080` by default, but this is configurable via the `-p`/`--port` command line option.
- `web` directory: Svelte is configured to compile the frontend into a collection of static files, which can be generated by running `npm run build` from `./frontend`. 
Those are copied from `./target/frontend/build/` into `./target/web/`. In that directory, you'll find the relevant HTML, CSS, JavaScript, and any assets used on the site. 
Svelte is also configured to include Brotli and Gzipped versions of those files to reduce bundle size.

There are two ways to copy `hyde-data` folder from `./backend/` to `./target`. First is to manually move the folder, second is to use the build scripts to do it for you:

- Linux: `build.sh -c <path_to_hyde_data>`

- Windows `build.ps1 -C <path_to_hyde_data>` 

Behaviors can differ between development and production builds. Notably, in release mode, the application requires `https` to function.

Hyde's logging can be configured by setting the `RUST_LOG` environment variable or using the `-v`/`--verbosity` command line flag. Possible values are: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, and `OFF`.

On first start, Hyde will check if you have a git repository for your wiki at the relative folder `./repo`. 
If it is not there, it will clone the repo based on the `repo_url` supplied in the `.toml` config. If it exists
then it will pull upstream changes.

The final product can be found by navigating to <http://localhost:8080> in your web browser.

### Building a containerized version of the project
This does not require that you have language tooling installed (Rust, JavaScript), only requiring an OCI implementation of your choice.

Docker:
```sh
docker build -t hyde .
```

Podman:
```sh
podman build -t hyde .
```

## Testing
To run the backend tests, navigate to `./backend`, and run `cargo test`.

To run the frontend tests, navigate to `./frontend` and run `npm test`, or `npm test:watch` for hot reload.
