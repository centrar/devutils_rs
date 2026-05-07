# DevUtils Fish Shell Completions

complete -c devutils -n "__fish_use_subcommand" -a "search" -d "AI semantic code search"
complete -c devutils -n "__fish_use_subcommand" -a "find" -d "Find files by name"
complete -c devutils -n "__fish_use_subcommand" -a "grep" -d "Fast grep replacement"
complete -c devutils -n "__fish_use_subcommand" -a "complete" -d "AI code completion"
complete -c devutils -n "__fish_use_subcommand" -a "refactor" -d "Refactor with patterns"
complete -c devutils -n "__fish_use_subcommand" -a "ai" -d "AI assistant"
complete -c devutils -n "__fish_use_subcommand" -a "explain" -d "Explain code"
complete -c devutils -n "__fish_use_subcommand" -a "generate" -d "Generate code"
complete -c devutils -n "__fish_use_subcommand" -a "debug" -d "Find and fix bugs"
complete -c devutils -n "__fish_use_subcommand" -a "tests" -d "Generate tests"
complete -c devutils -n "__fish_use_subcommand" -a "plugin" -d "Plugin management"
complete -c devutils -n "__fish_use_subcommand" -a "status" -d "Git status"
complete -c devutils -n "__fish_use_subcommand" -a "commits" -d "Recent commits"
complete -c devutils -n "__fish_use_subcommand" -a "branches" -d "List branches"
complete -c devutils -n "__fish_use_subcommand" -a "commit" -d "Create commit"
complete -c devutils -n "__fish_use_subcommand" -a "push" -d "Push to remote"
complete -c devutils -n "__fish_use_subcommand" -a "docker" -d "Docker commands"
complete -c devutils -n "__fish_use_subcommand" -a "k8s" -d "Kubernetes commands"
complete -c devutils -n "__fish_use_subcommand" -a "run" -d "Run project"
complete -c devutils -n "__fish_use_subcommand" -a "test" -d "Run tests"
complete -c devutils -n "__fish_use_subcommand" -a "build" -d "Build project"
complete -c devutils -n "__fish_use_subcommand" -a "project" -d "Project info"
complete -c devutils -n "__fish_use_subcommand" -a "system" -d "System info"
complete -c devutils -n "__fish_use_subcommand" -a "interactive" -d "Interactive TUI"
complete -c devutils -n "__fish_use_subcommand" -a "benchmark" -d "Run benchmarks"
complete -c devutils -n "__fish_use_subcommand" -a "version" -d "Show version"
complete -c devutils -n "__fish_use_subcommand" -a "help" -d "Show help"

function __fish_use_subcommand
    set -l cmd (commandline -opc)
    test (count $cmd) -eq 1
end