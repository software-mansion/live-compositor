import { useAfterTimestamp } from '../hooks.js';
import { LiveCompositorContext } from '../context/index.js';
import { useContext, useEffect, useState } from 'react';
import type { ComponentBaseProps } from '../component.js';

export type ShowProps = Omit<ComponentBaseProps, 'id'> & {
  /**
   * Optional range in milliseconds defining when the child should be visible.
   */
  timeRangeMs?: { start?: number; end?: number };
  /**
   * Delay of the child being shown in milliseconds.
   */
  delayMs?: number;
};

function Show(props: ShowProps) {
  if ('delayMs' in props && props.timeRangeMs) {
    throw new Error('"delayMs" and "timestamp" props can\'t be specified at the same time.');
  }
  if (props.timeRangeMs && !props.timeRangeMs.start && !props.timeRangeMs.end) {
    throw new Error('"timestampMs" prop needs to define at least one value "start" or "end".');
  }
  const ctx = useContext(LiveCompositorContext);
  const [mountTimestampMs, setStart] = useState<number>(() => ctx.timeContext.timestampMs());
  const afterStart = useAfterTimestamp(props.timeRangeMs?.start ?? 0);
  const afterEnd = useAfterTimestamp(props.timeRangeMs?.end ?? Infinity);
  const isAfterDelay = useAfterTimestamp(mountTimestampMs + (props.delayMs ?? 0));

  useEffect(() => {
    setStart(ctx.timeContext.timestampMs());
  }, []);

  if (props.delayMs !== undefined && isAfterDelay) {
    return props.children;
  } else if (props.timeRangeMs && afterStart && !afterEnd) {
    return props.children;
  } else {
    return null;
  }
}

export default Show;
