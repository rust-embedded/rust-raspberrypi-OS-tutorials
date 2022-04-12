define color_header
    @tput setaf 6 2> /dev/null || true
    @printf '\n%s\n' $(1)
    @tput sgr0 2> /dev/null || true
endef

define color_progress_prefix
    @tput setaf 2 2> /dev/null || true
    @tput bold 2 2> /dev/null || true
    @printf '%12s ' $(1)
    @tput sgr0 2> /dev/null || true
endef
