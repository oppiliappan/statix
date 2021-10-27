function! ApplyStatixSuggestion()
    let l:l = line('.')
    let l:c = col('.')
    let l:filter = "%!statix single -p " . l . "," . c . ""
    execute l:filter

    silent if v:shell_error == 1
        undo
    endif

    call cursor(l, c)
endfunction

nnoremap gf :call ApplyStatixSuggestion()<cr>
