# Update the system and associated built utils
sudo pacman -Syu

# Setup rust
# Download rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > script.sh 
# Install rust
bash script.sh -y
# Setup environment
source $HOME/.cargo/env
# Remove install script
rm -rf script.sh

# Install git
sudo pacman -S git

# Clone our code
git clone https://github.com/adithyabhatkajake/libchatter-rs.git

# Get into our code
cd libchatter-rs

# Build experiments
cargo build --all