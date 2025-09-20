# Tinyboards

### About the project

---

Tinyboards is similar to projects like Hacker News, Lemmy, Reddit, etc. you can subscribe to boards that you are interested in, post links and discussion and then vote/comment on them as well. But unlike Hacker News and Reddit (and like Lemmy) behind the scenes a much different process is going on.

Anyone is able to run their own Tinyboards server and customize it to their own liking, and through the power of a decentralized protocol called Activitypub, Tinyboards servers are able to communicate with each other through federation (think fancy server to server email), implementing Activitypub also connects Tinyboards to a shared network of applications as well known as the Fediverse which all communicate over the protocol.

Each Tinyboards server can set it's own moderation policies and be ran however the owner sees fit, outside of corporate control and advertisements.

---



### Local Development Setup

---

1. Clone this repository wherever you like on your local PC
2. Install [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html), if it is not already installed
3. Install `libpq` or `libpq-dev` depending on your distribution, also make sure that `gcc` is installed.
4. Install [Postgresql](https://www.postgresql.org/) on your system, this will be used when the backend starts up when you test things locally
5. Create the test database on your local Postgres server (this part subject to change)

   1. `sudo -Hu postgres psql` (this will launch the Postgres CLI with the postgres user)
   2. `CREATE DATABASE tinyboards;`(creates the test db)
   3. `CREATE USER tinyboards WITH PASSWORD 'tinyboards';` (creates the database user)
   4. `\c tinyboards` and then `GRANT ALL PRIVILEGES ON DATABASE tinyboards TO tinyboards;`and then `GRANT ALL ON SCHEMA public TO tinyboards;`(grants needed privileges to the admin user)
   5. `\q` (exits psql)
6. Setup the environment variable for the database URL (needed for diesel cli)

   1. `sudo nano ~/.bashrc` then go to the bottom of the file and type the following: `export DATABASE_URL=postgresql://tinyboards:tinyboards@localhost:5432`
   2. `CTRL+X`in order to save and exit the file (type Y and press enter when it asks if you want to save)
   3. `source ~/.bashrc`(reloads the bashrc file)
7. Install Diesel CLI by opening a terminal in your IDE and then enter `cargo install diesel_cli --no-default-features --features postgres`
8. Try running all the migrations to see if the database setup is working: `diesel migration run`
9. If the migrations work then you should now have everything you need to start developing locally, you can try building the server now by using `cargo build` and if you want to try running the webserver to test things you can use `cargo run`

---



### Docker Setup

---



1. Download the [Docker setup files](https://github.com/tinyboard/tinyboards/tree/master/docker) wherever you want to setup Tinyboards
2. Make sure that you have Docker and Docker Compose installed on your distribution
3. In the same directory as your docker compose file:

   1. `mkdir -p nginx/conf`
   2. Download the [NGINX Config File](https://github.com/tinyboard/tinyboards/blob/master/docker/nginx/conf/nginx.conf) and place it inside the `conf` directory you just made
   3. `mkdir -p volumes/media && mkdir -p volumes/postgres` (these are going to be needed by the docker containers)
   4. Copy the appropriate environment file (`.env.dev.example` or `.env.prod.example`) to `.env` and configure your settings
   5. Open the `.env` file in a text editor and change the parameters to your liking, including database credentials, admin user settings, and domain configuration.
   6. Make sure to also edit the `hostname` field in the settings file as well, this is mandatory if you are running Tinyboards on a VPS, but you should be able to leave it set to localhost for local testing. This should be set to the domain of your Tinyboards (example.com)
4. After your settings are configured, you can start TinyBoards using the appropriate compose file:
   - Development: `docker-compose -f docker-compose.dev.yml up -d`
   - Production: `docker-compose -f docker-compose.prod.yml up -d`
   - Registry-based: `docker-compose -f docker-compose.registry.yml up -d`
