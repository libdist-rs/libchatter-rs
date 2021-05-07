# Update packages
sudo pacman -Syu --noconfirm

# Install git
sudo pacman -S git --noconfirm

# Setup rust
# Download rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > script.sh 
# Install rust
bash script.sh -y
# Setup environment
source $HOME/.cargo/env
# Remove install script
rm -rf script.sh

if [ ! -d libchatter-rs ]; then 
    # Clone our code
    git clone https://github.com/adithyabhatkajake/libchatter-rs.git
fi

# Get into our code
cd libchatter-rs

git pull origin master

# Build experiments
cargo build --all --release