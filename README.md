# Porpl Backend

Local Installation for Development:

1. Install Postgres 14 on your OS - https://www.postgresql.org/download/

2. Install Rust on your OS - https://www.rust-lang.org/tools/install/

3. Clone this repository to your system using Git

4. Open up this folder on your system and install diesel-cli by doing `cargo install diesel_cli`

5. Create a file ending in `.hjson` wherever you want on your computer and then copy the contents of `defaults.hjson` into it, change the database username and database name to whatever you have setup on your Postgres database and then change the hostname variable from `unset` to `localhost`, this is the config file for Porpl and you tell Porpl to use it by setting up a environment variable called `PORPL_CONFIG_LOCATION` inside of your `.env` file in step 6.

6. Create a `.env` file on the root level of the folder, you will need to manually add a couple of environment variables to get this to work (Example below).

```
DATABASE_URL=postgres://localhost:5432
SALT_SUFFIX=somesalt
PORPL_CONFIG_LOCATION=/path/to/porpl/config/file.hjson
```
7. To build & run the code for local testing simply go into terminal in this folder and type `cargo run`, this command both compiles the source code and starts running the webserver on `http://127.0.0.1:8536` so you can begin testing the API. The code will also automatically run all of the database migrations as well.
