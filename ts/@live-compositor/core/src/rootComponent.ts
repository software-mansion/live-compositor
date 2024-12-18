import { _liveCompositorInternals, useAfterTimestamp, View } from 'live-compositor';
import { createElement, useEffect, useSyncExternalStore, type ReactElement } from 'react';

type CompositorOutputContext = _liveCompositorInternals.CompositorOutputContext;
type ChildrenLifetimeContext = _liveCompositorInternals.ChildrenLifetimeContext;

// External store to share shutdown information between React tree
// and external code that is managing it.
export class OutputShutdownStateStore {
  private shutdown: boolean = false;
  private onChangeCallbacks: Set<() => void> = new Set();

  public close() {
    this.shutdown = true;
    this.onChangeCallbacks.forEach(cb => cb());
  }

  // callback for useSyncExternalStore
  public getSnapshot = (): boolean => {
    return this.shutdown;
  };

  // callback for useSyncExternalStore
  public subscribe = (onStoreChange: () => void): (() => void) => {
    this.onChangeCallbacks.add(onStoreChange);
    return () => {
      this.onChangeCallbacks.delete(onStoreChange);
    };
  };
}

const globalDelayRef = Symbol();

export function OutputRootComponent({
  outputContext,
  outputRoot,
  outputShutdownStateStore,
  childrenLifetimeContext,
}: {
  outputContext: CompositorOutputContext;
  outputRoot: ReactElement;
  outputShutdownStateStore: OutputShutdownStateStore;
  childrenLifetimeContext: ChildrenLifetimeContext;
}) {
  const shouldShutdown = useSyncExternalStore(
    outputShutdownStateStore.subscribe,
    outputShutdownStateStore.getSnapshot
  );

  useMinimalStreamDuration(childrenLifetimeContext);

  if (shouldShutdown) {
    // replace root with view to stop all the dynamic code
    return createElement(View, {});
  }

  return createElement(
    _liveCompositorInternals.LiveCompositorContext.Provider,
    { value: outputContext },
    createElement(
      _liveCompositorInternals.ChildrenLifetimeContextType.Provider,
      { value: childrenLifetimeContext },
      outputRoot
    )
  );
}

/**
 * Add minimal 1 second lifetime in case there are not live
 * components inside the scene.
 */
function useMinimalStreamDuration(childrenLifetimeContext: ChildrenLifetimeContext) {
  useEffect(() => {
    childrenLifetimeContext.removeRef(globalDelayRef);
    return () => {
      childrenLifetimeContext.removeRef(globalDelayRef);
    };
  }, []);
  const afterTimestamp = useAfterTimestamp(1000);
  useEffect(() => {
    if (afterTimestamp) {
      childrenLifetimeContext.removeRef(globalDelayRef);
    }
  }, [afterTimestamp]);
}
