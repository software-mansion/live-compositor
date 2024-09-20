# Image

Represents an image asset uploaded to the compositor. Used by a [`Image` component](../components/Image).

## `Renderers.RegisterImage`

```typescript
type RegisterImage = {
  | { assetType: "png"; url?: string; serverPath?: string; }
  | { assetType: "jpeg"; url?: string; serverPath?: string; }
  | { assetType: "gif"; url?: string; serverPath?: string; }
  | { 
      assetType: "svg";
      url?: string;
      path?: string;
      resolution?: {
        width: u32,
        height: u32,
      };
    }
```

- `url` - Url to download an image. This field is mutually exclusive with the `path` field.
- `path` - Path to an image. This field is mutually exclusive with the `url` field.
- `asset_type` - Format of an image.
- `resolution` - The resolution at which an SVG image should be rendered.
