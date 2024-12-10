import type React from 'react';

import { useAfterTimestamp } from '../hooks.js';
import { LiveCompositorContext } from '../context/index.js';
import { useContext, useEffect, useState } from 'react';

export type ShowProps = {
  timestampMs?: { start?: number; end?: number };
  delayMs?: number;
  children?: React.ReactNode;
};

function Show(props: ShowProps) {
  if ('delayMs' in props && props.timestampMs) {
    throw new Error('"delayMs" and "timestamp" props can\'t be specified at the same time.');
  }
  if (props.timestampMs && !props.timestampMs.start && !props.timestampMs.end) {
    throw new Error('"timestampMs" prop needs to define at least one value "start" or "end".');
  }
  const ctx = useContext(LiveCompositorContext);
  const [mountTimestampMs, setStart] = useState<number>(() => ctx.timeContext.timestampMs());
  const afterStart = useAfterTimestamp(props.timestampMs?.start ?? 0);
  const afterEnd = useAfterTimestamp(props.timestampMs?.end ?? Infinity);
  const isAfterDelay = useAfterTimestamp(mountTimestampMs + (props.delayMs ?? 0));

  useEffect(() => {
    setStart(ctx.timeContext.timestampMs());
  }, []);

  if (props.delayMs !== undefined && isAfterDelay) {
    return props.children;
  } else if (props.timestampMs && afterStart && !afterEnd) {
    return props.children;
  } else {
    return null;
  }
}

export default Show;
