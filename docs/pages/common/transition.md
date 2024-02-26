## Transition

```typescript
type Transition = {
  duration_ms: f64;
  easing_function?: EasingFunction | EasingFunctionObject | CubicBezierEasing;
}

```

#### Properties

- `duration_ms` - Duration of the transition in milliseconds.
- `easing_function` - (**default="linear"**) Easing function to be used for the transition.

```typescript
type EasingFunctionObject = {
  "function_name": EasingFunction;
}

type EasingFunction =
  | "linear"
  | "ease"
  | "ease_in"
  | "ease_in_out"
  | "ease_in_quint"
  | "ease_out_quint"
  | "ease_in_out_quint"
  | "ease_in_expo"
  | "ease_out_expo"
  | "ease_in_out_expo"
  | "bounce"
```

You can find detailed information about the above easing functions [here](https://easings.net/).

```typescript
type CubicBezierEasing = {
  "function_name": "cubic_bezier";
  "points": [f64, f64, f64, f64];
}
```

Easing function defined by a cubic bezier curve. If cubic bezier result is outside of `[0, 1]` range, the result will be clamped to `[0, 1]`.

#### Properties

- `points` - Control points for cubic bezier. The array contains four values: `x1`, `y1`, `x2` and `y2` which define two control points for the cubic bezier curve. `x1` and `x2` values have to be in the range of `0.0` to `1.0`.
