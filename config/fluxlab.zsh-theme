# FluxLab ZSH Theme
# A clean and informative theme for Flux Framework

# Colors
local ret_status="%(?:%{$fg_bold[green]%}➜ :%{$fg_bold[red]%}➜ )"
local user_host="%{$fg[cyan]%}%n@%m%{$reset_color%}"
local current_dir="%{$fg_bold[blue]%}%~%{$reset_color%}"
local git_branch='$(git_prompt_info)'
local time_stamp="%{$fg[yellow]%}[%*]%{$reset_color%}"

# Prompt
PROMPT="${time_stamp} ${user_host} ${current_dir} ${git_branch}
${ret_status}%{$reset_color%} "

# Git prompt settings
ZSH_THEME_GIT_PROMPT_PREFIX="%{$fg[magenta]%}("
ZSH_THEME_GIT_PROMPT_SUFFIX="%{$reset_color%} "
ZSH_THEME_GIT_PROMPT_DIRTY="%{$fg[magenta]%})%{$fg[red]%} ✗"
ZSH_THEME_GIT_PROMPT_CLEAN="%{$fg[magenta]%})%{$fg[green]%} ✓"

# Right prompt with timestamp
RPROMPT=""
