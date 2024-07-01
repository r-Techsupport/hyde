Write-Host "Building the frontend..."
Set-Location frontend
npm i
npm run build
Set-Location ..

Write-Host "Copying frontend files..."
New-Item -Path "target\web" -ItemType Directory
Copy-Item -Path "frontend\build\*" -Destination "target\web" -Recurse -Force

Write-Host "Building the backend..."
Set-Location backend
cargo build --release
Set-Location ..

Write-Host "Copying backend files..."
Copy-Item -Path "backend\target\release\hyde-backend.exe" -Destination "target\hyde.exe"