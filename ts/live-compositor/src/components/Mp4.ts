import { createElement, useContext, useEffect, useState } from 'react';
import type * as Api from '../api.js';
import { newBlockingTask, useAudioInput, useBlockingTask } from '../hooks.js';
import { useTimeLimitedComponent } from '../context/childrenLifetimeContext.js';
import { LiveCompositorContext } from '../context/index.js';
import { OfflineTimeContext } from '../internal.js';
import { InnerInputStream } from './InputStream.js';
import { newInternalStreamId, useInternalStreamId } from '../context/internalStreamStore.js';

export type Mp4Props = {
  children?: undefined;

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Audio volume represented by a number between 0 and 1.
   */
  volume?: number;
  /**
   * Mute audio.
   */
  muted?: boolean;

  source: string;
};

function Mp4(props: Mp4Props) {
  const { muted, volume, ...otherProps } = props;
  const ctx = useContext(LiveCompositorContext);
  const [inputId, setInputId] = useState(0);

  const [result, setResult] = useState<T | undefined>(undefined);
  useEffect(() => {
    const newInputId = newInternalStreamId();
    setInputId(newInputId);
    const task = newBlockingTask(ctx);
    const pathOrUrl =
      props.source.startsWith('http://') || props.source.startsWith('https://')
        ? { url: props.source }
        : { serverPath: props.source };
    void (async () => {
      try {
        const registerResult = await ctx.registerMp4Input(newInputId, {
          ...pathOrUrl,
          required: ctx.timeContext instanceof OfflineTimeContext,
          // offsetMs will be overridden by registerMp4Input implementation
        });
        setResult(registerResult);
      } finally {
        task.done();
      }
    })();
    return () => {
      task.done();
      void ctx.unregisterMp4Input(newInputId);
    };
  }, [props.source]);

  const registerResult = useBlockingTask(async () => {
    await ctx.registerMp4Input({});
  });
  useAudioInput(inputId, {
    volume: muted ? 0 : (volume ?? 1),
  });

  return createElement(InnerInputStream, {
    ...otherProps,
    inputId: `output-local:${inputId}:${ctx.outputId}`,
  });
}

function useMp4InOfflineContext(inputId: string) {
  const ctx = useContext(LiveCompositorContext);
  if (!(ctx.timeContext instanceof OfflineTimeContext)) {
    // condition is constant so it's fine to use hook after that
    return;
  }
  const input = inputs[inputId];
  useTimeLimitedComponent((input?.offsetMs ?? 0) + (input?.videoDurationMs ?? 0));
  useTimeLimitedComponent((input?.offsetMs ?? 0) + (input?.audioDurationMs ?? 0));
}

export default Mp4;
