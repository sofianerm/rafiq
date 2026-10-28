;; Rafiq garde ses données dans %APPDATA%\rafiq (config.json, horaires, presse-papier épinglé,
;; clés d'API de Clippy…), pas dans le dossier au nom de l'identifiant que l'uninstaller de Tauri
;; efface. Quand la case « Supprimer les données de l'application » est cochée (et que ce n'est pas
;; une mise à jour), ce dossier part aussi. %APPDATA%\codenotch, l'ancien nom, n'est jamais touché.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    RmDir /r "$APPDATA\rafiq"
  ${EndIf}
!macroend
