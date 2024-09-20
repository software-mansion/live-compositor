---
sidebar_position: 5
---
# Text

A component for rendering text.

## Text
```typescript
type Text = {
  id?: string;
  children: string;
  width?: number;
  height?: number;
  maxWidth?: number;
  maxHeight?: number;
  fontSize: number;
  lineHeight?: number;
  color?: string;
  backgroundColor?: string;
  font_family?: string;
  style?: "normal" | "italic" | "oblique";
  align?: "left" | "right" | "justified" | "center";
  wrap?: "none" | "glyph" | "word";
  weight?: 
    | "thin"
    | "extra_light"
    | "light"
    | "normal"
    | "medium"
    | "semi_bold"
    | "bold"
    | "extra_bold"
    | "black";
}
```

- `id` - Id of a component. Defaults to value produced by `useId` hook.
- `width` - Width of a texture that text will be rendered on. If not provided, the resulting texture
  will be sized based on the defined text but limited to `max_width` value.
- `height` - Height of a texture that text will be rendered on. If not provided, the resulting texture
  will be sized based on the defined text but limited to `max_height` value.
  It's an error to provide `height` if `width` is not defined.
- `maxWidth` - (**default=`7682`**) Maximal `width`. Limits the width of the texture that the text will be rendered on.
  Value is ignored if `width` is defined.
- `maxHeight` - (**default=`4320`**) Maximal `height`. Limits the height of the texture that the text will be rendered on.
  Value is ignored if height is defined.
- `fontSize` - Font size in pixels.
- `lineHeight` - Distance between lines in pixels. Defaults to the value of the `font_size` property.
- `color` - (**default=`"#FFFFFFFF"`**) Font color in `#RRGGBBAA` or `#RRGGBB` format.
- `backgroundColor` - (**default=`"#00000000"`**) Background color in `#RRGGBBAA` or `#RRGGBB` format.
- `fontFamily` - (**default=`"Verdana"`**) Font family. Provide [family-name](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value)
  for a specific font. "generic-family" values like e.g. "sans-serif" will not work.
- `style` - (**default=`"normal"`**) Font style. The selected font needs to support the specified style.
- `align` - (**default=`"left"`**) Text align.
- `wrap` - (**default=`"none"`**) Text wrapping options.
  - `"none"` - Disable text wrapping. Text that does not fit inside the texture will be cut off.
  - `"glyph"` - Wraps at a glyph level.
  - `"word"` - Wraps at a word level. Prevent splitting words when wrapping.
- `weight` - (**default=`"normal"`**) Font weight. The selected font needs to support the specified weight. Font weight, based on the [OpenType specification](https://learn.microsoft.com/en-gb/typography/opentype/spec/os2#usweightclass).
  - `"thin"` - Weight 100.
  - `"extra_light"` - Weight 200.
  - `"light"` - Weight 300.
  - `"normal"` - Weight 400.
  - `"medium"` - Weight 500.
  - `"semi_bold"` - Weight 600.
  - `"bold"` - Weight 700.
  - `"extra_bold"` - Weight 800.
  - `"black"` - Weight 900.
