# Localization Configuration — String Tables & Locale Support

> **Description:** Defines the localization (L10n) system for *Kuruluş* — how all user-facing strings are stored, loaded, and displayed in multiple languages. Primary locale is Arabic (ar), with English (en) as fallback.

---

## 1. Design Principles

| # | Principle | Rationale |
|---|-----------|-----------|
| 1 | **No hardcoded strings** | Every display string is a key referencing a localization table |
| 2 | **Arabic first** | Arabic (ar) is the primary locale. English (en) is fallback |
| 3 | **RTL support** | Full right-to-left layout support for Arabic UI |
| 4 | **String formatting** | Dynamic values (numbers, names) injected via format strings |
| 5 | **Plural rules** | Arabic has complex plural rules (singular, dual, plural 3-10, plural 11+) |
| 6 | **Context awareness** | Same concept may translate differently depending on context (e.g., "attack" as noun vs verb) |
| 7 | **Hot-reloadable** | Locale files can be swapped at runtime (affects display only, not data) |

---

## 2. String Table Format

### 2.1 Data Structure

```json
{
  "locale": "ar",
  "version": 1,
  "strings": {
    "ui_main_menu_title": "قيامة",
    "ui_main_menu_new_game": "لعبة جديدة",
    "ui_main_menu_continue": "متابعة",
    "ui_main_menu_options": "الإعدادات",
    "ui_main_menu_quit": "خروج",

    "ui_pause_resume": "استئناف",
    "ui_pause_save": "حفظ",
    "ui_pause_load": "تحميل",
    "ui_pause_quit_to_menu": "العودة إلى القائمة",

    "unit_swordsman_name": "مبارز",
    "unit_swordsman_desc": "وحدة مشاة خفيفة مسلحة بالسيف والدرع. فعالة ضد المشاة الآخرين.",
    "unit_archer_name": "رامي",
    "unit_archer_desc": "وحدة مدى تطلق السهام من مسافة بعيدة. ضعيفة في الاشتباك المباشر.",

    "building_barracks_name": "ثكنة",
    "building_barracks_desc": "مبنى لتدريب وحدات المشاة.",
    "building_lumber_mill_name": "منشرة",
    "building_lumber_mill_desc": "يزيد من إنتاج الخشب.",

    "resource_gold_name": "ذهب",
    "resource_wood_name": "خشب",
    "resource_food_name": "طعام",

    "ability_sword_storm_name": "عاصفة السيوف",
    "ability_sword_storm_desc": "يطلق عاصفة من الضربات السريعة تضرب جميع الأعداء في منطقة قريبة.",

    "mission_01_name": "قلعة كولاجا حصار",
    "mission_01_desc": "اقتحام القلعة البيزنطية وتحريرها.",
    "mission_01_obj_01": "سيطرة على المعقل الرئيسي",
    "mission_01_obj_02": "القضاء على القائد البيزنطي",

    "notification_unit_attacked": "{unit_name} تتعرض للهجوم!",
    "notification_building_complete": "اكتمل بناء {building_name}.",
    "notification_resource_low": "تحذير: {resource_name} منخفض! ({current}/{max})",

    "dialog_greeting": "السلام عليكم يا أخي. لقد حان وقت التحرير.",
    "dialog_victory": "لقد حررنا هذه الأرض بعون الله.",
    "dialog_ambush_warning": "احذر! هناك كمين في الطريق أمامنا."
  }
}
```

### 2.2 String Key Convention

| Pattern | Example |
|---------|---------|
| `{domain}_{sub}_{item}` | `unit_swordsman_name` |
| `{domain}_{sub}_{item}_{variant}` | `notification_resource_low` |
| `{domain}_{id}_name` | `building_barracks_name` |
| `{domain}_{id}_desc` | `ability_sword_storm_desc` |
| `dialog_{context}_{id}` | `dialog_ambush_warning` |

---

## 3. Locale Support Matrix

| Locale | Code | RTL | Launch Support | Notes |
|--------|------|-----|----------------|-------|
| **Arabic** | `ar` | ✅ Yes | ✅ Yes | Primary locale |
| **English** | `en` | ❌ No | ✅ Yes | Fallback locale |
| **Turkish** | `tr` | ❌ No | 🟡 Post-launch | High demand |
| **French** | `fr` | ❌ No | 🟡 Post-launch | |
| **German** | `de` | ❌ No | ❌ Future | |
| **Spanish** | `es` | ❌ No | ❌ Future | |

---

## 4. Locale Manager

