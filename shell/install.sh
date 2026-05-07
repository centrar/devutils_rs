#!/bin/bash
# DevUtils Shell Completions Installer

install_fish() {
    mkdir -p ~/.config/fish/completions
    curl -sL https://raw.githubusercontent.com/devutils/devutils/main/shell/devutils.fish > ~/.config/fish/completions/devutils.fish
    echo "✅ Fish shell completions installed"
}

install_bash() {
    mkdir -p ~/.devutils
    curl -sL https://raw.githubusercontent.com/devutils/devutils/main/shell/devutils.bash > ~/.devutils/devutils.bash
    echo "    source ~/.devutils/devutils.bash" >> ~/.bashrc
    echo "✅ Bash shell completions installed"
}

install_zsh() {
    mkdir -p ~/.zsh/completions
    curl -sL https://raw.githubusercontent.com/devutils/devutils/main/shell/devutils.zsh > ~/.zsh/completions/_devutils
    echo "✅ Zsh shell completions installed"
}

main() {
    case "$SHELL" in
        *fish)
            install_fish
            ;;
        *zsh)
            install_zsh
            ;;
        *bash)
            install_bash
            ;;
        *)
            echo "Installing for bash by default..."
            install_bash
            ;;
    esac
}

main "$@"