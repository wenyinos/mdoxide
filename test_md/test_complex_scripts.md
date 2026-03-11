# Complex Script & Unicode Font Test

Test lazy font loading for complex scripts (v0.2.7).
Open this file and verify each script renders correctly (not as boxes/tofu).

## Arabic (Right-to-Left)

مرحبا بالعالم — Hello World in Arabic

بسم الله الرحمن الرحيم

الخط العربي جميل ومعقد

## Hebrew

שלום עולם — Hello World in Hebrew

## Devanagari (Hindi)

नमस्ते दुनिया — Hello World in Hindi

हिंदी में लिखा हुआ पाठ

## Bengali

হ্যালো বিশ্ব — Hello World in Bengali

## Tamil

வணக்கம் உலகம் — Hello World in Tamil

## Thai

สวัสดีชาวโลก — Hello World in Thai

## Georgian

გამარჯობა მსოფლიო — Hello World in Georgian

## Armenian

Բարեdelays աdelays — Test Armenian script

## Ethiopic

ሰላም ዓለም — Hello World in Ethiopic/Ge'ez

## Mixed Script Paragraph

This paragraph mixes Latin with العربية (Arabic), हिंदी (Hindi), 日本語 (Japanese), 한국어 (Korean), and ไทย (Thai) text within a single line to test font fallback behavior.

## Test Checklist

- [ ] Each script section renders with correct glyphs (no tofu boxes)
- [ ] Font loads lazily when scrolling to a new script section
- [ ] No visible lag when fonts load on demand
- [ ] Mixed-script paragraph renders all scripts correctly
- [ ] Font preferences in Settings -> Appearance -> Additional Scripts work
- [ ] CJK text in other files still loads correctly after opening this file
- [ ] Memory usage stays reasonable (not all fonts loaded at once)

## Existing CJK Tests

Open these files alongside this one to verify CJK still works:
- `test_japanese.md`
- `test_korean.md`
- `test_chinese.md`
