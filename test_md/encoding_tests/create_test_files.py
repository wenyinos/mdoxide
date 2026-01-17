#!/usr/bin/env python3
"""
Creates test files in various encodings for testing Ferrite's multi-encoding support.
Run this script to regenerate the test files.
"""

import os

# Get the directory where this script is located
script_dir = os.path.dirname(os.path.abspath(__file__))

# UTF-8 without BOM
utf8_content = """# UTF-8 Test File (No BOM)

This file is encoded in UTF-8 without a Byte Order Mark.

## Special Characters
- German: Müller, Größe, Übung
- French: café, résumé, naïve
- Spanish: señor, niño, año
- Polish: żółć, ąę, ść

## Emoji and Symbols
- Emoji: 🎉 🚀 ✨ 📝
- Math: ∑ ∫ √ ∞ π
- Arrows: → ← ↑ ↓ ↔

This is the default encoding for most modern text files.
"""

with open(os.path.join(script_dir, "utf8_no_bom.md"), "w", encoding="utf-8") as f:
    f.write(utf8_content)

# UTF-8 with BOM
with open(os.path.join(script_dir, "utf8_with_bom.md"), "wb") as f:
    f.write(b'\xef\xbb\xbf')  # UTF-8 BOM
    f.write(utf8_content.encode("utf-8"))

# Windows-1252 (Western European)
windows1252_content = """# Windows-1252 Test File

This file is encoded in Windows-1252 (CP1252).

## Special Characters
- German: Müller, Größe, Übung
- French: café, résumé, naïve
- Spanish: señor, niño, año
- Copyright: © ® ™
- Currency: € £ ¥
- Quotes: "curly" 'quotes'
- Dash: — (em dash)

Windows-1252 is common in legacy Windows applications.
"""

with open(os.path.join(script_dir, "windows1252.txt"), "w", encoding="windows-1252") as f:
    f.write(windows1252_content)

# ISO-8859-1 (Latin-1)
latin1_content = """# ISO-8859-1 (Latin-1) Test File

This file is encoded in ISO-8859-1.

## Special Characters
- German: Müller, Größe, Übung
- French: café, résumé, naïve
- Spanish: señor, niño, año
- Copyright: ©
- Currency: £ ¥

Note: ISO-8859-1 does NOT include the Euro sign.
Latin-1 is a subset of Windows-1252.
"""

with open(os.path.join(script_dir, "latin1.txt"), "w", encoding="iso-8859-1") as f:
    f.write(latin1_content)

# Shift-JIS (Japanese)
shiftjis_content = """# Shift-JIS テストファイル

このファイルは Shift-JIS でエンコードされています。

## 日本語テキスト
- ひらがな: あいうえお かきくけこ
- カタカナ: アイウエオ カキクケコ
- 漢字: 日本語 東京 大阪 京都
- 記号: 。、！？「」

## 文例
私の名前は田中です。
今日はいい天気ですね。
ありがとうございます。

Shift-JIS は日本で広く使われているエンコーディングです。
"""

with open(os.path.join(script_dir, "shiftjis.txt"), "w", encoding="shift-jis") as f:
    f.write(shiftjis_content)

# EUC-JP (Japanese)
eucjp_content = """# EUC-JP テストファイル

このファイルは EUC-JP でエンコードされています。

## 日本語テキスト
- ひらがな: さしすせそ たちつてと
- カタカナ: サシスセソ タチツテト
- 漢字: 新聞 雑誌 本 図書館

## 文例
こんにちは。
お元気ですか。
さようなら。

EUC-JP は Unix/Linux システムでよく使われています。
"""

with open(os.path.join(script_dir, "eucjp.txt"), "w", encoding="euc-jp") as f:
    f.write(eucjp_content)

# GBK (Simplified Chinese)
gbk_content = """# GBK 测试文件

这个文件使用 GBK 编码。

## 中文文本
- 你好世界
- 北京 上海 广州 深圳
- 中华人民共和国

## 示例句子
今天天气很好。
欢迎来到中国。
谢谢你的帮助。

GBK 是简体中文常用的编码格式。
"""

with open(os.path.join(script_dir, "gbk.txt"), "w", encoding="gbk") as f:
    f.write(gbk_content)

# EUC-KR (Korean)
euckr_content = """# EUC-KR 테스트 파일

이 파일은 EUC-KR 인코딩을 사용합니다.

## 한글 텍스트
- 가나다라마바사
- 서울 부산 인천 대구
- 대한민국

## 예문
안녕하세요.
반갑습니다.
감사합니다.

EUC-KR은 한국어에 널리 사용되는 인코딩입니다.
"""

with open(os.path.join(script_dir, "euckr.txt"), "w", encoding="euc-kr") as f:
    f.write(euckr_content)

# UTF-16 LE (Little Endian) - with BOM for proper detection
utf16le_content = """# UTF-16 LE Test File

This file is encoded in UTF-16 Little Endian with BOM.

## Features
- Full Unicode support
- Common on Windows systems
- BOM: FF FE (at start of file)

## Mixed Scripts
- English: Hello World
- Japanese: こんにちは
- Chinese: 你好
- Korean: 안녕하세요
- Emoji: 🌍 🌎 🌏
"""

with open(os.path.join(script_dir, "utf16le.txt"), "wb") as f:
    f.write(b'\xff\xfe')  # UTF-16 LE BOM
    f.write(utf16le_content.encode("utf-16-le"))

# UTF-16 BE (Big Endian) - with BOM for proper detection
utf16be_content = """# UTF-16 BE Test File

This file is encoded in UTF-16 Big Endian with BOM.

## Features
- Full Unicode support
- Network byte order
- BOM: FE FF (at start of file)

## Mixed Scripts
- English: Hello World
- Japanese: さようなら
- Chinese: 再见
- Korean: 안녕히 가세요
"""

with open(os.path.join(script_dir, "utf16be.txt"), "wb") as f:
    f.write(b'\xfe\xff')  # UTF-16 BE BOM
    f.write(utf16be_content.encode("utf-16-be"))

print("Created encoding test files:")
print("  - utf8_no_bom.md (UTF-8 without BOM)")
print("  - utf8_with_bom.md (UTF-8 with BOM)")
print("  - windows1252.txt (Windows-1252)")
print("  - latin1.txt (ISO-8859-1)")
print("  - shiftjis.txt (Shift-JIS)")
print("  - eucjp.txt (EUC-JP)")
print("  - gbk.txt (GBK)")
print("  - euckr.txt (EUC-KR)")
print("  - utf16le.txt (UTF-16 LE)")
print("  - utf16be.txt (UTF-16 BE)")
