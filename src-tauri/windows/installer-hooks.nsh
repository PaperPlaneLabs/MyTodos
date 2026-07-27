!include LogicLib.nsh

Var LegacyInstallDir

!macro NSIS_HOOK_PREINSTALL
  ; Product names are part of Tauri's NSIS installation identity. Version
  ; 0.1.66 changed the product name from "my-todos" to "Todoz", so the default
  ; installer created a second installation instead of replacing the legacy
  ; one. Remember the legacy location so POSTINSTALL can turn it into a
  ; compatibility launcher without touching application data.
  ReadRegStr $LegacyInstallDir SHCTX "Software\${MANUFACTURER}\my-todos" ""
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ${If} $LegacyInstallDir != ""
  ${AndIf} $LegacyInstallDir != "$INSTDIR"
    ; The old app calls relaunch() after the updater finishes. Replace its
    ; executable with the new build so that relaunch reaches the compatibility
    ; redirect in app/install_migration.rs.
    CopyFiles /SILENT "$INSTDIR\${MAINBINARYNAME}.exe" "$LegacyInstallDir\${MAINBINARYNAME}.exe"

    ; Preserve the user's shortcut choices while moving their targets to the
    ; canonical Todoz installation.
    ${If} ${FileExists} "$SMPROGRAMS\my-todos.lnk"
      Delete "$SMPROGRAMS\my-todos.lnk"
      CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
      !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    ${EndIf}

    ${If} ${FileExists} "$DESKTOP\my-todos.lnk"
      Delete "$DESKTOP\my-todos.lnk"
      CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
      !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
    ${EndIf}

    ; Keep start-at-login enabled only when the user previously enabled it.
    ReadRegStr $R0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "my-todos"
    ${If} $R0 != ""
      WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}" '"$INSTDIR\${MAINBINARYNAME}.exe" --hidden'
      DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "my-todos"
    ${EndIf}

    ; The current installer has already written the Todoz registration. Remove
    ; the stale Add/Remove Programs and installer-location registrations, but
    ; deliberately leave the legacy directory in place as a compatibility
    ; launcher. Todoz user data lives outside both install directories.
    DeleteRegKey SHCTX "Software\Microsoft\Windows\CurrentVersion\Uninstall\my-todos"
    DeleteRegKey SHCTX "Software\${MANUFACTURER}\my-todos"
  ${EndIf}
!macroend
