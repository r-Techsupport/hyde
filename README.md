# rts-cms
This project is intended as a Content Management System (CMS) for the r/Techsupport wiki (<https://rtech.support>), and is meant to provide a way
for permitted users to modify the wiki from a website.

It is composed of a frontend written in Svelte, and a backend written in Rust.

## Building
Run `build.sh` to build the front and and backend. The output will be assembled into `./target`, run `rts-cms` to start the binary.

## Running
The executable requires a few environment variables be set, see `default.env` for a full list. You may set them on the system or copy `default.env` to `.env`
and place it in a folder named `cms-data/` and customize as needed. This directory is used to store configs, the sqlite database, and the Github App private key.

## Developing
The frontend and backend can also operate in development mode.

You can run the backend with `cargo run` from the `backend` folder (note: you'll need to make an `cms-data` folder in `backend` and populate it with the required contents).

Once the backend is running, run `npm run dev` from the `frontend` folder to start the frontend.

## Testing
TODO