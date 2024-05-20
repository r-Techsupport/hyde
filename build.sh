echo "Building the frontend..."
cd frontend; npm i; npm run build; cd ..
echo "Copying frontend files..."
mkdir -p target/web
cp -r frontend/build/ target/web

echo "Building the backend..."
cd backend; cargo build --release; cd ..
echo "Copying backend files..."
cp backend/target/release/rts-cms-backend target/rts-cms