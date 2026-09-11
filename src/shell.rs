use crate::cli::Shell;

pub fn init_script(shell: Shell) -> &'static str {
    match shell {
        Shell::PowerShell => POWERSHELL,
        Shell::Cmd => CMD,
        Shell::Bash => BASH,
        Shell::Zsh => ZSH,
        Shell::Fish => FISH,
    }
}

const BASH: &str = r#"goto() {
    if [ "$#" -eq 0 ]; then
        command goto
        return $?
    fi

    case "$1" in
        -s|--save|-l|--list|-r|--remove|save|list|remove|init|-h|--help|-V|--version)
            command goto "$@"
            ;;
        --)
            if [ "$#" -ne 2 ]; then
                command goto "$@"
                return $?
            fi
            local _goto_target
            _goto_target="$(command goto __resolve -- "$2")" || return $?
            builtin cd -- "$_goto_target"
            ;;
        -*)
            command goto "$@"
            ;;
        *)
            local _goto_target
            _goto_target="$(command goto __resolve -- "$@")" || return $?
            builtin cd -- "$_goto_target"
            ;;
    esac
}
"#;

const ZSH: &str = r#"goto() {
    if [[ $# -eq 0 ]]; then
        command goto
        return $?
    fi

    case "$1" in
        -s|--save|-l|--list|-r|--remove|save|list|remove|init|-h|--help|-V|--version)
            command goto "$@"
            ;;
        --)
            if [[ $# -ne 2 ]]; then
                command goto "$@"
                return $?
            fi
            local _goto_target
            _goto_target="$(command goto __resolve -- "$2")" || return $?
            builtin cd -- "$_goto_target"
            ;;
        -*)
            command goto "$@"
            ;;
        *)
            local _goto_target
            _goto_target="$(command goto __resolve -- "$@")" || return $?
            builtin cd -- "$_goto_target"
            ;;
    esac
}
"#;

const FISH: &str = r#"function goto --description 'Jump to a saved workplace'
    if test (count $argv) -eq 0
        command goto
        return $status
    end

    switch $argv[1]
        case -s --save -l --list -r --remove save list remove init -h --help -V --version
            command goto $argv
        case --
            if test (count $argv) -ne 2
                command goto $argv
                return $status
            end
            set -l _goto_target (command goto __resolve -- $argv[2])
            or return $status
            cd -- $_goto_target
        case '-*'
            command goto $argv
        case '*'
            set -l _goto_target (command goto __resolve -- $argv)
            or return $status
            cd -- $_goto_target
    end
end
"#;

const POWERSHELL: &str = r#"function global:goto {
    if ($args.Count -eq 0) {
        & goto.exe
        return
    }

    $management = @('-s', '--save', '-l', '--list', '-r', '--remove', 'save', 'list', 'remove', 'init', '-h', '--help', '-V', '--version')
    if ($management -contains $args[0] -or ($args[0].StartsWith('-') -and $args[0] -ne '--')) {
        & goto.exe @args
        return
    }

    if ($args[0] -eq '--') {
        if ($args.Count -ne 2) {
            & goto.exe @args
            return
        }
        $key = $args[1]
    } else {
        $key = $args
    }

    $target = & goto.exe __resolve -- $key
    if ($LASTEXITCODE -eq 0) {
        Set-Location -LiteralPath $target
    }
}
"#;

const CMD: &str = "doskey goto=call goto-shell.cmd $*\r\n";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scripts_use_the_internal_resolver_or_cmd_helper() {
        for shell in [Shell::PowerShell, Shell::Bash, Shell::Zsh, Shell::Fish] {
            assert!(init_script(shell).contains("__resolve"));
        }
        assert!(init_script(Shell::Cmd).contains("goto-shell.cmd"));
    }
}
