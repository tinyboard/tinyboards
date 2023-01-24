#!/bin/bash
INSTALL_LOCATION="$(pwd)"

# Load variables from environment file
source .env

cat > $INSTALL_LOCATION/nginx/conf/nginx.conf <<EOF
  upstream tinyboards-frontend {
    server tinyboards-fe:3000;
  }
  upstream tinyboards-backend{
    server tinyboards-be:8536;
  }

  server {
    listen 80;
    listen [::]:80;
    server_name $SERVER_NAME;
EOF

# Check if SSL is enabled
if [ "$ENVIRONMENT" = "prod" ]; then
echo "****SSL IS ENABLED****"
    # Add SSL configuration to the file
    cat >> $INSTALL_LOCATION/nginx/conf/nginx.conf <<EOF
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }
    location / {
        return 301 https://$host$request_uri;
    }
}
  server {
    listen 443 ssl default_server;
    server_name example.com;
    ssl_certificate /etc/ssl/fullchain.pem;
    ssl_certificate_key /etc/ssl/privkey.pem;
EOF
fi

cat >> $INSTALL_LOCATION/nginx/conf/nginx.conf <<EOF

# media-server
    location ~^/(image) {
      expires 120d;
      add_header Pragma "public";
      add_header Cache-Control "public";
      proxy_pass "http://tinyboards-backend";
      proxy_http_version 1.1;
      proxy_set_header Upgrade \$http_upgrade;
      proxy_set_header Connection "upgrade";

      # IP forwarding headers
      proxy_set_header X-Real-IP \$remote_addr;
      proxy_set_header Host \$host;
      proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;

    }

    # frontend
    location / {

      set \$proxpass "http://tinyboards-frontend";
      if (\$http_accept ~ "application/activity+json") {
        set \$proxpass "http://tinyboards-frontend";
      }
      if (\$http_accept ~ "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"") {
        set \$proxpass "http://tinyboards-frontend";
      }
      if (\$http_accept ~ POST) {
        set \$proxpass "http://tinyboards-frontend";
      }
      proxy_pass \$proxpass;

      rewrite ^(.+)/+\$ \$1 permanent;

      # Send actual client IP upstream
      proxy_set_header X-Real-IP \$remote_addr;
      proxy_set_header Host \$host;
      proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    }

      # backend
      location ~ ^/(assets|api|.well-known) {
        proxy_pass "http://tinyboards-backend";
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";

        # Add IP forwarding headers
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header Host \$host;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    }
}
EOF

#generate tinyboards.hjson

cat > $INSTALL_LOCATION/tinyboards.hjson <<EOF
{
  # settings related to the postgresql database
  database: {
    # Username to connect to postgres
    user: "$POSTGRES_USER"
    # Password to connect to postgres
    password: "$POSTGRES_PASSWORD"
    # Host where postgres is running
    host: "postgres"
    # Port where postgres can be accessed
    port: 5432
    # Name of the postgres database for tinyboards
    database: "$POSTGRES_DB"
    # Maximum number of active sql connections
    pool_size: 5
  }
  # rate limits for various user actions, by user ip
  rate_limit: {
    # Maximum number of messages created in interval
    message: $MESSAGE_LIMIT
    # Interval length for message limit, in seconds
    message_per_second: $MESSAGE_PER_SECOND
    # Maximum number of posts created in interval
    post: $POST_LIMIT
    # Interval length for post limit, in seconds
    post_per_second: $POST_PER_SECOND
    # Maximum number of registrations in interval
    register: $REGISTER_LIMIT
    # Interval length for registration limit, in seconds
    register_per_second: $REGISTER_PER_SECOND
    # Maximum number of image uploads in interval
    image: $IMAGE_LIMIT
    # Interval length for image uploads, in seconds
    image_per_second: $IMAGE_PER_SECOND
    # Maximum number of comments created in interval
    comment: $COMMENT_LIMIT
    # Interval length for comment limit, in seconds
    comment_per_second: $COMMENT_PER_SECOND
    search: $SEARCH_LIMIT
    # Interval length for search limit, in seconds
    search_per_second: $SEARCH_PER_SECOND
  }
  captcha: {
    # Whether captcha is required for signup
    enabled: $CAPTCHA_ENABLED
    # Can be easy, medium, or hard
    difficulty: "$CAPTCHA_DIFFICULTY"
  }
  # Email sending configuration. All options except login/password are mandatory
  email: {
    # Hostname and port of the smtp server
    smtp_server: "$SMTP_SERVER"
    # Login name for smtp server
    smtp_login: "$SMTP_LOGIN"
    # Password to login to the smtp server
    smtp_password: "$SMTP_PASSWORD"
    # Address to send emails from, eg noreply@your-server.com
    smtp_from_address: "$SMTP_FROM_ADDRESS"
    # Whether or not smtp connections should use tls. Can be none, tls, or starttls
    tls_type: "$TLS_TYPE"
  }
  # Parameters for automatic configuration of new server (only used at first start)
  setup: {
    # Username for the admin user
    admin_username: "$ADMIN_USER"
    # Password for the admin user. It must be at least 10 characters.
    admin_password: "$ADMIN_PASSWORD"
    # Name of the site (can be changed later)
    site_name: "$SITE_NAME"
  }
  # the domain name of your server (mandatory)
  hostname: "localhost"
  # Address where tinyboards should listen for incoming requests
  bind: "0.0.0.0"
  # Port where tinyboards should listen for incoming requests
  port: 8536
  # Whether the site is available over TLS. Needs to be true for federation to work.
  tls_enabled: true
  # Maximum length of local community and user names
  name_max_length: $NAME_MAX_LENGTH
  # The salt suffix used for making password hashes (it still uses a UUID along with this suffix)
  salt_suffix: "$SALT_SUFFIX"
  # Environment where the code is being ran (prod or dev)
  environment: "$ENVIRONMENT"

  # pictrs host
  pictrs_url: "http://pictrs:8080"
}
EOF

