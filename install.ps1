# installation script for voyage on Windows
$host.UI.RawUI.WindowTitle = "Voyage Installer"

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Output "Administrator privileges are required for this installation."
    $confirmation = Read-Host "Would you like to run with administrator privileges? (Y/N)"

    if ($confirmation -ne 'Y' -and $confirmation -ne 'y') {
        Write-Output "Installation cancelled."
        exit
    }

    Write-Output "Restarting script with administrator privileges..."
    Start-Process powershell -ArgumentList "-ExecutionPolicy Bypass -File `"$($MyInvocation.MyCommand.Path)`"" -Verb RunAs
    exit
}

$repoUrl = "https://github.com/clickswave/voyage.git"
$defaultInstallDir = "$env:ProgramFiles\clickswave"
$buildDir = "$env:TEMP\voyage_build"

# Parse arguments
$installDir = $defaultInstallDir

# Ensure dependencies are installed
function Check-Dependencies {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Error "Error: git is not installed. Please install git and try again."
        exit 1
    }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Error: Rust (cargo) is not installed. Please install Rust and try again."
        exit 1
    }
    if (-not (rustup show active-toolchain -ErrorAction SilentlyContinue)) {
        Write-Error "Error: Rust toolchain is not set. Please run the following command to set it up: rustup default stable"
        exit 1
    }
}

Check-Dependencies

# Ensure installDir is an absolute path
$installDir = [System.IO.Path]::GetFullPath($installDir)
$voyageDir = "$installDir\voyage"
$binDir = "$installDir\bin"

# Create installation directories
New-Item -ItemType Directory -Path $voyageDir -Force | Out-Null
New-Item -ItemType Directory -Path $binDir -Force | Out-Null

# Create build directory
Remove-Item -Recurse -Force $buildDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $buildDir | Out-Null
Write-Output "Using build directory: $buildDir"
Set-Location -Path $buildDir

# Clone repository
Write-Output "Cloning voyage repository..."
git clone $repoUrl .

# Build the project
Write-Output "Building voyage..."
cargo build --release

# Move binary to installation directory
Write-Output "Installing voyage..."
Move-Item "target\release\voyage.exe" "$voyageDir\voyage.exe" -Force

# Create symlink in bin directory
Write-Output "Creating symlink..."
New-Item -ItemType SymbolicLink -Path "$binDir\voyage.exe" -Target "$voyageDir\voyage.exe" -Force | Out-Null

# Add bin directory to PATH (system-wide since we're in Program Files)
$pathVariable = [System.Environment]::GetEnvironmentVariable("Path", [System.EnvironmentVariableTarget]::Machine)
if ($pathVariable -notlike "*$binDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$pathVariable;$binDir", [System.EnvironmentVariableTarget]::Machine)
    Write-Output "Added $binDir to system PATH. Restart your terminal to apply changes."
} else {
    Write-Output "$binDir is already in system PATH."
}

# Clean up
Write-Output "Cleaning up..."
Remove-Item -Recurse -Force $buildDir -ErrorAction SilentlyContinue

Write-Output "voyage installed successfully in $installDir! You can now run: voyage --help"