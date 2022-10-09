# Porpl Backend

Local Installation for Development:

1. Install Postgres 14 on your OS - https://www.postgresql.org/download/

2. Install Rust on your OS - https://www.rust-lang.org/tools/install/

3. Clone this repository to your system using Git

4. Open up this folder on your system and install diesel-cli by doing `cargo add diesel-cli`

5. Create a `.env` file on the root level of the folder, you will need to manually add a couple of environment variables to get this to work (Example below).

```
DATABASE_URL=postgres://localhost:5432
SALT_SUFFIX=somesalt
MASTER_KEY=dudeporpllmao
```
6. Run the database migrations to setup the backend that Porpl will be using, this should be ran as `diesel migration run` which will setup everything on the database you specify in the .env file

7. To build & run the code for local testing simply go into terminal in this folder and type `cargo run`, this command both compiles the source code and starts running the webserver on `http://127.0.0.1:8080` so you can begin testing the API