#generate docker-compose files
if [ "$ENVIRONMENT" = "prod" ]; then
  COMPOSE_FILE="docker-compose.yml"
else
  COMPOSE_FILE="docker-compose.dev.yml"
fi

echo "****${ENVIRONMENT} ENVIRONMENT****"

  # Create a docker-compose.yml file if prod is true
cat > $INSTALL_LOCATION/$COMPOSE_FILE <<EOF
version: '3.3'

services:
EOF

#nginx service

  # use dev nginx service
  cat >> $INSTALL_LOCATION/$COMPOSE_FILE <<EOF
  nginx:
    image: nginx:1-alpine
    ports:
      - "$HTTP_PORT:80"
      - "$HTTPS_PORT:443"
    volumes:
      - ./nginx/conf/:/etc/nginx/conf.d
      - /etc/ssl:/etc/ssl/
    networks:
      tinyboards:
        aliases:
          - $SERVER_NAME
    restart: always
    depends_on:
      - tinyboards
      - tinyboards-fe
EOF

#tinyboards-be
if [ "$ENVIRONMENT" = "prod" ]; then
echo "****WILL PULL IMAGES FROM DOCKERHUB****"
  # pull prod image from dockerhub
cat >> $INSTALL_LOCATION/docker-compose.yml <<EOF
  tinyboards:
    image: kronusdev/tinyboards-be:latest
    ports:
      - "127.0.0.1:$BE_PORT_A:8536"
      - "127.0.0.1:$BE_PORT_B:6669"
    restart: always
    environment:
      - RUST_LOG="info"
    links:
      - postgres
    volumes:
      - ./tinyboards.hjson:/config/defaults.hjson
    networks:
      tinyboards:
        aliases:
          - tinyboards-be
    depends_on:
      - postgres
      - pictrs
EOF
else
  # dev - build from source
  cat >> $INSTALL_LOCATION/docker-compose.dev.yml <<EOF
  tinyboards:
    image: tinyboards
    ports:
      - "127.0.0.1:$BE_PORT_A:8536"
      - "127.0.0.1:$BE_PORT_B:6669"
    restart: always
    environment:
      - RUST_LOG="info"
    build:
      context: ../
      dockerfile: ./docker/Dockerfile
    links:
      - postgres
    volumes:
      - ./tinyboards.hjson:/config/defaults.hjson
    networks:
      tinyboards:
        aliases:
          - tinyboards-be
    depends_on:
      - postgres
      - pictrs
EOF
fi
#tinyboards-fe
if [ "$ENVIRONMENT" = "prod" ]; then
echo "****TINYBOARDS-FE PULL FROM DOCKERHUB ****"
  # pull prod image from dockerhub
