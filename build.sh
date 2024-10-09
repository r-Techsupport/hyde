#!/bin/bash

BUILD_DIR="./target"

BACKEND_DIR="./backend"
BACKEND_BUILD="$BACKEND_DIR/target/release/hyde-backend"

FRONTEND_DIR="./frontend"
FRONTEND_BUILD="$FRONTEND_DIR/build"

check_target() {
  echo "Checking target folder"
  if [ -d $BUILD_DIR ]; then
    echo "Target folder exists, clearing for rebuild"
    rm -rf "${BUILD_DIR:?}/"*
    return
  fi
  mkdir $BUILD_DIR
}

build_frontend() {
  echo "Building frontend"
  (
    cd $FRONTEND_DIR || exit && npm i && npm run build
  )
}

build_backend() {
  echo "Building backend"
  (
    cd $BACKEND_DIR || exit
    cargo build --release
  )
}

copy_build() {
  echo "Copying builds to target build folder"
  mkdir -p "$BUILD_DIR/web" && cp -r "$FRONTEND_BUILD/"* "$BUILD_DIR/web"
  cp "$BACKEND_BUILD" "$BUILD_DIR/hyde"
}

copy_hyde_data() {
  echo "Copying hyde data"
  cp -r "$1" "$BUILD_DIR"
}

main() {
  check_target
  build_frontend
  build_backend
  copy_build
}

while getopts ":c:h" opt; do
  case ${opt} in
    c )
      if [ ! -d "$OPTARG" ]; then
        echo "Error: The specified directory '$OPTARG' does not exist."
        exit 1
      fi
      main
      copy_hyde_data "$OPTARG"
      ;;
    h )
      echo "Options: build.sh [-c {hyde-data folder}]"
      ;;
    : )
      echo "Error: Option -$OPTARG requires an argument."
      exit 1
      ;;
    \? )
      echo "Invalid option: -$OPTARG"
      exit 1
      ;;
  esac
done

# If no options were provided, run the main function
if [ $OPTIND -eq 1 ]; then
  main
fi
