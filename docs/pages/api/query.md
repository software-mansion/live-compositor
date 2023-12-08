---
description: Queries
---

# Queries

### Inputs

```typescript
type Query = {
  query: "inputs";
}

type Response = {
  inputs: Input[];
}

type Input = {
  id: string;
  port: u16;
}
```

***

### Outputs

```typescript
type Query = {
  query: "outputs";
}

type Response = {
  outputs: Output[];
}

type Output = {
  id: string;
  port: u16;
  ip: string;
}

```

***

### Wait for the next frame

Wait for next frame to be composed for the input. This query is useful if you want to send a scene update after input stream already reached the compositor.

```typescript
type Query = {
  query: "wait_for_next_frame";
  input_id: string;
}

type Response = {}
```

***

### Wait for the End-of-Stream (EOS)

Wait for end of an RTP stream identified by `input_id`. End should be signaled using RTCP `Goodbye` packet.

```typescript
type Query = {
  query: "wait_for_eos";
  input_id: string;
}

type Response = {}
```
