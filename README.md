# Hyde
This project is intended as a Content Management System (CMS) for Jekyll static sites. It is meant to enable users to edit a Jekyll site hosted with Github Sites from
a web browser, and provides a permission management interface for multi-user support.

It is composed of a frontend written in Svelte, and a backend written in Rust.

## Building
Run `build.sh` to build the front and and backend. The output will be assembled into `./target`, run `hyde` to start the binary.

## Running
The executable requires a few environment variables be set, see `default.env` for a full list. You may set them on the system or copy `default.env` to `.env`
and place it in a folder named `hyde-data/` and customize as needed. This directory is used to store configs, the sqlite database, and the Github App private key.

## Developing
The frontend and backend can also operate in development mode.

You can run the backend with `cargo run` from the `backend` folder (note: you'll need to make an `hyde-data` folder in `backend` and populate it with the required contents).

Once the backend is running, run `npm run dev` from the `frontend` folder to start the frontend.

It's recommended that you configure your `rust-analyzer` installation to run `clippy` instead of `check`. See <https://users.rust-lang.org/t/how-to-use-clippy-in-vs-code-with-rust-analyzer/41881/2> for a guide, set `Check On Save: Command` to `clippy`. At the very least, run `cargo clippy` before committing to make sure your code passes lint.

## Testing
To run the backend tests, navigate to `./backend`, and run `cargo test`.

To run the frontend tests, navigate to `./frontend` and run `npm test`, or `npm test:watch` for hot reload.

