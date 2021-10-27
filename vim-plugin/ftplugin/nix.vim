setlocal makeprg=statix\ check\ -o\ errfmt\ %
setlocal errorformat=%f>%l:%c:%t:%n:%m

augroup StatixCheck
    autocmd!
    autocmd! BufWritePost *.nix | silent make! | silent redraw!
    autocmd QuickFixCmdPost [^l]* cwindow
augroup END
