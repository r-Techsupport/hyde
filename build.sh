mkdir -p target

echo "Building the frontend..."
cd frontend; npm i; npm run build; cd ..
echo "Linking frontend files..." # we are making it a link so you can build the frontend individually without manually copying it if you want
if ! [ -L target/web ]
then
    echo "target/web is not a symlink, removing it"
    rm -r target/web
else 
    unlink target/web
fi
ln -s ../frontend/build target/web

echo "Building the backend..."
cd backend; cargo build --release; cd ..
echo "Linking backend files..."
rm target/hyde
ln -s ../backend/target/release/hyde-backend target/hyde
