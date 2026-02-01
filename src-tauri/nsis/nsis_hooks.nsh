!macro customInstall
  ; Rename the desktop shortcut
  Delete "$DESKTOP\Piney.lnk"
  CreateShortCut "$DESKTOP\小兄许.lnk" "$INSTDIR\Piney.exe"
  
  ; Rename the Start Menu shortcut
  Delete "$SMPROGRAMS\Piney\Piney.lnk"
  CreateShortCut "$SMPROGRAMS\Piney\小兄许.lnk" "$INSTDIR\Piney.exe"
!macroend

!macro customUnInstall
  ; Remove the custom shortcuts
  Delete "$DESKTOP\小兄许.lnk"
  Delete "$SMPROGRAMS\Piney\小兄许.lnk"
!macroend