```python
class LocaleManager:
    def __init__(self):
        self.locales: dict[str, dict] = {}
        self.current_locale: str = "ar"
        self.fallback_locale: str = "en"

    def load_locale(self, locale_code: str):
        path = f"locales/{locale_code}.json"
        self.locales[locale_code] = load_json(path)

    def set_locale(self, locale_code: str):
        if locale_code not in self.locales:
            self.load_locale(locale_code)
        self.current_locale = locale_code
        self.config_manager.set("system.locale", locale_code)
        # Emit event for UI to refresh
        event_bus.emit("System_LocaleChanged", {"locale": locale_code})

    def get_string(self, key: str, **kwargs) -> str:
        # Try current locale
        table = self.locales.get(self.current_locale, {})
        if key in table:
            template = table[key]
            return self._format(template, **kwargs)

        # Try fallback
        table = self.locales.get(self.fallback_locale, {})
        if key in table:
            template = table[key]
            return self._format(template, **kwargs)

        # Key not found — return key itself (visible indicator of missing string)
        log_warning(f"Missing localization key: {key}")
        return f"[{key}]"

    def _format(self, template: str, **kwargs) -> str:
        """Replace {placeholder} with values"""
        import string
        try:
            return string.Template(template).safe_substitute(**kwargs)
        except:
            return template
```

---

## 5. Pluralization Rules

### 5.1 Arabic Plural Forms

Arabic has 6 plural forms:

| Form | Count | Example |
|------|-------|---------|
| **singular** | 1 | "جندي واحد" |
| **dual** | 2 | "جنديان" |
| **plural_few** | 3-10 | "جنود" (3-10) |
| **plural_many** | 11-99 | "جندياً" |
| **plural_other** | 0, 100+ | "جندي" |
| **zero** | 0 (explicit) | "لا جنود" |

### 5.2 Plural String Format

```json
{
  "unit_count": {
    "zero": "لا جنود",
    "one": "جندي واحد",
    "two": "جنديان",
    "few": "{count} جنود",
    "many": "{count} جندياً",
    "other": "{count} جندي"
  }
}
```

### 5.3 Pluralization Logic

```python
def get_plural_form(count: int, locale: str) -> str:
    if locale == "ar":
        if count == 0:
            return "zero"
        elif count == 1:
            return "one"
        elif count == 2:
            return "two"
        elif 3 <= count <= 10:
            return "few"
        elif 11 <= count <= 99:
            return "many"
        else:
            return "other"
    elif locale == "en":
        if count == 1:
            return "one"
        return "other"
    # Default English rules
    return "one" if count == 1 else "other"
```

---

## 6. RTL (Right-to-Left) Support

### 6.1 RTL Requirements

| Requirement | Implementation |
|-------------|---------------|
| Text alignment | All UI panels reverse alignment when locale is RTL |
| Mixed LTR/RTL | Numbers and English terms embedded in Arabic text handled correctly |
| Input fields | Text entry follows RTL direction |
| Icons and layout | Mirror icon positions where meaningful (e.g., back button) |
| Cursor | Cursor follows RTL when in RTL text |

### 6.2 Localization-Aware Layout

```python
# Pseudocode — layout mirroring for RTL
class UILayout:
    def __init__(self, is_rtl: bool):
        self.is_rtl = is_rtl

    def anchor_left(self):
        return "right" if self.is_rtl else "left"

    def anchor_right(self):
        return "left" if self.is_rtl else "right"

    def horizontal_direction(self):
        return -1 if self.is_rtl else 1  # For scroll, drag, etc.

    def back_button_icon(self):
        return "arrow_right" if self.is_rtl else "arrow_left"
```

---

## 7. Localization File Management

### 7.1 File Structure

```
locales/
├── ar.json          # Arabic (primary)
├── ar_phrases.json  # Arabic — combat barks, event messages
├── en.json          # English (fallback)
├── en_phrases.json  # English — combat barks
├── schema.json      # Localization schema definition
└── README.md        # Instructions for translators
```

### 7.2 Missing String Handling

| Scenario | Behavior |
|----------|----------|
| String missing in current locale | Fall back to English |
| String missing in all locales | Display the key in brackets: `[unit_unknown_name]` |
| Format argument missing | Display template as-is with `{arg}` placeholder visible |
| Locale file fails to load | Fall back to English, log error |

### 7.3 Tools for Translators

| Tool | Description |
|------|-------------|
| **String diff tool** | Compare two locale files, list missing keys |
| **Validation tool** | Check format arguments match between locales |
| **Context extractor** | Extract translatable strings from source code |
| **Google Sheets sync** | Export/import locale files to Google Sheets for collaborative translation |

---

## 8. Performance

| Metric | Target |
|--------|--------|
| Locale file size (ar, compressed) | < 50 KB |
| Locale load time | < 10 ms |
| String lookup (cached) | < 100 ns |
| String lookup (uncached) | < 1 µs |
| Locale switch UI refresh | < 16 ms (one frame) |

---

## 9. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Configuration architecture |
| `02_Runtime_Config.md` | Setting the locale via CVar `system.locale` |
| `04_Build_Config.md` | Embedding locale files in builds |
