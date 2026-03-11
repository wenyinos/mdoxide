# Welcome Page Test Instructions

The welcome page is not tested via this file — instead follow these steps:

## How to Trigger the Welcome Page

1. **Delete Ferrite's config/session data** (to simulate first run):
   - Windows: Delete `%APPDATA%\ferrite\` folder
   - Linux: Delete `~/.config/ferrite/` folder
   - macOS: Delete `~/Library/Application Support/ferrite/` folder
2. Launch Ferrite **without** any file arguments
3. The Welcome tab should appear

## Welcome Page Checklist

- [ ] Welcome tab appears on first launch
- [ ] Theme selector (Light/Dark) works
- [ ] Language dropdown shows available languages
- [ ] Editor settings toggles work (word wrap, line numbers, minimap, bracket matching, syntax highlighting)
- [ ] Max line width slider functions
- [ ] CJK font preference dropdown appears
- [ ] Auto-save toggle works
- [ ] "Get Started" button closes the welcome tab
- [ ] Welcome page does NOT appear on subsequent launches
- [ ] Welcome page does NOT appear when Ferrite is opened with a file argument
- [ ] Welcome page does NOT appear when session tabs are restored

## Settings Persistence

After completing the welcome page:
- [ ] Selected theme is applied
- [ ] Selected language is applied
- [ ] Editor settings match what was configured
- [ ] Settings survive app restart
