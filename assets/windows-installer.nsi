Unicode true
!ifndef APP_VERSION
  !define APP_VERSION "0.3.0"
!endif

Name "Obsidian Mods Manager"
OutFile "ObsidianModsManager-Setup.exe"
InstallDir "$LOCALAPPDATA\ObsidianMods"
RequestExecutionLevel user
SetCompressor /SOLID lzma

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Obsidian Mods Manager" SecMain
  SetOutPath "$INSTDIR"
  File "target\release\ObsidianModsManager.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\Obsidian Mods"
  CreateShortcut "$SMPROGRAMS\Obsidian Mods\Obsidian Mods Manager.lnk" "$INSTDIR\ObsidianModsManager.exe"
  CreateShortcut "$SMPROGRAMS\Obsidian Mods\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ObsidianModsManager" "DisplayName" "Obsidian Mods Manager"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ObsidianModsManager" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ObsidianModsManager" "Publisher" "Obsidian Mods"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ObsidianModsManager" "UninstallString" '"$INSTDIR\Uninstall.exe"'

  WriteRegStr HKCU "Software\Classes\obsidianmods" "" "URL:Obsidian Mods Protocol"
  WriteRegStr HKCU "Software\Classes\obsidianmods" "URL Protocol" ""
  WriteRegStr HKCU "Software\Classes\obsidianmods\DefaultIcon" "" '"$INSTDIR\ObsidianModsManager.exe",0'
  WriteRegStr HKCU "Software\Classes\obsidianmods\shell\open\command" "" '"$INSTDIR\ObsidianModsManager.exe" "%1"'
SectionEnd

Section "Uninstall"
  DeleteRegKey HKCU "Software\Classes\obsidianmods"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ObsidianModsManager"

  Delete "$SMPROGRAMS\Obsidian Mods\Obsidian Mods Manager.lnk"
  Delete "$SMPROGRAMS\Obsidian Mods\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Obsidian Mods"

  Delete "$INSTDIR\ObsidianModsManager.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
