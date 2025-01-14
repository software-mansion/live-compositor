import type { ImageRef } from './imageRef.js';
import type { InputRef } from './inputRef.js';

type Ref = InputRef | ImageRef;

export function areRefsEqual(ref1: Ref, ref2: Ref): boolean {
  const sameType = ref1.type === ref2.type;
  const sameId = ref1.id === ref2.id;
  if (
    (ref1.type === 'output-specific-input' && ref2.type === 'output-specific-input') ||
    (ref1.type === 'output-specific-image' && ref2.type === 'output-specific-image')
  ) {
    return sameId && sameType && ref1.outputId === ref2.outputId;
  } else {
    return sameId && sameType;
  }
}
