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
      serverPath?: string;
      resolution?: {
        width: u32,
        height: u32,
      };
    }
```

- `assetType` - Format of an image.
- `url` - Url to download an image. This field is mutually exclusive with the `path` field.
- `serverPath` - Path to an image (location on the server where LiveCompositor server is deployed). This field is mutually exclusive with the `url` field.
- `resolution` - The resolution at which an SVG image should be rendered.
