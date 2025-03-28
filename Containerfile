# builder base image
FROM rust:1.85-alpine3.21 AS builder

# add required packages
RUN apk add --no-cache \
     npm pkgconfig openssl-dev openssl-libs-static zlib build-base

# create the app directory and copy in source
RUN mkdir -p /app/target/web
COPY frontend/ /app/frontend
COPY backend/ /app/backend

# build the svelte frontend
WORKDIR /app/frontend
RUN npm i; npm run build
RUN cp -r /app/frontend/build/* /app/target/web/

# build the rust backend
WORKDIR /app/backend
RUN cargo build --release
RUN cp /app/backend/target/release/hyde-backend /app/target/hyde

# runtime container
FROM alpine:latest AS runtime

# add required runtime packages
RUN apk add --no-cache libgcc

# copy in built files from builder
RUN mkdir -p /app/hyde-data/
WORKDIR /app
COPY --from=builder /app/target/ /app

# run the stuff
ENTRYPOINT ["/app/hyde"]