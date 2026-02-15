; VibeShell NSIS installer hooks
; Adds/removes the install directory to/from user PATH so that
; `vshell` CLI is available system-wide after installation.

!macro NSIS_HOOK_POSTINSTALL
  ; Read current user PATH from registry
  ReadRegStr $0 HKCU "Environment" "Path"
  StrCmp $0 "" 0 _vs_path_not_empty
    ; PATH is empty — set it to just $INSTDIR
    WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
    Goto _vs_path_done
  _vs_path_not_empty:
    ; PATH exists — append $INSTDIR
    WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR"
  _vs_path_done:
  ; Broadcast WM_SETTINGCHANGE so running shells pick up the change
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Read current user PATH
  ReadRegStr $0 HKCU "Environment" "Path"
  StrCmp $0 "" _vs_unpath_done
    ; Use nsExec + PowerShell to cleanly remove $INSTDIR from PATH
    nsExec::ExecToLog 'powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$$entries = [Environment]::GetEnvironmentVariable(''Path'',''User'') -split '';''; $$filtered = $$entries | Where-Object { $$_ -ne ''$INSTDIR'' -and $$_ -ne '''' }; [Environment]::SetEnvironmentVariable(''Path'', ($$filtered -join '';''), ''User'')"'
  _vs_unpath_done:
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend
