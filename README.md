# TinyBoards Backend

# NOTE: this is a severely outdated readme, we need to update this at some point because the deployment method has changed a lot 

## Local Installation for Development: 

1. Install Postgres 14 on your OS - https://www.postgresql.org/download/

2. Install Rust on your OS - https://www.rust-lang.org/tools/install/

3. Clone this repository to your system using Git

4. Open up this folder on your system and install diesel-cli by doing `cargo install diesel_cli`

5. Create a file ending in `.hjson` wherever you want on your computer and then copy the contents of `defaults.hjson` into it, change the database username and database name to whatever you have setup on your Postgres database and then change the hostname variable from `unset` to `localhost`, this is the config file for TinyBoards and you tell TinyBoards to use it by setting up a environment variable called `TB_CONFIG_LOCATION` inside of your `.env` file in step 6.

6. Create a `.env` file on the root level of the folder, you will need to manually add a couple of environment variables to get this to work (Example below).

```
DATABASE_URL=postgres://localhost:5432
SALT_SUFFIX=somesalt
TB_CONFIG_LOCATION=/path/to/tinyboards/config/file.hjson
```
7. To build & run the code for local testing simply go into terminal in this folder and type `cargo run`, this command both compiles the source code and starts running the webserver on `http://127.0.0.1:8536` so you can begin testing the API. The code will also automatically run all of the database migrations as well.


## Run TinyBoards with Docker

1. Have docker and docker-compose installed on your OS.

2. Clone this repository, and [tinyboards-fe](https://git.tinyboards.net/TinyBoards/tinyboards-fe), into a folder on your local machine.

3. Open up this repository in terminal, and change directory to `docker` with `cd docker`

4. Rename `env.example` to `.env` by running `cp env.example .env` and adjust `.env` as desired.

5. Run `./docker.sh` to generate configuration files.

6. Run the start script with `./docker-start.sh`, this command should reference the `docker-compose.yml` file within the `docker` directory and then build, compile, and launch everything that TinyBoards needs.

7. You should be able to turn off TinyBoards by using `CTRL + C` and docker-compose will spin everything down.


## NOTE: while developing we will have a dev user you can use to login to the docker build with, username = dev_user & password = password
