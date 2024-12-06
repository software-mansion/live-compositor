import { createContext, useContext, useEffect, useState } from 'react';
import { useAfterTimestamp } from '../hooks.js';

export class ChildrenLifetimeContext {
  private timestamps: Array<{ end: number }> = [];
  private onTimestampRemoved: () => void;

  constructor(onSlideEnd: () => void) {
    this.onTimestampRemoved = onSlideEnd;
  }

  public addEndTimestamp(ts: { end: number }) {
    this.timestamps.push(ts);
  }

  public removeEndTimestamp(ts: { end: number }) {
    this.timestamps = this.timestamps.filter(timestamp => ts !== timestamp);
    this.onTimestampRemoved();
  }

  public isDone(): boolean {
    return this.timestamps.length === 0;
  }
}

/**
 * Context that exposes API to children to register themself as playing/in-progress. Some components
 * will change their behavior based on the state of its in-direct children, e.g. Slides component will
 * not switch Slide until children are finished.
 */
export const ChildrenLifetimeContextType = createContext(new ChildrenLifetimeContext(() => {}));

/**
 * Internal helper hook that can be use inside other components to propagate
 * their duration/lifetime to the parents.
 */
export function useTimeLimitedComponent(timestamp: number) {
  const childrenLifetimeContext = useContext(ChildrenLifetimeContextType);
  const afterTimestamp = useAfterTimestamp(timestamp);
  const [timestampObject, setTimestampObject] = useState<{ end: number }>();
  useEffect(() => {
    let tsObject = { end: timestamp };
    setTimestampObject(tsObject);
    childrenLifetimeContext.addEndTimestamp(tsObject);
    return () => {
      childrenLifetimeContext.removeEndTimestamp(tsObject);
    };
  }, [timestamp]);

  useEffect(() => {
    if (timestampObject && afterTimestamp) {
      childrenLifetimeContext.removeEndTimestamp(timestampObject);
    }
  }, [afterTimestamp, timestampObject]);
}
