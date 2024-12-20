import { createElement, useContext, useEffect, useState, useSyncExternalStore } from 'react';
import type * as Api from '../api.js';
import { newBlockingTask } from '../hooks.js';
import { useTimeLimitedComponent } from '../context/childrenLifetimeContext.js';
import { LiveCompositorContext } from '../context/index.js';
import { inputRefIntoRawId, OfflineTimeContext } from '../internal.js';
import { InnerInputStream } from './InputStream.js';
import { newInternalStreamId } from '../context/internalStreamIdManager.js';

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

  /**
   *  Url or path to the mp4 file. File path refers to the filesystem where LiveCompositor server is deployed.
   */
  source: string;
};

function Mp4(props: Mp4Props) {
  const { muted, volume, ...otherProps } = props;
  const ctx = useContext(LiveCompositorContext);
  const [inputId, setInputId] = useState(0);

  useEffect(() => {
    const newInputId = newInternalStreamId();
    setInputId(newInputId);
    const task = newBlockingTask(ctx);
    const pathOrUrl =
      props.source.startsWith('http://') || props.source.startsWith('https://')
        ? { url: props.source }
        : { path: props.source };
    let registerPromise: Promise<any>;

    void (async () => {
      try {
        registerPromise = ctx.registerMp4Input(newInputId, {
          ...pathOrUrl,
          required: ctx.timeContext instanceof OfflineTimeContext,
          // offsetMs will be overridden by registerMp4Input implementation
        });
        await registerPromise;
      } finally {
        task.done();
      }
    })();
    return () => {
      task.done();
      void (async () => {
        await registerPromise.catch(() => {});
        await ctx.unregisterMp4Input(newInputId);
      })();
    };
  }, [props.source]);

  useInternalAudioInput(inputId, muted ? 0 : (volume ?? 1));
  useTimeLimitedMp4(inputId);

  return createElement(InnerInputStream, {
    ...otherProps,
    inputId: inputRefIntoRawId({ type: 'output-local', id: inputId, outputId: ctx.outputId }),
  });
}

function useInternalAudioInput(inputId: number, volume: number) {
  const ctx = useContext(LiveCompositorContext);
  useEffect(() => {
    if (inputId === 0) {
      return;
    }
    const options = { volume };
    ctx.audioContext.addInputAudioComponent(
      { type: 'output-local', id: inputId, outputId: ctx.outputId },
      options
    );
    return () => {
      ctx.audioContext.removeInputAudioComponent(
        { type: 'output-local', id: inputId, outputId: ctx.outputId },
        options
      );
    };
  }, [inputId, volume]);
}

function useTimeLimitedMp4(inputId: number) {
  const ctx = useContext(LiveCompositorContext);
  const [startTime, setStartTime] = useState(0);
  useEffect(() => {
    setStartTime(ctx.timeContext.timestampMs());
  }, [inputId]);

  const internalStreams = useSyncExternalStore(
    ctx.internalInputStreamStore.subscribe,
    ctx.internalInputStreamStore.getSnapshot
  );
  const input = internalStreams[String(inputId)];
  useTimeLimitedComponent((input?.offsetMs ?? startTime) + (input?.videoDurationMs ?? 0));
  useTimeLimitedComponent((input?.offsetMs ?? startTime) + (input?.audioDurationMs ?? 0));
}

export default Mp4;
