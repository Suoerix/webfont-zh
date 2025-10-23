# WebFont-ZH

Webfont service used on Chinese Wikipedia and other Wikimedia projects.

## Features

- 🚀 **High-performance subsetting**: Font subsetting based on the [harfbuzz_rs](https://github.com/harfbuzz/harfbuzz_rs) library
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
    "fallback": ["WenJinMincho"]
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
  src: url("http://localhost:3000/static/plangothic/20013.woff2") format("woff2"),
       url("http://localhost:3000/api/v1/font?id=plangothic&char=20013") format("woff2");
  unicode-range: U+4E2D;
}

.webfont-text {
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
    `url(http://localhost:3000/api/v1/font?id=${fontId}&char=${codepoints.join(',')})`
  );
  
  document.fonts.add(fontFace);
  return fontFace.load();
}
```

## Fonts
Most of the fonts are licensed under SIL OFL 1.1. See the `/data/fonts/` folder and their `config.json` for more details.

## License
Apache-2.0