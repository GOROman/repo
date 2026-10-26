repo() {
    case " ${*} " in
        *" --list "*|*" --help "*|*" -h "*|*" --shell-init "*) command repo "$@"; return ;;
    esac
    local selection
    selection="$(command repo "$@")" || return
    [[ -z "$selection" ]] && return
    # Preserve spaces in paths; the first selected repository is the cd target.
    local first="${selection%%$'\n'*}"
    cd -- "$first" || return
    printf '\033[32m-> %s\033[0m\n' "$PWD"
}
