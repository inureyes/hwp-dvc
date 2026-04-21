# HWPX fixture authoring guide

This directory holds the HWPX documents used as inputs for the
integration and golden tests. Each file is a **single-purpose**
fixture that exercises exactly one validator, in one pass-or-fail
mode, against the spec at
[`../specs/fixture_spec.json`](../specs/fixture_spec.json).

> `fixture_spec.json` is tailored to HWP's default environment
> (base font `함초롬바탕`, not `바탕`) so authors can use 한글's
> out-of-the-box settings without overriding the font on every save.
> The verbatim reference spec lives alongside as
> [`../specs/hancom_test.json`](../specs/hancom_test.json) for
> upstream-shape comparison, but it is not used by the fixture tests.

Fixtures must be authored in 한컴오피스 (한글 2020 or newer) — no other
authoring tool reliably produces HWPX. The files here are either
written by the project maintainers or generated from public-domain
templates. **Do not commit third-party documents** whose redistribution
rights are unclear.

## Naming convention

```
<category>_<condition>.hwpx
```

- `<category>` matches the validator module name
  (`charshape`, `parashape`, `table`, `specialchar`, `bullet`,
  `outline`, `paranum`, `style`, `hyperlink`, `macro`).
- `<condition>` is one of: `pass`, `fail_<reason>`, or a descriptive
  noun (e.g. `nested`, `multilevel`, `external`).
- Exactly one validator target per file. Do not stack multiple fail
  conditions into one document — diffing failures becomes ambiguous.

## Common authoring checklist

Before saving any fixture:

1. **Start from a blank document.** `파일 → 새 문서 → 빈 문서`.
   Do not base new fixtures on office templates — they drag in unused
   styles and fonts.
2. **Remove header/footer/page-number** unless the fixture explicitly
   tests them.
3. **Scrub metadata.** `파일 → 문서 정보`:
   - 제목 / 회사 / 키워드 / 메모: leave blank or set to
     `hwp-dvc fixture`.
   - 작성자: `hwp-dvc fixture` (or anonymous).
4. **Strip personal info at save time.**
   `도구 → 환경 설정 → 개인 정보 보호 → 저장 시 개인 정보 제거` ✓.
5. **Save as HWPX**, not HWP.
   `다른 이름으로 저장 → 파일 형식: 한글 문서 (*.hwpx)`.
6. **Keep it small.** One or two pages, one to three paragraphs.
   Target file size: **under 50 KB** for most fixtures. If a file
   balloons past 100 KB, something unused snuck in.

## Verifying a fixture after saving

```bash
# Archive structure — expect roughly these 10 parts:
unzip -l FIXTURE.hwpx
#   mimetype, version.xml
#   Contents/header.xml, Contents/section0.xml, Contents/content.hpf
#   META-INF/container.xml, META-INF/container.rdf, META-INF/manifest.xml
#   Preview/PrvText.txt, Preview/PrvImage.png, settings.xml

# Eyeball the header — charshape / parashape tables should be short:
unzip -p FIXTURE.hwpx Contents/header.xml | xmllint --format - | head -80

# Make sure no identifying strings leaked in:
strings FIXTURE.hwpx | grep -iE "inureyes|lablup|jshin|<your-name>"
# Should print nothing.
```

If `strings` catches your name, re-save with option 4 above ticked.

## Fixture index

Legend for **Status**: ✅ authored / ⏳ pending.

| File                              | Status | Category         | Expect errors (codes) |
|-----------------------------------|:------:|------------------|-----------------------|
| `charshape_pass.hwpx`             |  ✅    | CharShape        | none                  |
| `charshape_fail_font.hwpx`        |  ⏳    | CharShape        | 1004 (font)           |
| `charshape_fail_ratio.hwpx`       |  ⏳    | CharShape        | 1007 (ratio)          |
| `parashape_pass.hwpx`             |  ⏳    | ParaShape        | none                  |
| `parashape_fail_indent.hwpx`      |  ⏳    | ParaShape        | 2005 (indent)         |
| `parashape_fail_linespacing.hwpx` |  ⏳    | ParaShape        | 2007 (linespacing)    |
| `table_simple.hwpx`               |  ⏳    | Table            | none                  |
| `table_nested.hwpx`               |  ✅    | Table            | 3056 (table-in-table) |
| `specialchar_pass.hwpx`           |  ⏳    | SpecialCharacter | none                  |
| `specialchar_fail_ctrl.hwpx`      |  ⏳    | SpecialCharacter | 3101 (min-range)      |
| `bullet_allowed.hwpx`             |  ⏳    | Bullet           | none                  |
| `bullet_disallowed.hwpx`          |  ⏳    | Bullet           | 3304 (shapes)         |
| `outline_multilevel.hwpx`         |  ⏳    | OutlineShape     | varies by level       |
| `paranum_simple.hwpx`             |  ⏳    | ParaNumBullet    | varies by level       |
| `style_default_only.hwpx`         |  ⏳    | Style            | none                  |
| `style_custom.hwpx`               |  ⏳    | Style            | 3502 (permission)     |
| `hyperlink_none.hwpx`             |  ⏳    | Hyperlink        | none                  |
| `hyperlink_external.hwpx`         |  ⏳    | Hyperlink        | 6901 (permission)     |
| `macro_none.hwpx`                 |  ⏳    | Macro            | none                  |
| `macro_present.hwpx`              |  ⏳    | Macro            | 7001 (permission)     |

