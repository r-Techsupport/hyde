echo "Building the frontend..."
cd frontend
npm i
npm run build
cd ..
echo "Copying frontend files..."
mkdir target\web
xcopy frontend\build target\web /s /e

echo "Building the backend..."
cd backend
cargo build --release
echo "Copying backend files..."
mkdir target\hyde
xcopy backend\target\release\hyde-backend.exe target\hyde.exe