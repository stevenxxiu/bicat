# BICat
Show image via the *Kitty* image protocol.

To work with nested *Tmux*, use the `$TMUX_NEST_COUNT` environment variable.

To use with *Vifm*, set in `~/.config/vifm/vifmrc` with desired colors:

```vifm
" Images
fileviewer {*.bmp,*.jpg,*.jpeg,*.png,*.gif,*.svg,*.xpm},<image/*>
         \ bicat %c %px %py %pw %ph --fg 238 --bg 255
         \ %N %pd >/dev/tty </dev/tty
```

## Credits
- *Broot*