> Error code numbers are illustrative until the validators in
> `crates/hwp-dvc-core/src/checker/` define their exact constants.
> Treat them as "this is the category" hints, not contract values.
> The validators will finalize the exact codes in the order they are
> ported (issues #5 through #14).

## Per-fixture specifications

The authoring baseline is:

- Page: A4, default margins.
- Body font: **함초롬바탕** (HWP default), 10pt, ratio 100%, spacing 0.
- Paragraph: 양쪽 정렬, 줄 간격 160%, 들여쓰기 0, 문단 위/아래 간격 0.
- No header, footer, or page number.
- Sample body text: 한 단락 1~2 문장. Suggested text:
  > 이 문서는 hwp-dvc 테스트 픽스처입니다. 유효성 검사를 위해 작성되었습니다.

All fixtures start from this baseline unless noted. Because the
baseline matches 한글's defaults, authoring a pass fixture is usually
a matter of typing the sample text into a blank document and saving.

---

### CharShape

#### `charshape_pass.hwpx` ✅
- Baseline only. Body must use **함초롬바탕** throughout (the 한글
  default, so no font action needed).
- Expected against `fixture_spec.json`: **no errors**.

#### `charshape_fail_font.hwpx`
- Baseline body **plus** one word whose font is changed to a
  non-allowed family. `fixture_spec.json` allows only 함초롬바탕 and
  함초롬돋움, so any other family works — recommended: `맑은 고딕`
  or `Arial` (both clearly outside the allow-list).
- Highlight the word and `서식 → 글자 모양 → 글꼴`.
- Keep the rest of the paragraph in 함초롬바탕 so the fixture isolates
  the font-family error.
- Expected: **1 error** in the 1000 range on the non-baseline run.

#### `charshape_fail_ratio.hwpx`
- Select one paragraph and set `글자 모양 → 장평 150%`.
- Leave font and spacing at baseline.
- Expected: **1 error** in the 1000 range for ratio.

---

### ParaShape

#### `parashape_pass.hwpx`
- Baseline only, 2 paragraphs.
- Expected: no errors.

#### `parashape_fail_indent.hwpx`
- One paragraph with `문단 모양 → 첫 줄 → 들여쓰기 → 10pt` (spec
  requires 0).
- Expected: **1 error** in the 2000 range.

#### `parashape_fail_linespacing.hwpx`
- One paragraph with line spacing changed to 200% (spec requires
  160%, type = "글자에 따라").
- Expected: **1 error** in the 2000 range.

---

### Table

#### `table_simple.hwpx`
- 2×2 table, outer four borders = `실선 0.12mm 검정`, inner lines
  removed.
- `표 속성 → 글자처럼 취급` ✓ (spec requires `treatAsChar: true`).
- `표 속성 → 표 안에 표 → 허용 안 함`.
- Expected: no errors.

#### `table_nested.hwpx` ✅
- A 2×2 outer table; inside cell (1,1) insert a 1×1 inner table.
- Expected: **1 error** around the `table-in-table` rule (spec has
  `"table-in-table": false`).

---

### SpecialCharacter

#### `specialchar_pass.hwpx`
- Only Korean syllables + ASCII + common CJK punctuation (range
  covered by spec `minimum: 32`, `maximum: 1048575`).
- Expected: no errors.

#### `specialchar_fail_ctrl.hwpx`
- Hard to author in the GUI. Recommended procedure:
  1. Save a copy of `specialchar_pass.hwpx`.
  2. `unzip -p copy.hwpx Contents/section0.xml > section0.xml`
  3. Insert a `&#x7;` (or similar below-0x20) entity inside a `<hp:t>`
     element in `section0.xml`.
  4. Re-pack: `zip copy.hwpx Contents/section0.xml` (preserving the
     archive's other parts untouched).
- Expected: **1 error** in the 3100 range for minimum-range violation.

---

### Bullet

#### `bullet_allowed.hwpx`
- 3-item list using the allowed bullet shapes from the spec
  (`"bulletshapes": "□○-•*"`).
- Use `서식 → 글머리표 → 사각형 (□)`.
- Expected: no errors.

#### `bullet_disallowed.hwpx`
- Same 3 items but use `♠` or `▶` as the bullet (not in `bulletshapes`).
- Expected: **1 error** in the 3300 range.

---

### OutlineShape

#### `outline_multilevel.hwpx`
- Turn on outline view: `보기 → 개요`.
- 5 paragraphs with levels 1/2/3/4/1 in sequence, matching the
  `leveltype` array in the spec:
  - Level 1 `^1.`, shape = DIGIT (0)
  - Level 2 `^2.`, shape = HANGUL_SYLLABLE (8)
  - Level 3 `^3)`, shape = DIGIT (0)
  - Level 4 `^4)`, shape = HANGUL_SYLLABLE (8)
- Expected: no errors (this is intended as a pass fixture).
- If you want a fail variant later, create `outline_wrong_level3.hwpx`
  with level 3 using `(^3)` instead of `^3)`.

---

### ParaNumBullet

#### `paranum_simple.hwpx`
- 3 paragraphs with `서식 → 문단 번호` producing `1.`, `2.`, `3.`.
- Matches the `paranumbullet.leveltype` for level 1 in the spec.
- Expected: no errors.

---

### Style

#### `style_default_only.hwpx`
- Use only the built-in `바탕글` paragraph style.
- Expected: no errors.

#### `style_custom.hwpx`
- `서식 → 스타일 → 스타일 만들기` → name it `테스트스타일`, base on
  바탕글, apply to at least one paragraph.
- Expected: **1 error** in the 3500 range because spec has
  `"style": { "permission": false }`.

---

### Hyperlink

#### `hyperlink_none.hwpx`
- Baseline only.
- Expected: no errors.

#### `hyperlink_external.hwpx`
- Baseline plus one word with `입력 → 하이퍼링크 → 웹 주소:
  https://example.com`.
- Expected: **1 error** in the 6900 range.

---

### Macro

#### `macro_none.hwpx`
- Baseline only. Confirm with
  `unzip -l file.hwpx` that no `Scripts/` or similar macro part
  exists.
- Expected: no errors.

#### `macro_present.hwpx`
- `도구 → 매크로 → 새로 만들기` →
  ```
  Sub test
  End Sub
  ```
  save, close, re-open document to ensure the macro persists.
- Expected: **1 error** in the 7000 range.

---

## Authoring priority

If authoring everything in one sitting is too much, do them in this
order so each phase of the port has inputs when it needs them:

1. `charshape_pass.hwpx` ✅ (already committed) — smallest surface,
   unblocks Phase 1a header parser work.
2. `table_nested.hwpx` ✅ (already committed) — exercises recursive
   section walker for Phase 1b.
3. `macro_present.hwpx`, `macro_none.hwpx` — smallest validator
   (Phase 2, issue #7), good first end-to-end run.
4. `charshape_fail_font.hwpx`, `parashape_pass.hwpx`,
   `parashape_fail_indent.hwpx` — covers two most common validators.
5. Everything else, as the matching validator issue comes up.

## When XML-level edits are needed

Some fail cases (control chars, malformed IDs, corrupted border
entries) cannot be produced through the 한글 UI. For those, start
from a `*_pass.hwpx` copy and edit the unzipped XML directly:

```bash
cp original_pass.hwpx modified.hwpx
mkdir work && cd work
unzip ../modified.hwpx
# edit Contents/section0.xml or Contents/header.xml with an editor
zip -X ../modified.hwpx mimetype                    # mimetype first, STORED
zip -rX ../modified.hwpx . -x mimetype              # rest, deflated
cd .. && rm -rf work
```

The `mimetype` file must be stored (not deflated) and placed first in
the archive — same constraint ODF has. `-X` strips extra attributes
that some tools dislike.

## References

- Hancom DVC reference README:
  https://github.com/hancom-io/dvc/blob/main/README.md
- Full spec key catalog: [`../specs/hancom_full.json`](../specs/hancom_full.json)
  (documentation only, not a runnable spec)
- Fixture-suite spec (used by tests here):
  [`../specs/fixture_spec.json`](../specs/fixture_spec.json)
- Reference-shape spec (smoke test only):
  [`../specs/hancom_test.json`](../specs/hancom_test.json)
- OWPML model source: https://github.com/hancom-io/hwpx-owpml-model
