# IME Input Test (Chinese / Japanese / Korean)

Test for GitHub Issue #91: Backspace during IME composition should not delete editor text.

---

## Test 1: Basic IME Backspace

1. Switch to a Chinese input method (Microsoft Pinyin, Xiaolanghao, etc.)
2. Place your cursor at the end of the line below:

一二三四五六七八九

3. Type `shi` (pinyin for 十)
4. You should see a preedit/composition underline showing candidates
5. Press **Backspace** once — the pinyin should change to `sh`
6. **Expected:** The character 九 should NOT be deleted
7. **Bug (before fix):** 九 gets deleted because the editor also processes the Backspace

---

## Test 2: Backspace All Pinyin Away

1. Place your cursor at the end of the line below:

你好世界

2. Type `ni` (pinyin for 你)
3. Press **Backspace** twice to clear all pinyin (`ni` → `n` → empty)
4. **Expected:** 界 and 世 should NOT be deleted — only the pinyin should be cleared
5. The IME composition should cancel, and the text should remain exactly: 你好世界

---

## Test 3: Normal Backspace (No IME)

1. Switch to English/direct input mode (no IME composition)
2. Place your cursor at the end of the line below:

ABCDEF

3. Press **Backspace** — F should be deleted
4. Press **Backspace** — E should be deleted
5. **Expected:** Normal backspace behavior works as before

---

## Test 4: IME Commit Then Backspace

1. Switch to Chinese input method
2. Place your cursor at the end of the line below:

测试文本

3. Type `hao` and select 好 (commit the character)
4. The line should now read: 测试文本好
5. Switch to English or press Backspace without starting new composition
6. **Expected:** 好 is deleted (normal backspace after IME commit)

---

## Test 5: Delete Key During IME

1. Place your cursor before the last character on the line below:

春夏秋冬

2. Start typing pinyin with Chinese IME
3. Press **Delete** key during composition
4. **Expected:** Only the IME composition is affected, 冬 is NOT deleted

---

## Test 6: Japanese IME (if available)

1. Switch to Japanese IME (Microsoft IME, Google Japanese Input)
2. Place your cursor at the end of the line below:

こんにちは

3. Type `sekai` (romaji for 世界)
4. Press **Backspace** to correct the romaji
5. **Expected:** は should NOT be deleted

---

## Test 7: Korean IME (if available)

1. Switch to Korean IME
2. Place your cursor at the end of the line below:

안녕하세요

3. Start composing a character
4. Press **Backspace** during composition
5. **Expected:** 요 should NOT be deleted

---

## Test 8: Multi-Cursor + IME

1. Use Ctrl+Click to place multiple cursors
2. Start IME composition at one cursor
3. Press **Backspace** during composition
4. **Expected:** No text deleted at any cursor position during composition

---

## Results

| Test | Pass/Fail | Notes |
|------|-----------|-------|
| 1. Basic IME Backspace | | |
| 2. Backspace All Pinyin | | |
| 3. Normal Backspace | | |
| 4. Commit Then Backspace | | |
| 5. Delete During IME | | |
| 6. Japanese IME | | |
| 7. Korean IME | | |
| 8. Multi-Cursor + IME | | |