cat >> $INSTALL_LOCATION/docker-compose.yml <<EOF
  tinyboards-fe:
    image: kronusdev/tinyboards-fe:latest
    ports:
      - "127.0.0.1:$FE_PORT:3000"
    restart: always
    networks:
      tinyboards:
        aliases:
          - tinyboards-fe
    depends_on:
      - tinyboards
EOF
else
  # dev - build from source
echo "****WILL BUILD FROM SOURCE****"
  cat >> $INSTALL_LOCATION/docker-compose.dev.yml <<EOF
  tinyboards-fe:
    image: tinyboards-fe
    ports:
      - "127.0.0.1:$FE_PORT:3000"
    restart: always
    build:
      context: ../../tinyboards-fe
      dockerfile: Dockerfile
    volumes:
      - ../../tinyboards-fe/package.json:/usr/src/app/package.json
    networks:
      tinyboards:
        aliases:
          - tinyboards-fe
    depends_on:
      - tinyboards
EOF
fi

#pictrs
cat >> $INSTALL_LOCATION/$COMPOSE_FILE <<EOF
  pictrs:
    image: asonix/pictrs:0.4.0-beta.7
    # needs to match the pictrs url in the config hjson
    hostname: pictrs
    ports:
      - "127.0.0.1:$PICTRS_PORT:8080"
    networks:
      tinyboards:
        aliases:
          - pictrs
    environment:
      - PICTRS__API_KEY=$API_KEY
    user: root
    volumes:
      - ./volumes/pictrs:/mnt
    restart: always
EOF


#Postgres

cat >> $INSTALL_LOCATION/$COMPOSE_FILE <<EOF
  postgres:
    image: postgres:14-alpine
    ports:
      # use a different port so it doesn't conflict with postgres running on the host
      - "127.0.0.1:$POSTGRES_PORT:5432"
    environment:
      - POSTGRES_USER=$POSTGRES_USER
      - POSTGRES_PASSWORD=$POSTGRES_PASSWORD
      - POSTGRES_DB=$POSTGRES_DB
    volumes:
      - ./volumes/postgres:/var/lib/postgresql/data
    networks:
      tinyboards:
        aliases:
          - postgres
    restart: always
    command: ["postgres", "-c", "session_preload_libraries=auto_explain", "-c", "auto_explain.log_min_duration=5ms", "-c", "auto_explain.log_analyze=true"]
EOF

#volumes and networks

cat >> $INSTALL_LOCATION/$COMPOSE_FILE <<EOF

networks:
  tinyboards: {}
EOF

#docker-scripts

if [ "$ENVIRONMENT" = "prod" ]; then
cat > $INSTALL_LOCATION/docker-start.sh <<EOF
#!/bin/sh

# This script uses a docker file that builds with musl, and runs on linux alpine
# Its a bit slower for development than the volume mount.

set -e

sudo docker-compose down
sudo docker-compose build --no-cache
sudo docker-compose up
EOF
else
  cat > $INSTALL_LOCATION/docker-start-dev.sh <<EOF
sudo docker-compose -f docker-compose.dev.yml up
EOF
fi

#Override constants.js

if [ "$ENVIRONMENT" = "prod" ]; then
cat > ../../tinyboards-fe/server/constants.js <<EOF
export const baseURL = "https://$SERVER_NAME/api/v1";
EOF
else
  cat > ../../tinyboards-fe/server/constants.js <<EOF
export const baseURL = "http://$SERVER_NAME:$HTTP_PORT/api/v1";
EOF
fi

#add to hosts file
if [ "$ENVIRONMENT" = "dev" ]; then
echo "127.0.0.1  $SERVER_NAME" >> /etc/hosts
fi

#make scripts executable

if [ "$ENVIRONMENT" = "prod" ]; then
chmod u+x docker-start.sh
else
chmod u+x docker-start-dev.sh
fi

if [ "$ENVIRONMENT" = "prod" ]; then
echo "****PROCESS COMPLETE, RUN "docker-start.sh" TO BUILD PROD ENVIRONMENT****"
else
echo "****PROCESS COMPLETE, RUN "docker-start-dev.sh" TO BUILD DEV ENVIRONMENT****"
fi