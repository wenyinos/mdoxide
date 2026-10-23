!ifndef VERSION
!define VERSION "0.1.2"
!endif

!define APPNAME "MDOxide"
!define EXE "mdoxide.exe"

Name "${APPNAME}"
OutFile "dist\MDOxide-${VERSION}-setup.exe"
Unicode true
InstallDir "$PROGRAMFILES64\MDOxide"
RequestExecutionLevel admin

Page directory
Page instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File "target\release\${EXE}"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateShortcut "$DESKTOP\MDOxide.lnk" "$INSTDIR\${EXE}"
  CreateDirectory "$SMPROGRAMS\MDOxide"
  CreateShortcut "$SMPROGRAMS\MDOxide\MDOxide.lnk" "$INSTDIR\${EXE}"
  CreateShortcut "$SMPROGRAMS\MDOxide\Uninstall.lnk" "$INSTDIR\uninstall.exe"

  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MDOxide" "DisplayName" "${APPNAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MDOxide" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MDOxide" "UninstallString" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$DESKTOP\MDOxide.lnk"
  Delete "$SMPROGRAMS\MDOxide\MDOxide.lnk"
  Delete "$SMPROGRAMS\MDOxide\Uninstall.lnk"
  RMDir "$SMPROGRAMS\MDOxide"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MDOxide"
SectionEnd
