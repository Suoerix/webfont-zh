# WebFont-ZH

Webfont service used on Chinese Wikipedia and other Wikimedia projects.

## Features

- 🚀 **High-performance subsetting**: Font subsetting based on the [harfbuzz_rs_now](https://github.com/KonghaYao/harfbuzz_rs) library
- 📦 **WOFF2 compression**: Automatically converts TTF fonts to WOFF2 format for further size reduction
- 💾 **Intelligent caching system**: Supports both single-character and multi-character caching to improve response speed
- 🔄 **Font fallback mechanism**: Automatically falls back across multiple font files when needed
- 🌐 **RESTful API**: Provides a clean HTTP API interface
- ⚡ **Asynchronous processing**: High-performance async server built on Tokio

## API Endpoints

### 1. List Available Fonts

```http
GET /api/v1/list
```

**Example Response**:
```json
[
  {
    "id": "plangothic",
    "version": "2.9.5787",
    "font_family": "Plangothic",
    "license": "SIL Open Font License 1.1",
    "fallback": [
      "WenJinMincho"
    ],
    "name": {
      "zh-hans": "遍黑体",
      "zh-hant": "遍黑體"
    },
    "title": {
      "zh-hans": "[[遍黑體|遍黑体]]",
      "zh-hant": "[[遍黑體]]"
    }
  },
  {
    "id": "WenJinMincho",
    "version": "2.001",
    "font_family": "WenJinMincho",
    "license": "SIL Open Font License 1.1",
    "fallback": [
      "plangothic"
    ],
    "name": {
      "zh-hans": "文津宋体",
      "zh-hant": "文津明朝"
    },
    "title": {
      "zh-hans": "[https://github.com/takushun-wu/WenJinMincho 文津宋体]",
      "zh-hant": "[https://github.com/takushun-wu/WenJinMincho 文津明朝]"
    }
  }
]
```

### 2. Retrieve Font Subset

```http
GET /api/v1/font?id={font-id}&char={unicode-codepoints}
```
**Parameters**:
- `id`: Font ID (required)
- `char`: Unicode decimal codepoints, separated by commas (required)

**Response**:
- Content-Type: `application/font-woff2`
- Cache-Control: `public, max-age=31536000, immutable`

### 3. Regenerate Font Cache

```http
POST /api/v1/generate?id={font-id}&char={unicode-codepoints}
```

### 4. Access Static Files

```http
GET /static/{font-id}/{cache-filename}
```

**Cache filename rules**:
- Single character: `{unicode-codepoint}.woff2`
- Multiple characters: `cache/{codepoint1,codepoint2,codepoint3}.woff2`


## Web usage

### CSS Example

```css
@font-face {
  font-family: "Plangothic-20013";
  src: url("http://webfont-zh.toolforge.org/static/plangothic/20013.woff2") format("woff2"),
       url("http://webfont-zh.toolforge.org/api/v1/font?id=plangothic&char=20013") format("woff2");
  unicode-range: U+4E2D;
}

.inline-unihan {
  font-family: "Plangothic-20013", serif;
}
```

### JavaScript Example

```javascript
// Check font loading status
document.fonts.ready.then(function() {
  console.log('Fonts loaded');
});

// Dynamically load a font
function loadFont(fontId, codepoints) {
  const fontFace = new FontFace(
    `${fontId}-${codepoints.join(',')}`,
    `url(http://webfont-zh.toolforge.org/api/v1/font?id=${fontId}&char=${codepoints.join(',')})`
  );
  
  document.fonts.add(fontFace);
  return fontFace.load();
}
```

## Fonts
Most of the fonts are licensed under SIL OFL 1.1. See the `/data/fonts/` folder and their `config.json` for more details.

## Credits
- [cn-font-split](https://github.com/KonghaYao/cn-font-split). An intelligent font subsetting and packaging project desgined for Chinese characters, which inspired the implementation of our backend logic.

## License
Apache-2.0