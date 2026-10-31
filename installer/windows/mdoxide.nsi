!ifndef VERSION
!define VERSION "0.5.1"
!endif
!ifndef ARCH
!define ARCH "x64"
!endif
!ifndef BIN_DIR
!define BIN_DIR "x86_64-pc-windows-msvc/release"
!endif

!define APPNAME "MDOxide"
!define EXE "mdoxide.exe"
; NSIS resolves relative paths from this script's directory.
!define PROJECT_ROOT "..\.."

; Keep x64 and ARM64 installs side by side.
!if "${ARCH}" == "arm64"
  !define APP_LABEL "MDOxide (ARM64)"
  !define INSTALL_DIR_NAME "MDOxide ARM64"
  !define REG_KEY "MDOxide-ARM64"
!else
  !define APP_LABEL "MDOxide"
  !define INSTALL_DIR_NAME "MDOxide"
  !define REG_KEY "MDOxide"
!endif

Name "${APP_LABEL}"
OutFile "${PROJECT_ROOT}\dist\MDOxide-${VERSION}-${ARCH}-setup.exe"
Unicode true
InstallDir "$PROGRAMFILES64\${INSTALL_DIR_NAME}"
RequestExecutionLevel admin

Page directory
Page instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File "${PROJECT_ROOT}\target\${BIN_DIR}\${EXE}"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateShortcut "$DESKTOP\${APP_LABEL}.lnk" "$INSTDIR\${EXE}"
  CreateDirectory "$SMPROGRAMS\${APP_LABEL}"
  CreateShortcut "$SMPROGRAMS\${APP_LABEL}\${APP_LABEL}.lnk" "$INSTDIR\${EXE}"
  CreateShortcut "$SMPROGRAMS\${APP_LABEL}\Uninstall.lnk" "$INSTDIR\uninstall.exe"

  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${REG_KEY}" "DisplayName" "${APP_LABEL}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${REG_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${REG_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$DESKTOP\${APP_LABEL}.lnk"
  Delete "$SMPROGRAMS\${APP_LABEL}\${APP_LABEL}.lnk"
  Delete "$SMPROGRAMS\${APP_LABEL}\Uninstall.lnk"
  RMDir "$SMPROGRAMS\${APP_LABEL}"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${REG_KEY}"
SectionEnd
