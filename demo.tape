Output demo.gif

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1400
Set Height 800

Hide
Env XDG_CONFIG_HOME "/tmp/demo"
Type "alias cs='./target/release/cs'"
Enter
Type "clear"
Enter
Show

#------------------------------------------------------------------------------
# ADD
#------------------------------------------------------------------------------

Type "cs add"
Sleep 500ms
Enter

Type "du -ah / | sort -rh | head -n 10"
Sleep 500ms
Enter

Type "Find 10 largest files"
Sleep 500ms
Enter

Type "files,find,size"
Sleep 500ms
Enter

Sleep 1500ms
Enter

Hide
Type "clear"
Enter
Show

#------------------------------------------------------------------------------
# ADD
#------------------------------------------------------------------------------

Type "cs add"
Sleep 500ms
Enter

Type "du -ah / | sort -rh | head -n 5"
Sleep 500ms
Enter

Type "Find 5 largest files"
Sleep 500ms
Enter

Type "files,find,size"
Sleep 500ms
Enter

Sleep 1500ms
Enter

Hide
Type "clear"
Enter
Show

#------------------------------------------------------------------------------
# ADD
#------------------------------------------------------------------------------

Type "cs add"
Sleep 500ms
Enter

Type "sudo firewall-cmd --permanent --list-ports"
Sleep 500ms
Enter

Type "List ports added to firewall"
Sleep 500ms
Enter

Type "firewall,ports"
Sleep 500ms
Enter

Sleep 1000ms
Type "fedora"
Sleep 500ms
Enter

Hide
Type "clear"
Enter
Show

#------------------------------------------------------------------------------
# LIST
#------------------------------------------------------------------------------

Type "cs list"
Sleep 500ms
Enter

Sleep 1000ms

Hide
Type "clear"
Enter
Show

#------------------------------------------------------------------------------
# LIST
#------------------------------------------------------------------------------

Type "cs list --all-os"
Sleep 500ms
Enter

Sleep 1000ms

Hide
Type "clear"
Enter
Show

#------------------------------------------------------------------------------
# SEARCH
#------------------------------------------------------------------------------

Type "cs"
Sleep 500ms
Enter

Sleep 1000ms

Type "find 10"
Sleep 1500ms
Enter

#------------------------------------------------------------------------------
# END
#------------------------------------------------------------------------------

Sleep 5s
