!ifndef VERSION
!define VERSION "0.3.1"
!endif

!define APPNAME "MDOxide"
!define EXE "mdoxide.exe"
; NSIS resolves relative paths from this script's directory.
!define PROJECT_ROOT "..\.."

Name "${APPNAME}"
OutFile "${PROJECT_ROOT}\dist\MDOxide-${VERSION}-setup.exe"
Unicode true
InstallDir "$PROGRAMFILES64\MDOxide"
RequestExecutionLevel admin

Page directory
Page instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File "${PROJECT_ROOT}\target\release\${EXE}"
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
