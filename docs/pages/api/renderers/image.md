# Image

Represents an image asset uploaded to the compositor. Used by a [`Image` component](../components/Image).

## Image

```typescript
type Image = {
  url?: string;
  path?: string;
} & Asset

type Asset =
  | { asset_type: "png" }
  | { asset_type: "jpeg" }
  | { asset_type: "gif" }
  | { 
      asset_type: "svg";
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
